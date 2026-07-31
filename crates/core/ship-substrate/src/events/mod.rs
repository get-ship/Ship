// Re-exports from ship-types
pub use ship_types::envelope::EventEnvelope;
pub use ship_types::event_registry::EventScope;
pub use ship_types::payload::*;
pub use ship_types::{ActorConfig, CallerKind, EmitContext};
pub mod types {
    pub use ship_types::payload::*;
    pub mod event_types {
        pub use ship_types::event_types::*;
    }

    // The former const-uniqueness/count tests over the hand-maintained `ALL`
    // slice are gone with `ALL` itself: registry-name uniqueness is asserted in
    // ship-types (`registry_names_are_unique`), the L0 registration count is
    // ratcheted in `events::tests::l0_registry_shrinks_under_keystone_a`, and
    // the full-linkage vocabulary pin lives in shipd `tests/event_vocabulary.rs`.
}

// Re-exports from ship-kernel
pub use ship_kernel::EventPersist;
pub use ship_kernel::KernelError as PureKernelError;
pub use ship_kernel::Mailbox;
// Type-state proof for stamped envelopes (persistence layer)
pub use ship_types::Stamped;

// Persistence (re-exported from crate::persistence)
pub use crate::persistence::{EventStore, SqliteEventStore};
// Backward-compatible module alias for the canonical event store.
pub use crate::persistence::event_store as store;
// Modules kept here (implement runtime-specific logic)
// annotation.rs + feedback.rs moved to ship-agents (keystone-A de-bucket,
// 2026-06-10): the STUDIO domain owns its payload types at L1.
// payload_registry.rs + pipeline/* live here too — they depend on
// `ship-pipelines` types only, which is itself a clean crate (no
// object_store / notify / portable-pty), so the carve invariant holds
// without per-facade shims.
pub mod daemon;
pub mod emit;
pub mod executor;
pub mod filter;
pub mod kernel_lifecycle;
pub mod kernel_router;
pub mod payload_registry;
pub mod payload_schema;
pub mod pipeline;
pub mod process;
pub mod validator;

pub use filter::EventFilter;
pub use kernel_router::{
    KernelRouter, SharedKernelRouter, init_kernel_router, init_kernel_router_with,
    new_isolated_for_test, new_isolated_for_test_with,
};
pub use payload_schema::{
    PayloadSchemaValidator, StampError, apply_stamp_chokepoint, apply_stamp_chokepoint_at,
    off_kernel_ctx, validate_scope_registered,
};
pub use validator::{
    ElevatedValidator, EventValidator, KERNEL_NAMESPACES, NamespaceValidator,
    PLATFORM_ONLY_NAMESPACES, RESERVED_NAMESPACES, ReservedNamespaceValidator, ValidationError,
    is_kernel_event,
};

// Test modules
#[cfg(test)]
mod cli_routing_tests;
#[cfg(test)]
mod cross_actor_tests;
#[cfg(test)]
mod kernel_router_enforce_tests;
#[cfg(test)]
mod kernel_router_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod validator_tests;

use anyhow::Result;
use chrono::{DateTime, Utc};
pub fn read_events() -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_all_events()
}

pub fn list_events_since(
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_events_since_time(since, limit)
}

pub fn read_recent_events(limit: usize) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_recent_events(limit)
}

/// Query events with ID greater than the given cursor.
///
/// Used by the sync client to find events that haven't been pushed yet.
/// If `elevated_only` is true, only returns elevated events (platform-scope).
pub fn query_events_since(cursor: Option<&str>, elevated_only: bool) -> Result<Vec<EventEnvelope>> {
    crate::db::events::query_events_since(cursor, elevated_only)
}
