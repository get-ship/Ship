//! Event scope discipline tests for the canonical session + op families.
//!
//! These tests verify that the canonical events registered in
//! agent-def conform to the scope and retention rules defined in
//! the canonical-session-events ADR (plus the op.* R2 addendum
//! in op.* family work).

#[cfg(test)]
mod tests {
    use ship_types::{
        event_types::canonical_session::{ALL_CANONICAL_OP, ALL_CANONICAL_SESSION},
        event_registry,
        event_registry::{EventScope, Retention},
    };

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

            // dispatch.* → Runtime + Ephemeral
            // The spawn-intent channel (#26). Runtime-scoped because it drives
            // machine-local process spawn; Ephemeral because the intent is
            // fulfilled immediately (the harness is up or the dispatch failed).
            if name.starts_with("dispatch.") {
                if primary != EventScope::Runtime {
                    violations.push(format!(
                        "event `{name}`: expected primary scope Runtime (got {:?}); \
                         dispatch events drive machine-local harness spawning",
                        primary
                    ));
                }
                if retention != Retention::Ephemeral {
                    violations.push(format!(
                        "event `{name}`: expected retention Ephemeral (got {:?}); \
                         dispatch intent is fulfilled immediately — no durable replay needed",
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

    /// Every name in `ALL_CANONICAL_SESSION` and `ALL_CANONICAL_OP`
    /// is registered. Pairs with `event_scope_discipline` — the
    /// const-set consistency test that catches dropped register
    /// calls. The master `ALL` slice is retired
    /// (the registry is the vocabulary SSOT); the domain-owned
    /// ALL_CANONICAL_* family slices are the closed lists this
    /// crate guarantees are registered.
    #[test]
    fn all_entries_have_registry_entries() {
        let registry_names: std::collections::HashSet<&str> =
            event_registry::registry().map(|d| d.name).collect();
        for slice in [ALL_CANONICAL_SESSION, ALL_CANONICAL_OP] {
            let missing: Vec<&str> = slice
                .iter()
                .copied()
                .filter(|name| !registry_names.contains(name))
                .collect();
            assert!(
                missing.is_empty(),
                "event types in const slice not registered in event_registry: {missing:?}\n\
                 Add entries to agent-def::event_registrations or ship_substrate::events::payload_registry"
            );
        }
    }

    /// ★ THE CROSS-LAYER SCHEMA GUARD (Phase 4a, keystone A) — this binary
    /// links BOTH registration sources (L0 payload_registry + this crate's
    /// event_registrations), so unlike ship-substrate's L0-scoped
    /// `all_known_event_types_have_schemas` it can assert the FULL claim:
    /// every event name in `ALL` has a registration, and every registration
    /// carries a payload validator that is LIVE — the substrate's emit-time
    /// `PayloadSchemaValidator` rejects malformed canonical `session.*` /
    /// `op.*` payloads (which the old hand-written L0 match silently let
    /// through, the long-standing RED).
    #[test]
    fn canonical_payload_validators_are_live() {
        use ship_substrate::events::PayloadSchemaValidator;
        use ship_substrate::events::validator::{CallerKind, EmitContext, EventValidator, ValidationError};

        fn ctx() -> EmitContext {
            EmitContext {
                caller_kind: CallerKind::Runtime,
                skill_id: None,
                binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
                session_id: None,
            }
        }

        // Malformed canonical payloads are REJECTED at emit time.
        for event_type in ["session.started", "session.turn.ended", "op.tool_use_started"] {
            let event = ship_substrate::events::EventEnvelope::new(
                event_type,
                "sess-1",
                &serde_json::json!({ "definitely": "not the payload" }),
            )
            .unwrap();
            let err = PayloadSchemaValidator.validate(&event, &ctx()).unwrap_err();
            assert!(
                matches!(err, ValidationError::SchemaViolation { .. }),
                "{event_type}: expected SchemaViolation for a malformed payload, got {err:?}"
            );
        }

        // A well-formed canonical payload passes.
        let started = crate::canonical_session::SessionStarted {
            session_id: "01HZX".into(),
            agent_id: "claude".into(),
            harness: "floki".into(),
            model: None,
            space_id: "space-1".into(),
            slug: None,
            input: None,
            parent: None,
        };
        let event = ship_substrate::events::EventEnvelope::new(
            "session.started",
            "sess-1",
            &started,
        )
        .unwrap();
        assert!(
            PayloadSchemaValidator.validate(&event, &ctx()).is_ok(),
            "a well-formed SessionStarted must pass the registration-carried validator"
        );
    }

    /// `agent.def.updated` scope/retention pin — moved here from
    /// ship-substrate when the registration moved (keystone-A Phase A),
    /// and re-pinned in job/transient-events when the kernel started
    /// honoring `Retention` (Durable = persist + deliver, Ephemeral =
    /// deliver only). The `ship-config` watcher now emits a transient
    /// `config.file_changed` for every config file; the agent-def
    /// domain consumer maps the `agents` subset into a durable
    /// `agent.def.updated` (the read model the read-only agent-def
    /// index + the `ship-agents` watch consumers key off). See
    /// `.artifacts/transient-events-design.md`.
    #[test]
    fn agent_def_updated_is_space_durable() {
        let def = event_registry::lookup("agent.def.updated")
            .expect("agent.def.updated must be in registry");
        assert_eq!(
            def.allowed_scopes,
            &[EventScope::Space],
            "def change notification is Space-scoped (it lives in a space's .ship)"
        );
        assert_eq!(def.primary_scope(), EventScope::Space);
        assert_eq!(
            def.retention,
            Retention::Durable,
            "agent.def.updated is the DOMAIN event for an agent-def file change — \
             it is the durable fact the read-only index and the per-agent watch \
             consumers key off (job/transient-events spike)"
        );
    }
}
