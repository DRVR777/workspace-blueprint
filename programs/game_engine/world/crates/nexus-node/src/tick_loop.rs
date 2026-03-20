//! The simulation tick loop.
//!
//! Runs at TARGET_TICK_DURATION intervals (50Hz / 20ms).
//! Each tick:
//!   Phase A: drain action queue (from connected clients)
//!   Phase B: run simulation (call run_tick)
//!   Phase C: apply results to world state
//!   Phase D: broadcast position updates to all clients
//!   Phase E: ticker log (stub)
//!   Phase F: self-monitor (tick duration, load warnings)
//!
//! Spec: node-manager/MANIFEST.md "TICK LOOP"

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time;

use nexus_core::constants::{TARGET_TICK_DURATION, HIGH_LOAD_THRESHOLD_MS, LOAD_GRACE_TICKS};
use nexus_core::types::ChangeRequest;
use nexus_simulation::run_tick_mut;

use crate::{WorldState, QueuedAction};
use crate::clients::ClientManager;
use crate::protocol;

/// Run the tick loop forever.
pub async fn run(
    state: Arc<RwLock<WorldState>>,
    mut action_rx: mpsc::UnboundedReceiver<QueuedAction>,
    broadcast_tx: broadcast::Sender<Vec<u8>>,
    client_manager: Arc<RwLock<ClientManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_interval = Duration::from_secs_f32(TARGET_TICK_DURATION);
    let mut interval = time::interval(tick_interval);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    let mut load_warning_count: u32 = 0;

    tracing::info!(
        "Tick loop started — target {:.0}Hz ({:.1}ms)",
        1.0 / TARGET_TICK_DURATION,
        TARGET_TICK_DURATION * 1000.0,
    );

    loop {
        interval.tick().await;
        let tick_start = Instant::now();

        // === Phase A: Drain action queue ===
        let mut inputs: Vec<ChangeRequest> = Vec::new();
        while let Ok(queued) = action_rx.try_recv() {
            inputs.push(queued.request);
        }

        // === Phase B + C: Run simulation IN PLACE and apply results ===
        let (tick_number, body_count, input_count) = {
            let mut world = state.write().await;

            // run_tick_mut mutates snapshot.bodies directly — no clone, no re-apply
            let result = run_tick_mut(&mut world.snapshot, &inputs, TARGET_TICK_DURATION);

            // Update tick counter
            world.snapshot.tick_number = result.next_tick_number;
            world.snapshot.timestamp_ms += (TARGET_TICK_DURATION * 1000.0) as u64;

            // Update spatial index for moved bodies
            let moved: Vec<(u64, nexus_core::math::Vec3f64)> = world.snapshot.bodies.iter()
                .filter(|b| b.is_dynamic())
                .map(|b| (b.object_id, b.position))
                .collect();
            for (id, pos) in moved {
                world.spatial_index.move_object(id, pos);
            }

            (world.snapshot.tick_number, world.snapshot.bodies.len(), inputs.len())
        };

        // === Phase D: Broadcast position updates to all clients ===
        let client_count = {
            let cm = client_manager.read().await;
            cm.count()
        };

        if client_count > 0 {
            // Send ENTITY_POSITION_UPDATE
            let positions = {
                let world = state.read().await;
                protocol::encode_position_updates(&world.snapshot.bodies)
            };
            let _ = broadcast_tx.send(positions);

            // Send TICK_SYNC every 10 ticks (~200ms)
            if tick_number % 10 == 0 {
                let sync = protocol::encode_tick_sync(tick_number);
                let _ = broadcast_tx.send(sync);
            }
        }

        // === Phase E: Ticker log (Phase 0 stub) ===
        // No-op — would write to distributed log in production

        // === Phase F: Self-monitor ===
        let tick_duration = tick_start.elapsed();
        let tick_ms = tick_duration.as_secs_f32() * 1000.0;

        if tick_ms > HIGH_LOAD_THRESHOLD_MS {
            load_warning_count += 1;
            if load_warning_count > LOAD_GRACE_TICKS {
                tracing::error!(
                    "Tick {} OVERLOADED: {:.2}ms (budget: {:.0}ms) — {} consecutive overloads",
                    tick_number, tick_ms, TARGET_TICK_DURATION * 1000.0, load_warning_count,
                );
                // Phase 0: log only. Phase 2: request domain split.
            } else {
                tracing::warn!(
                    "Tick {} slow: {:.2}ms ({}/{})",
                    tick_number, tick_ms, load_warning_count, LOAD_GRACE_TICKS,
                );
            }
        } else {
            load_warning_count = 0;
        }

        // Log every 250 ticks (~5 seconds)
        if tick_number % 250 == 0 {
            tracing::info!(
                "Tick {} — {:.2}ms — {} bodies — {} clients — {} actions",
                tick_number, tick_ms, body_count, client_count, input_count,
            );
        }
    }
}
