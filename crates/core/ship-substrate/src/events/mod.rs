// Re-exports from ship-types
pub use ship_types::envelope::EventEnvelope;
pub use ship_types::payload::*;
pub use ship_types::{CallerKind, EmitContext, ActorConfig};
pub mod types {
    pub use ship_types::payload::*;
    pub mod event_types {
        pub use ship_types::event_types::*;
    }

    #[cfg(test)]
    mod tests {
        use super::event_types::*;

        #[test]
        fn event_type_constants_are_unique() {
            let mut seen = std::collections::HashSet::new();
            for &t in ALL {
                assert!(seen.insert(t), "duplicate event type constant: {t}");
            }
        }

        #[test]
        fn all_constants_have_expected_count() {
            // F1 (substrate-cleanup slice 01) deleted 12 unused constants
            // (9 workspace.*, job.claimed, executor.session_created,
            // executor.window_created). event-coverage/01 added 7
            // (skill.{created,updated,deleted}, rule.{created,updated,
            // deleted}, permissions.updated). event-coverage/02 added 2 (session.status_changed, session.turn_failed) for 82 total.
            // project-switch-wiring added project.switched for 84 total.
            // Drift to 88: 4 new constants since this counter was last
            // updated (verify via grep before bumping further).
            // 89: #85a added agent.grants.provisioned (per-agent grant baseline).
            // Adjust this when adding/removing event types.
            assert_eq!(ALL.len(), 89, "exactly 89 event type constants required");
        }
    }
}

// Re-exports from ship-kernel
pub use ship_kernel::EventSink;
pub use ship_kernel::KernelError as PureKernelError;
pub use ship_kernel::Mailbox;

// Persistence (re-exported from crate::persistence)
pub use crate::persistence::{
    ActorStore, EventStore, SqliteEventStore, ActorSnapshot,
};
// Backward-compatible module aliases for consumers.
pub use crate::persistence::event_store as store;
// Modules kept here (implement runtime-specific logic)
pub mod agent_session;
pub mod annotation;
pub mod daemon;
pub mod executor;
pub mod hooks;
pub mod feedback;
pub mod filter;
pub mod global_kernel_router;
pub mod pipeline;
pub mod process;
pub mod supervisor;
pub mod kernel_lifecycle;
pub mod kernel_router;
pub mod payload_schema;
pub mod validator;

pub use feedback::{
    AuthorKind, FeedbackEvent, FeedbackIntent, FeedbackKind, FeedbackTarget, Selection,
};

#[cfg(feature = "unstable")]
pub mod permissions;
#[cfg(feature = "unstable")]
pub mod kernel_security;

pub use filter::EventFilter;
pub use kernel_router::KernelRouter;
pub use payload_schema::PayloadSchemaValidator;
pub use validator::{
    ElevatedValidator, EventValidator, KERNEL_NAMESPACES,
    NamespaceValidator, PLATFORM_ONLY_NAMESPACES, RESERVED_NAMESPACES,
    ReservedNamespaceValidator, ValidationError, is_kernel_event,
};
pub use global_kernel_router::{init_kernel_router, kernel_router, new_isolated_for_test};

// Test modules
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod kernel_router_tests;
#[cfg(test)]
mod cross_actor_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod cli_routing_tests;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod permission_tests;
#[cfg(test)]
mod scoped_delivery_tests;
#[cfg(test)]
mod directed_delivery_tests;
#[cfg(test)]
mod cursor_tests;
#[cfg(test)]
mod validator_tests;
#[cfg(test)]
mod kernel_security_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod snapshot_edge_tests;
#[cfg(test)]
mod tests_mesh_scope;

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
pub fn query_events_since(
    cursor: Option<&str>,
    elevated_only: bool,
) -> Result<Vec<EventEnvelope>> {
    crate::db::events::query_events_since(cursor, elevated_only)
}
