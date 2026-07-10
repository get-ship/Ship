//! Event-id deduplication primitive for runner gateways.
//!
//! Returned by `runner_entry_gate` (etc.) so the domain owner of the dedup
//! state — the orchestrator / agents actor / tests — can hand a single
//! `Arc<EventDedup>` into its event-handling loop and avoid re-advancing
//! state for the same envelope across mailbox redelivery / replay races.
//!
//! Originally part of `ship-code::runner::dedup`. Re-homed to L0
//! `ship_substrate::dispatch_gate` so both the orchestrator
//! (`ship-pipeline-runtime`) and the agent-session actor (`ship-agents`)
//! can depend on it without a cargo cycle. The shape is unchanged.

use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

const DEFAULT_CAPACITY: usize = 4096;

pub struct EventDedup {
    inner: Mutex<Inner>,
}

struct Inner {
    seen: HashSet<String>,
    order: VecDeque<String>,
    capacity: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Seen {
    Fresh,
    Duplicate,
}

impl EventDedup {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(Inner {
                seen: HashSet::with_capacity(capacity),
                order: VecDeque::with_capacity(capacity),
                capacity,
            }),
        }
    }

    /// Record `id`. Returns [`Seen::Duplicate`] if the same id was seen
    /// within the recent window; otherwise [`Seen::Fresh`] and the id now
    /// lives in the set. Evicts the oldest id when the set grows past
    /// capacity (sliding window — older envelopes that haven't repeated
    /// are no longer interesting).
    pub fn record(&self, id: &str) -> Seen {
        let mut inner = self.inner.lock().expect("EventDedup mutex poisoned");
        if !inner.seen.insert(id.to_string()) {
            return Seen::Duplicate;
        }
        inner.order.push_back(id.to_string());
        while inner.order.len() > inner.capacity {
            if let Some(oldest) = inner.order.pop_front() {
                inner.seen.remove(&oldest);
            }
        }
        Seen::Fresh
    }
}

impl Default for EventDedup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_record_is_fresh() {
        let d = EventDedup::new();
        assert_eq!(d.record("a"), Seen::Fresh);
    }

    #[test]
    fn second_record_is_duplicate() {
        let d = EventDedup::new();
        assert_eq!(d.record("a"), Seen::Fresh);
        assert_eq!(d.record("a"), Seen::Duplicate);
    }

    #[test]
    fn distinct_ids_are_fresh() {
        let d = EventDedup::new();
        assert_eq!(d.record("a"), Seen::Fresh);
        assert_eq!(d.record("b"), Seen::Fresh);
        assert_eq!(d.record("c"), Seen::Fresh);
    }

    #[test]
    fn evicts_oldest_at_capacity() {
        let d = EventDedup::with_capacity(2);
        assert_eq!(d.record("a"), Seen::Fresh);
        assert_eq!(d.record("b"), Seen::Fresh);
        // Window full {a, b}; adding c evicts a, leaving {b, c}.
        assert_eq!(d.record("c"), Seen::Fresh);
        assert_eq!(d.record("c"), Seen::Duplicate, "c is in the window");
        assert_eq!(d.record("a"), Seen::Fresh, "a was evicted, now fresh");
    }

    #[test]
    fn concurrent_same_id_deduplicates() {
        use std::sync::Arc;
        use std::thread;

        let d = Arc::new(EventDedup::with_capacity(8));
        let mut handles = vec![];
        for _ in 0..4 {
            let d = Arc::clone(&d);
            handles.push(thread::spawn(move || {
                (0..1000).map(|_| d.record("x")).collect::<Vec<_>>()
            }));
        }
        let mut all_fresh = 0;
        for h in handles {
            for s in h.join().unwrap() {
                if s == Seen::Fresh {
                    all_fresh += 1;
                }
            }
        }
        // Exactly one thread wins `Fresh` for "x"; the rest see Duplicate.
        assert_eq!(all_fresh, 1);
    }
}
