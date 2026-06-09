//! Event-id deduplication for the runner router.
//! Moved from ship-runner (L0) to ship-code (L1).
//!
//! Prevents a single event from advancing the pipeline twice when the
//! same `EventEnvelope` reaches the router more than once.

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
                seen: HashSet::with_capacity(capacity.min(1024)),
                order: VecDeque::with_capacity(capacity.min(1024)),
                capacity,
            }),
        }
    }

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
        assert_eq!(d.record("a"), Seen::Duplicate);
        assert_eq!(d.record("b"), Seen::Duplicate);
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
