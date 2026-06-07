//! Event-id deduplication for the runner router.
//!
//! Prevents a single event from advancing the pipeline twice when the
//! same `EventEnvelope` reaches the router more than once (mailbox
//! redelivery, startup replay racing with live delivery, test harness
//! replays).
//!
//! Bounded in-memory LRU. On daemon restart the cache is empty, so any
//! previously-seen ids look fresh — the reconcile/replay path is what
//! handles post-restart idempotency, not this module.

use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

/// Default cap: 4096 distinct event ids. Events average ~80B of id key,
/// ~320 KiB total at cap — negligible.
const DEFAULT_CAPACITY: usize = 4096;

/// Bounded event-id set with oldest-insertion eviction.
pub struct EventDedup {
    inner: Mutex<Inner>,
}

struct Inner {
    seen: HashSet<String>,
    order: VecDeque<String>,
    capacity: usize,
}

/// Outcome of a dedup check.
#[derive(Debug, PartialEq, Eq)]
pub enum Seen {
    /// First time we've seen this id — caller should process the event.
    Fresh,
    /// Duplicate — caller should skip routing.
    Duplicate,
}

impl EventDedup {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(Inner {
                seen: HashSet::with_capacity(capacity.min(1024)),
                order: VecDeque::with_capacity(capacity.min(1024)),
                capacity,
            }),
        }
    }

    /// Record an event id. Returns `Fresh` on first observation, `Duplicate`
    /// on repeats. Oldest ids are evicted once `capacity` is exceeded.
    pub fn record(&self, id: &str) -> Seen {
        let mut inner = self.inner.lock().expect("EventDedup poisoned");
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
        assert_eq!(d.record("evt-1"), Seen::Fresh);
    }

    #[test]
    fn second_record_is_duplicate() {
        let d = EventDedup::new();
        d.record("evt-1");
        assert_eq!(d.record("evt-1"), Seen::Duplicate);
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
        // Window still contains {a, b}: repeats are duplicates.
        assert_eq!(d.record("a"), Seen::Duplicate);
        assert_eq!(d.record("b"), Seen::Duplicate);
        // Push a third distinct id — evicts "a" as the oldest.
        assert_eq!(d.record("c"), Seen::Fresh);
        assert_eq!(d.record("a"), Seen::Fresh, "evicted id should re-enter fresh");
        assert_eq!(d.record("c"), Seen::Duplicate, "c is still inside the window");
    }

    #[test]
    fn concurrent_same_id_deduplicates() {
        use std::sync::Arc;
        use std::thread;

        let d = Arc::new(EventDedup::new());
        let handles: Vec<_> = (0..32)
            .map(|_| {
                let d = d.clone();
                thread::spawn(move || d.record("shared"))
            })
            .collect();

        let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let fresh_count = outcomes.iter().filter(|o| **o == Seen::Fresh).count();
        assert_eq!(fresh_count, 1, "exactly one thread should see Fresh");
    }
}
