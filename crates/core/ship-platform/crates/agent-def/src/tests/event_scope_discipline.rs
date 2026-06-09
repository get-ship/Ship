//! Event scope discipline tests for the canonical session family.
//!
//! These tests verify that the canonical session events registered in
//! agent-def conform to the scope and retention rules defined in the
//! canonical-session-events ADR.

#[cfg(test)]
mod tests {
    use ship_types::{event_types::ALL, event_registry, event_registry::{EventScope, Retention}};

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
    fn all_entries_have_registry_entries() {
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
             Add entries to agent-def::event_registrations or ship_substrate::events::payload_registry"
        );
    }
}
