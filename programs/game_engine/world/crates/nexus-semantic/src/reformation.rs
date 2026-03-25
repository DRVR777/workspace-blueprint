//! Bone 4 — drift detection and identity reformation.
//!
//! Drift: an identity file that consistently produces poor outputs has
//! misaligned content. Its outputs land in the wrong region of the field.
//! The quality scorer writes quality scores back to the event log via
//! EventLog::score(). The drift detector reads those scores and surfaces
//! addresses where the last `window` scored events all fall below `threshold`.
//!
//! Reformation: the candidate's content is rewritten using actual output text
//! from its poorest recent events, re-embedded, the full store re-laid out,
//! and swapped in. The file moves in 3D space. The motion is not a metaphor —
//! the world_coord changes because the embedding changed, because the content
//! changed, because the outputs were wrong.
//!
//! The agent does not trigger its own reformation. The drift detector does.
//! An identity cannot accurately evaluate its own drift using the drifted
//! judgment it's trying to correct.

use nexus_core::types::{PacketData, SemanticPacket};
use nexus_events::EventLog;
use crate::identity::{IdentityFile, IdentityStore};
use crate::layout::apply_layout;
use crate::llm::LlmClient;
use crate::worker::RoutingLoop;

// ─── DriftDetector ────────────────────────────────────────────────────────────

/// Reads the event log and surfaces identity addresses whose recent outputs
/// are consistently poor-quality.
pub struct DriftDetector {
    /// How many consecutive scored EVENT records must fall below threshold.
    pub window: usize,
    /// Quality threshold. Events with quality < threshold count as "poor".
    pub threshold: f32,
}

impl DriftDetector {
    pub fn new(window: usize, threshold: f32) -> Self {
        Self { window, threshold }
    }

    /// Return the dworld:// addresses of all identities whose last `window`
    /// scored EVENT records all have quality < threshold.
    ///
    /// Unscored records (quality IS NULL) are skipped — we cannot evaluate
    /// what has not been scored. An identity needs at least `window` scored
    /// events before it can be flagged as a drift candidate.
    pub fn candidates(&self, events: &EventLog) -> Vec<String> {
        let identities = match events.distinct_identities() {
            Ok(ids) => ids,
            Err(_) => return vec![],
        };

        let mut candidates = Vec::new();
        for address in identities {
            let recent = match events.recent_events_for_identity(&address, self.window) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // Collect only the scored events from the window.
            let scored: Vec<f32> = recent.iter()
                .filter_map(|r| r.quality)
                .collect();

            // Need a full window of scored events to make a decision.
            if scored.len() < self.window {
                continue;
            }

            // All window events below threshold → drift candidate.
            if scored.iter().all(|&q| q < self.threshold) {
                candidates.push(address);
            }
        }
        candidates
    }
}

// ─── Reformation ──────────────────────────────────────────────────────────────

/// Rewrite one identity file's content from its failure history, re-embed,
/// re-layout the full store, swap it in, and write a SIGNAL to the log.
///
/// This is the only path that moves an identity in 3D space. The motion is
/// evidence that the content changed, not a cause of anything.
pub async fn reform_identity(
    address: &str,
    loop_: &RoutingLoop,
    llm: &dyn LlmClient,
    events: &EventLog,
) -> Result<(), String> {
    // ── 1. Current identity ───────────────────────────────────────────────────
    let current = loop_
        .get_identity(address)
        .ok_or_else(|| format!("identity not found: {address}"))?;

    // ── 2. Gather failure evidence from the log ───────────────────────────────
    let recent = events
        .recent_events_for_identity(address, 10)
        .map_err(|e| e.to_string())?;

    let failures: Vec<&str> = recent.iter()
        .filter(|r| r.quality.map(|q| q < 0.5).unwrap_or(false))
        .map(|r| r.output.as_str())
        .collect();

    if failures.is_empty() {
        return Err(format!("no scored failure events found for {address}"));
    }

    let failure_context = failures.iter()
        .enumerate()
        .map(|(i, output)| format!("Failure {}: {}", i + 1, output))
        .collect::<Vec<_>>()
        .join("\n");

    // ── 3. Rewrite content via LLM ────────────────────────────────────────────
    // We reuse the existing complete(identity_content, packet) signature:
    //   identity_content = the meta-instruction for how to rewrite
    //   packet.data      = the current content + failure examples
    // This maps correctly to system/user in every LlmClient implementation.
    let rewrite_instruction = "\
You are rewriting an identity file that has been producing poor outputs. \
The current content and failure examples are provided below. \
Rewrite the identity to address the failures: keep the same role and \
perspective, but correct the approach that led to these poor outputs. \
Output only the rewritten identity file content — no preamble, no commentary.";

    let user_text = format!(
        "Current identity content:\n{}\n\nFailure examples from event log:\n{}",
        current.content, failure_context
    );

    let reform_packet = SemanticPacket::new(
        0, 0,
        PacketData::Text(user_text),
        address.to_string(),
    );

    let new_content = llm
        .complete(rewrite_instruction, &reform_packet)
        .await
        .map_err(|e| e.to_string())?;

    // ── 4. Re-embed the new content ───────────────────────────────────────────
    let new_vector = llm
        .embed(&new_content)
        .await
        .map_err(|e| e.to_string())?;

    // ── 5. Rebuild store with the reformed identity + re-layout ──────────────
    let updated_store = {
        let store = loop_.current_store();
        let updated_files: Vec<IdentityFile> = store.iter()
            .map(|f| {
                if f.address == address {
                    IdentityFile {
                        address:     f.address.clone(),
                        content:     new_content.clone(),
                        vector:      new_vector.clone(),
                        world_coord: f.world_coord, // overwritten by apply_layout
                    }
                } else {
                    f.clone()
                }
            })
            .collect();
        apply_layout(&IdentityStore::build(updated_files), 3)
    };

    // ── 6. Swap the store — the identity moves in 3D space ───────────────────
    loop_.apply_reformed_store(updated_store);

    // ── 7. Write a SIGNAL to the permanent log ────────────────────────────────
    let mean_failure_quality: f32 = recent.iter()
        .filter_map(|r| r.quality)
        .filter(|&q| q < 0.5)
        .sum::<f32>()
        / failures.len() as f32;

    events
        .record_signal(
            address,
            &format!(
                "reformation: {} failure(s) triggered content rewrite (mean quality: {:.3})",
                failures.len(),
                mean_failure_quality
            ),
            mean_failure_quality,
        )
        .map_err(|e| e.to_string())?;

    tracing::info!(
        "reformed identity {address}: {} failure(s), mean quality {mean_failure_quality:.3}",
        failures.len()
    );

    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_events::{EventLog, EventRecord, RecordType};

    fn ts() -> u64 { 1_700_000_000_000 }

    fn make_event(identity: &str, quality: Option<f32>, packet_id: u64, offset_ms: u64) -> EventRecord {
        EventRecord {
            record_type:    RecordType::Event,
            timestamp_ms:   ts() + offset_ms,
            chain_id:       packet_id,
            packet_id,
            hop_count:      1,
            identity:       identity.to_string(),
            output:         format!("output for event {packet_id}"),
            world_position: None,
            quality,
        }
    }

    const PHIL: &str = "dworld://council.local/identities/PHILOSOPHER";
    const ENG:  &str = "dworld://council.local/identities/ENGINEER";
    const SYN:  &str = "dworld://council.local/identities/SYNTHESIZER";

    #[test]
    fn candidates_returns_address_when_window_all_below_threshold() {
        let log = EventLog::open_in_memory().unwrap();
        let detector = DriftDetector::new(3, 0.5);

        for i in 0..3u64 {
            log.append(&make_event(PHIL, Some(0.2), i, i * 1000)).unwrap();
        }

        let candidates = detector.candidates(&log);
        assert!(
            candidates.contains(&PHIL.to_string()),
            "expected PHILOSOPHER as candidate, got: {candidates:?}"
        );
    }

    #[test]
    fn candidates_empty_when_quality_above_threshold() {
        let log = EventLog::open_in_memory().unwrap();
        let detector = DriftDetector::new(3, 0.5);

        for i in 0..3u64 {
            log.append(&make_event(ENG, Some(0.8), i, i * 1000)).unwrap();
        }

        let candidates = detector.candidates(&log);
        assert!(
            !candidates.contains(&ENG.to_string()),
            "ENGINEER should not be a candidate when quality is high"
        );
    }

    #[test]
    fn candidates_empty_when_window_not_fully_scored() {
        let log = EventLog::open_in_memory().unwrap();
        let detector = DriftDetector::new(5, 0.5);

        // Only 3 scored events — window requires 5.
        for i in 0..3u64 {
            log.append(&make_event(SYN, Some(0.1), i, i * 1000)).unwrap();
        }

        let candidates = detector.candidates(&log);
        assert!(
            candidates.is_empty(),
            "should need a full window of {}: scored events before flagging",
            detector.window
        );
    }

    #[test]
    fn candidates_ignores_unscored_events() {
        let log = EventLog::open_in_memory().unwrap();
        let detector = DriftDetector::new(2, 0.5);

        for i in 0..3u64 {
            // quality = None — not yet scored
            log.append(&make_event(PHIL, None, i, i * 1000)).unwrap();
        }

        let candidates = detector.candidates(&log);
        assert!(
            candidates.is_empty(),
            "unscored events must not trigger drift detection"
        );
    }

    #[test]
    fn candidates_mixed_quality_does_not_trigger() {
        let log = EventLog::open_in_memory().unwrap();
        let detector = DriftDetector::new(3, 0.5);

        // Two poor + one good: window not all-below-threshold.
        log.append(&make_event(ENG, Some(0.2), 1, 0)).unwrap();
        log.append(&make_event(ENG, Some(0.2), 2, 1000)).unwrap();
        log.append(&make_event(ENG, Some(0.9), 3, 2000)).unwrap();

        let candidates = detector.candidates(&log);
        assert!(
            !candidates.contains(&ENG.to_string()),
            "mixed quality should not trigger reformation"
        );
    }

    #[test]
    fn recent_events_for_identity_returns_correct_events() {
        let log = EventLog::open_in_memory().unwrap();

        for i in 0..5u64 {
            log.append(&make_event(SYN, Some(0.7), i, i * 1000)).unwrap();
        }
        // Different identity — must not appear in results.
        log.append(&make_event(ENG, Some(0.9), 99, 99_000)).unwrap();

        let events = log.recent_events_for_identity(SYN, 3).unwrap();

        assert_eq!(events.len(), 3, "should return exactly 3 events");
        for r in &events {
            assert_eq!(r.identity, SYN, "wrong identity in result");
        }
    }

    #[test]
    fn record_signal_is_retrievable() {
        let log = EventLog::open_in_memory().unwrap();
        log.record_signal(PHIL, "reformation: 2 failure(s)", 0.3).unwrap();

        let signals: Vec<_> = log.recent(5).unwrap()
            .into_iter()
            .filter(|r| r.record_type == RecordType::Signal)
            .collect();

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].identity, PHIL);
        assert!(signals[0].output.contains("reformation"));
        let q = signals[0].quality.unwrap();
        assert!((q - 0.3).abs() < 1e-4, "quality={q}");
    }

    #[test]
    #[ignore = "requires LLM — reform_identity makes embed + complete calls"]
    fn reform_identity_rewrites_store_and_moves_position() {
        // Integration test: build store, inject failures, call reform_identity,
        // verify the identity's world_coord changed and a SIGNAL appears in the log.
    }
}
