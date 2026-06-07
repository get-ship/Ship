//! EventSink — persistence abstraction for the kernel router.
//!
//! The kernel routes events but does not own persistence. Callers provide
//! a sink implementation (e.g. SQLite, in-memory mock) at construction time.

use ship_types::envelope::EventEnvelope;

/// Persistence interface for routed events.
///
/// Implementations handle where and how events are stored. The kernel calls
/// `persist` synchronously during `route` — before mailbox delivery — so the
/// event is durable before any actor sees it.
pub trait EventSink: Send + Sync {
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()>;
}
