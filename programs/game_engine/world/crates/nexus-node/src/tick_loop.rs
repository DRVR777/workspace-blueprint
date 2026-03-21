//! The simulation tick loop.
//!
//! Runs at TARGET_TICK_DURATION intervals (100Hz / 10ms).
//! Each tick:
//!   Phase A: drain action queue (from connected clients)
//!   Phase B: run simulation (call run_tick)
//!   Phase C: apply results to world state
//!   Phase D: per-client interest management — send filtered physics updates
//!   Phase E: ticker log (stub)
//!   Phase F: self-monitor (tick duration, load warnings)
//!
//! Interest management (Phase D):
//!   Each client only receives updates for entities within DEFAULT_VISIBILITY_RADIUS.
//!   The octree query is O(log N) per client. On full-sync ticks (every
//!   PREDICTION_HORIZON_TICKS), all nearby bodies are sent to correct drift.
//!   On delta ticks, only non-inertial nearby bodies are sent.
//!
//! Spec: node-manager/MANIFEST.md "TICK LOOP"

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time;

use nexus_core::constants::{
    TARGET_TICK_DURATION, HIGH_LOAD_THRESHOLD_MS, LOAD_GRACE_TICKS, PREDICTION_HORIZON_TICKS,
    DEFAULT_VISIBILITY_RADIUS,
};
use nexus_core::types::{ChangeRequest, MotionState, ObjectId};
use nexus_simulation::run_tick_mut;

use crate::{WorldState, QueuedAction};
use crate::clients::ClientManager;
use crate::protocol;

/// Run the tick loop forever.
pub async fn run(
    state: Arc<RwLock<WorldState>>,
    mut action_rx: mpsc::UnboundedReceiver<QueuedAction>,
    client_manager: Arc<RwLock<ClientManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_interval = Duration::from_secs_f32(TARGET_TICK_DURATION);
    let mut interval = time::interval(tick_interval);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    let mut load_warning_count: u32 = 0;

    // Track the last input sequence number processed per player entity.
    // Sent back in PHYSICS_DELTA so the client can prune its input replay buffer.
    let mut last_processed_seq: HashMap<ObjectId, u32> = HashMap::new();

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
            // Track the highest sequence number seen per player this tick.
            // The max handles out-of-order delivery and multiple inputs per tick.
            let seq = queued.request.sequence_number;
            if seq > 0 {
                let entry = last_processed_seq.entry(queued.request.source).or_insert(0);
                if seq > *entry {
                    *entry = seq;
                }
            }
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

        // === Phase D: Per-client interest-managed physics updates ===
        //
        // For each client:
        //   1. Find their entity position in the world snapshot.
        //   2. Query the octree for entities within DEFAULT_VISIBILITY_RADIUS.
        //   3. On full-sync ticks: send all visible dynamic bodies (drift correction).
        //   4. On delta ticks: send only non-inertial visible dynamic bodies.
        //
        // This replaces the flat broadcast: clients only receive what they can see.
        // Bandwidth scales with local density, not world population.

        let cm = client_manager.read().await;
        let client_count = cm.count();

        if client_count > 0 {
            let world = state.read().await;

            let is_full_sync_tick = tick_number % PREDICTION_HORIZON_TICKS == 0;

            for client in cm.iter_clients() {
                // Look up client's own entity position
                let client_pos = world.snapshot.bodies.iter()
                    .find(|b| b.object_id == client.entity_id)
                    .map(|b| b.position);

                let Some(pos) = client_pos else { continue };

                // Octree query: entities within visibility radius
                let visible_ids: HashSet<ObjectId> = world.spatial_index
                    .query_radius(pos, DEFAULT_VISIBILITY_RADIUS)
                    .into_iter()
                    .collect();

                // Last input seq we processed for this player — used by client to prune buffer
                let ack_seq = last_processed_seq.get(&client.entity_id).copied().unwrap_or(0);

                if is_full_sync_tick {
                    // Send all visible dynamic bodies — corrects accumulated prediction drift
                    let to_send: Vec<&nexus_core::types::PhysicsBody> = world.snapshot.bodies.iter()
                        .filter(|b| b.is_dynamic() && visible_ids.contains(&b.object_id))
                        .collect();

                    if !to_send.is_empty() {
                        let msg = protocol::encode_full_sync(&to_send, ack_seq);
                        let _ = client.tx.send(msg);
                    }
                } else {
                    // Send only non-inertial visible bodies (Accelerating or Collision)
                    let to_send: Vec<&nexus_core::types::PhysicsBody> = world.snapshot.bodies.iter()
                        .filter(|b| {
                            b.is_dynamic()
                                && visible_ids.contains(&b.object_id)
                                && b.motion_state != MotionState::Inertial
                        })
                        .collect();

                    if !to_send.is_empty() {
                        let msg = protocol::encode_physics_delta(&to_send, ack_seq);
                        let _ = client.tx.send(msg);
                    }
                }
            }

            // TICK_SYNC every 10 ticks (~100ms) — clock alignment for all clients
            if tick_number % 10 == 0 {
                cm.send_to_all(protocol::encode_tick_sync(tick_number));
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

        // Log every 500 ticks (~5 seconds)
        if tick_number % 500 == 0 {
            tracing::info!(
                "Tick {} — {:.2}ms — {} bodies — {} clients — {} actions",
                tick_number, tick_ms, body_count, client_count, input_count,
            );
        }
    }
}
