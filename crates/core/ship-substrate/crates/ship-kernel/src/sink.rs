//! EventPersist — persistence abstraction for the kernel router.
//!
//! The kernel routes events but does not own persistence. Callers provide
//! a persist implementation (e.g. SQLite, in-memory mock) at construction time.
//!
//! Named `EventPersist` (sync `persist`) to keep it distinct from the async
//! emission port `ship_types::EventSink` (`async fn emit`).

use ship_types::Stamped;

/// Persistence interface for routed events.
///
/// Implementations handle where and how events are stored. The kernel calls
/// `persist` synchronously during `route` — before mailbox delivery — so the
/// event is durable before any actor sees it.
///
/// Takes [`Stamped`]: only envelopes that have passed the stamp chokepoint
/// (or a `test-support` unchecked mint) can be persisted.
pub trait EventPersist: Send + Sync {
    fn persist(&self, event: &Stamped) -> anyhow::Result<()>;
}
