//! The simulation tick loop.
//!
//! Runs at TARGET_TICK_DURATION intervals (50Hz / 20ms).
//! Each tick: drain inputs → run_tick → apply results → record metrics.
//!
//! Spec: node-manager/MANIFEST.md "TICK LOOP"

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time;

use nexus_core::constants::{TARGET_TICK_DURATION, HIGH_LOAD_THRESHOLD_MS};
use nexus_simulation::run_tick;

use crate::WorldState;

/// Run the tick loop forever.
pub async fn run(state: Arc<RwLock<WorldState>>) -> Result<(), Box<dyn std::error::Error>> {
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

        // === Phase A: Collect inputs ===
        // TODO: drain action queues from connected clients
        let inputs = Vec::new();

        // === Phase B: Run simulation ===
        let tick_result = {
            let world = state.read().await;
            let dt = TARGET_TICK_DURATION; // TODO: use actual elapsed time
            run_tick(&world.snapshot, &inputs, dt)
        };

        // === Phase C: Apply results ===
        {
            let mut world = state.write().await;

            // Apply state changes to local snapshot
            // TODO: apply tick_result.state_changes to world.snapshot.bodies

            // Advance tick counter
            world.snapshot.tick_number = tick_result.next_tick_number;
            world.snapshot.timestamp_ms += (TARGET_TICK_DURATION * 1000.0) as u64;
        }

        // === Phase D: Broadcast to clients ===
        // TODO: send ENTITY_POSITION_UPDATE to connected clients

        // === Phase E: Flush ticker log ===
        // Phase 0 stub: no-op

        // === Phase F: Self-monitor ===
        let tick_duration = tick_start.elapsed();
        let tick_ms = tick_duration.as_secs_f32() * 1000.0;

        if tick_ms > HIGH_LOAD_THRESHOLD_MS {
            load_warning_count += 1;
            tracing::warn!(
                "Tick {} took {:.2}ms (budget: {:.0}ms) — warning {}/{}",
                tick_result.next_tick_number - 1,
                tick_ms,
                TARGET_TICK_DURATION * 1000.0,
                load_warning_count,
                nexus_core::constants::LOAD_GRACE_TICKS,
            );
        } else {
            load_warning_count = 0;
        }

        // Log every 250 ticks (~5 seconds)
        if tick_result.next_tick_number % 250 == 0 {
            let world = state.read().await;
            tracing::info!(
                "Tick {} — {:.2}ms — {} bodies — {} changes",
                tick_result.next_tick_number - 1,
                tick_ms,
                world.snapshot.bodies.len(),
                tick_result.state_changes.len(),
            );
        }
    }
}
