//! Public test utilities for downstream crates.
//!
//! [`MockSink`] is the canonical in-memory [`EventSink`] for tests that want
//! to assert a route/persist produced an [`EventEnvelope`] without standing
//! up a SQLite store. Always compiled so test code in downstream crates can
//! import it directly:
//!
//! ```ignore
//! use ship_kernel::testing::MockSink;
//! ```
//!
//! The kernel's own `#[cfg(test)]` block re-uses this same struct rather than
//! maintaining a parallel copy.

use parking_lot::Mutex;
use ship_types::envelope::EventEnvelope;

use crate::EventSink;

/// In-memory [`EventSink`] that captures every persisted envelope in a vector.
/// Tests inspect the captured events with [`MockSink::captured`].
#[derive(Default)]
pub struct MockSink {
    events: Mutex<Vec<EventEnvelope>>,
}

impl MockSink {
    /// Create an empty `MockSink`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of every event captured so far (in insertion order).
    pub fn captured(&self) -> Vec<EventEnvelope> {
        self.events.lock().clone()
    }
}

impl EventSink for MockSink {
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        self.events.lock().push(event.clone());
        Ok(())
    }
}
