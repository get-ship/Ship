//! Public test utilities for downstream crates.
//!
//! [`MockPersist`] is the canonical in-memory [`EventPersist`] for tests that
//! want to assert a route/persist produced an [`EventEnvelope`] without
//! standing up a SQLite store. Always compiled so test code in downstream
//! crates can import it directly:
//!
//! ```ignore
//! use ship_kernel::testing::MockPersist;
//! ```
//!
//! The kernel's own `#[cfg(test)]` block re-uses this same struct rather than
//! maintaining a parallel copy.
//!
//! `MockSink` is a compatibility alias for `MockPersist`.

use parking_lot::Mutex;
use ship_types::envelope::EventEnvelope;
use ship_types::Stamped;

use crate::EventPersist;

/// In-memory [`EventPersist`] that captures every persisted envelope in a vector.
/// Tests inspect the captured events with [`MockPersist::captured`].
#[derive(Default)]
pub struct MockPersist {
    events: Mutex<Vec<EventEnvelope>>,
}

/// Compatibility alias — prefer [`MockPersist`].
pub type MockSink = MockPersist;

impl MockPersist {
    /// Create an empty `MockPersist`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of every event captured so far (in insertion order).
    pub fn captured(&self) -> Vec<EventEnvelope> {
        self.events.lock().clone()
    }
}

impl EventPersist for MockPersist {
    fn persist(&self, event: &Stamped) -> anyhow::Result<()> {
        self.events.lock().push(event.as_envelope().clone());
        Ok(())
    }
}
