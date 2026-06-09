// Re-exports from ship-types
pub use ship_types::envelope::EventEnvelope;
pub use ship_types::event_registry::EventScope;
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
        use ship_types::event_registry::{self, EventScope, Retention};

        #[test]
        fn event_type_constants_are_unique() {
            let mut seen = std::collections::HashSet::new();
            for &t in ALL {
                assert!(seen.insert(t), "duplicate event type constant: {t}");
            }
        }

        /// EVENT-SCOPE DISCIPLINE — invariant test over the live registry.
        ///
        /// Rules (from the canonical-session-events ADR):
        ///   • `session.*` (but NOT `session.turn.*`) → primary scope = Space,
        ///     retention = Durable. These are coarse-grain session lifecycle events
        ///     that belong to the Space's replayed history.
        ///   • `session.turn.*` → primary scope = Runtime, retention = Ephemeral.
        ///     Turns churn too fast for full history; only the last N are retained.
        ///   • `op.*` (if any) → primary scope = Runtime, retention = Ephemeral.
        ///     Fine-grained op events ride the hook stream; not durable.
        ///
        /// This generalises the slice-2 payload guard: rather than checking struct
        /// fields, we check the registry metadata directly. Adding a new `session.*`
        /// event with wrong scope/retention will fail here immediately.
        #[test]
        fn event_scope_discipline() {
            let mut violations: Vec<String> = Vec::new();

            for def in event_registry::registry() {
                let name = def.name;
                let primary = def.primary_scope();
                let retention = def.retention;

                // session.* but NOT session.turn.* → Space + Durable
                if name.starts_with("session.") && !name.starts_with("session.turn.") {
                    if primary != EventScope::Space {
                        violations.push(format!(
                            "event `{name}`: expected primary scope Space (got {:?}); \
                             coarse session events must be Space-scoped so the Space \
                             history can replay them",
                            primary
                        ));
                    }
                    if retention != Retention::Durable {
                        violations.push(format!(
                            "event `{name}`: expected retention Durable (got {:?}); \
                             coarse session events must be Durable so they survive \
                             Space restoration",
                            retention
                        ));
                    }
                }

                // session.turn.* → Runtime + Ephemeral
                if name.starts_with("session.turn.") {
                    if primary != EventScope::Runtime {
                        violations.push(format!(
                            "event `{name}`: expected primary scope Runtime (got {:?}); \
                             turn events are per-machine execution state",
                            primary
                        ));
                    }
                    if retention != Retention::Ephemeral {
                        violations.push(format!(
                            "event `{name}`: expected retention Ephemeral (got {:?}); \
                             turns churn too fast for full history",
                            retention
                        ));
                    }
                }

                // op.* → Runtime + Ephemeral
                if name.starts_with("op.") {
                    if primary != EventScope::Runtime {
                        violations.push(format!(
                            "event `{name}`: expected primary scope Runtime (got {:?}); \
                             op events ride the hook stream and are Runtime-owned",
                            primary
                        ));
                    }
                    if retention != Retention::Ephemeral {
                        violations.push(format!(
                            "event `{name}`: expected retention Ephemeral (got {:?}); \
                             op events are fine-grained and not retained in full history",
                            retention
                        ));
                    }
                }
            }

            assert!(
                violations.is_empty(),
                "event-scope discipline violations ({} total):\n  {}",
                violations.len(),
                violations.join("\n  ")
            );
        }
        #[test]
        fn all_constants_have_expected_count() {
            // Spec 7a bumped from 88 → 111 by adding the 23 Spec-6 events to ALL.
            // S3 adds `settings.changed` (the unified scope-discriminated keystone)
            // → 113. S1 (agents-arc) adds `agent.def.updated` (the Format-2 def
            // change notification) → 114. agents-arc E6 GUTS `studio.message`
            // (the deprecated chat read-projection event, sole consumer was chat)
            // → 113. Job `dead-events-cull` removes `project.changed` and
            // `skill.attached_to_session` (zero producers, no consumers) → 111.
            // Job `canonical-session-events` adds the 6 canonical agent-session
            // events (session.completed, session.failed, session.cancelled,
            // session.suspended, session.turn.started, session.turn.ended) → 117.
            // Job `workbench-teardown` retires the 11 workbench session.* events
            // AND adds the canonical `session.started` (the workbench string the
            // canonical family deferred on) → 117 - 11 + 1 = 107.
            // The hard count is enforced here to catch accidental removals.
            // When adding events: update ALL in event_types.rs AND payload_registry.rs.
            assert_eq!(ALL.len(), 107, "exactly 107 event type constants required (workbench-teardown: -11 workbench + 1 canonical session.started)");
        }

        #[test]
        fn all_entries_have_registry_entries() {

            use ship_types::event_registry;
            let registry_names: std::collections::HashSet<&str> =
                event_registry::registry().map(|d| d.name).collect();
            let missing: Vec<&str> = ALL
                .iter()
                .copied()
                .filter(|name| !registry_names.contains(name))
                .collect();
            assert!(
                missing.is_empty(),
                "event types in ALL not registered in event_registry: {missing:?}\n\
                 Add entries to ship_substrate::events::payload_registry"
            );
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
pub mod canonical_session;
pub mod daemon;
pub mod executor;
pub mod hooks;
pub mod feedback;
pub mod filter;
pub mod global_kernel_router;
pub mod payload_registry;
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
mod kernel_router_enforce_tests;
#[cfg(test)]
mod cross_actor_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod cli_routing_tests;
#[cfg(test)]
mod validator_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod snapshot_edge_tests;

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
