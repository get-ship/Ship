//! `ReplayLog` — the kernel's bounded in-memory ring of recently-emitted
//! events, used by the `unstable` cursor-replay path to redeliver events a
//! permissioned actor missed while it was stopped.
//!
//! Bounded on TWO axes so retained RSS is capped regardless of uptime OR
//! payload size — see [`ReplayLog`] for the leak history behind that.

use ship_types::envelope::EventEnvelope;

/// Bounded in-memory ring of recently-emitted events.
///
/// Originally a plain `Vec` that grew on EVERY `emit` for the life of the
/// daemon and was never pruned — an unbounded leak whose growth tracked the
/// 30s coherence-sweep cadence (every sweep emits a handful of events that
/// landed here forever) and drove shipd RSS up without limit. A first fix
/// bounded it to `CAP` envelopes by COUNT.
///
/// But a count cap does NOT bound BYTES: an `EventEnvelope`'s `payload_json`
/// is unbounded (a `tool_use` input or `tool_result` echoing a large file is
/// hundreds of KB), so 10k retained envelopes can be multiple GB. During a
/// fanout — agents firing a torrent of hook / tool / studio events with fat
/// payloads — the log fills toward 10k and retained RSS climbs roughly
/// linearly until the count cap is reached, which reads as the #193 leak.
/// This is DOWNSTREAM of (and unrelated to) the `LineAccumulator`
/// line-buffer fix: that bounds a single harness reader's un-terminated
/// line; this bounds the kernel's cross-actor replay ring. So the log is now
/// bounded on BOTH axes — count AND a byte budget — evicting oldest until
/// under both.
///
/// Cursors are LOGICAL positions — counts of total events ever appended, not
/// physical indices. `base` tracks how many events were dropped off the
/// front so reads translate logical → physical. A cursor older than what
/// remains in memory replays from the oldest retained event; the rest are
/// durable in the SQLite event store, so this is bounded best-effort
/// redelivery, not data loss.
#[derive(Default)]
pub(crate) struct ReplayLog {
    events: std::collections::VecDeque<EventEnvelope>,
    /// Events dropped off the front. Logical positions `0..base` are no
    /// longer retained in memory.
    base: usize,
    /// Running sum of `env_bytes` for every retained envelope — the byte
    /// budget is enforced against this without rescanning the deque.
    bytes: usize,
}

/// Approximate heap footprint of a retained envelope. `payload_json`
/// dominates; the rest is the handful of short id/type strings plus a fixed
/// slop for the struct itself and the deque slot. Exact-to-the-byte isn't
/// needed — this drives eviction, not accounting.
fn env_bytes(e: &EventEnvelope) -> usize {
    e.payload_json.len()
        + e.id.len()
        + e.event_type.len()
        + e.entity_id.len()
        + e.actor.len()
        + 128
}

impl ReplayLog {
    /// Max retained envelopes. ~10k covers the restart-replay window with
    /// room to spare while keeping in-memory event retention bounded.
    const CAP: usize = 10_000;

    /// Max retained bytes across all envelopes. 64 MiB is generous for the
    /// restart-replay window yet caps RSS even when payloads are fat (large
    /// `tool_result` / `tool_use` events). Whichever bound (count or bytes)
    /// binds first governs eviction.
    const BYTE_CAP: usize = 64 * 1024 * 1024;

    /// Append an event, evicting the oldest while over EITHER the count cap
    /// or the byte budget. Always retains at least the just-appended event
    /// (a lone envelope larger than `BYTE_CAP` is kept rather than dropped
    /// into emptiness).
    pub(crate) fn append(&mut self, event: EventEnvelope) {
        self.bytes += env_bytes(&event);
        self.events.push_back(event);
        while self.events.len() > Self::CAP
            || (self.bytes > Self::BYTE_CAP && self.events.len() > 1)
        {
            if let Some(old) = self.events.pop_front() {
                self.bytes = self.bytes.saturating_sub(env_bytes(&old));
                self.base += 1;
            }
        }
    }

    /// Number of envelopes currently retained in memory.
    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }

    /// Bytes currently retained across all envelopes. Test/observability.
    pub(crate) fn retained_bytes(&self) -> usize {
        self.bytes
    }

    /// Logical length = total events ever appended (including pruned). This
    /// is the value stored as an actor's replay cursor.
    pub(crate) fn logical_len(&self) -> usize {
        self.base + self.events.len()
    }

    /// Retained events at logical positions `>= from`, oldest first. A
    /// cursor below `base` clamps to the oldest retained event.
    pub(crate) fn since(&self, from: usize) -> impl Iterator<Item = &EventEnvelope> {
        let start = from.saturating_sub(self.base);
        self.events.iter().skip(start)
    }
}

#[cfg(test)]
mod tests {
    use super::ReplayLog;
    use ship_types::envelope::EventEnvelope;

    fn ev() -> EventEnvelope {
        EventEnvelope::new("studio.test", "e", &serde_json::json!({})).unwrap()
    }

    #[test]
    fn append_is_bounded_but_logical_len_keeps_growing() {
        let mut log = ReplayLog::default();
        let total = ReplayLog::CAP + 5;
        for _ in 0..total {
            log.append(ev());
        }
        // Physical retention is capped...
        assert_eq!(log.len(), ReplayLog::CAP, "retained events capped at CAP");
        assert_eq!(log.base, 5, "the 5 oldest events were pruned off the front");
        // ...but the logical cursor space keeps counting every append, so a
        // cursor handed out before pruning stays meaningful.
        assert_eq!(log.logical_len(), total);
    }

    #[test]
    fn since_translates_logical_positions_across_pruning() {
        let mut log = ReplayLog::default();
        let total = ReplayLog::CAP + 5;
        for _ in 0..total {
            log.append(ev());
        }
        // A cursor at the very latest logical position replays nothing.
        assert_eq!(log.since(total).count(), 0);
        // One before the end replays exactly one event.
        assert_eq!(log.since(total - 1).count(), 1);
        // A cursor older than what we retain clamps to the oldest retained
        // event — never panics, never double-counts.
        assert_eq!(log.since(0).count(), ReplayLog::CAP);
        assert_eq!(log.since(log.base).count(), ReplayLog::CAP);
    }

    #[test]
    fn no_pruning_below_cap_keeps_exact_positions() {
        let mut log = ReplayLog::default();
        for _ in 0..10 {
            log.append(ev());
        }
        assert_eq!(log.base, 0);
        assert_eq!(log.logical_len(), 10);
        assert_eq!(log.since(3).count(), 7);
    }

    fn fat_ev(kib: usize) -> EventEnvelope {
        // A `tool_result`-shaped event whose payload echoes a large file —
        // the realistic fat-payload case that the count cap alone fails to
        // bound. ~`kib` KiB of payload.
        let blob = "x".repeat(kib * 1024);
        EventEnvelope::new("hook.tool_use.completed", "agent-lane", &serde_json::json!({ "out": blob }))
            .unwrap()
    }

    // leak-193 regression: the count cap (10k envelopes) does NOT bound
    // bytes. Append events whose count stays well under `CAP` but whose
    // total payload far exceeds `BYTE_CAP`. Before the byte budget, every
    // one is retained and `retained_bytes` blows past the cap (this is the
    // #193 RSS climb). After it, eviction holds the byte footprint bounded.
    #[test]
    fn append_bounds_retained_bytes_not_just_count() {
        let mut log = ReplayLog::default();
        let payload_kib = 128;
        let per_event = (payload_kib * 1024) + 128; // matches env_bytes slop
        // Enough fat events to exceed BYTE_CAP by ~20%, but far under CAP.
        let target = ReplayLog::BYTE_CAP + ReplayLog::BYTE_CAP / 5;
        let count = target / per_event;
        assert!(count < ReplayLog::CAP, "test stays under the count cap: {count}");
        for _ in 0..count {
            log.append(fat_ev(payload_kib));
        }
        // Byte footprint stays bounded by the budget (+ at most one event).
        assert!(
            log.retained_bytes() <= ReplayLog::BYTE_CAP + per_event,
            "retained bytes must respect the byte budget: {} > {}",
            log.retained_bytes(),
            ReplayLog::BYTE_CAP + per_event,
        );
        // Eviction happened by BYTES while still below the count cap — the
        // behaviour the count-only cap lacked.
        assert!(log.base > 0, "oldest fat events were evicted on the byte budget");
        assert!(log.len() < count, "not every appended event is retained");
        // Logical cursor space still counts every append (replay cursors stay valid).
        assert_eq!(log.logical_len(), count);
    }

    // A single envelope larger than the whole budget is retained rather than
    // evicted into emptiness — replay of the latest event still works.
    #[test]
    fn append_keeps_one_oversized_event() {
        let mut log = ReplayLog::default();
        log.append(ev());
        log.append(fat_ev(ReplayLog::BYTE_CAP / 1024 + 1024)); // > BYTE_CAP
        assert_eq!(log.len(), 1, "the small event was evicted, the big one kept");
        assert!(log.retained_bytes() > ReplayLog::BYTE_CAP);
        assert_eq!(log.logical_len(), 2);
    }
}
