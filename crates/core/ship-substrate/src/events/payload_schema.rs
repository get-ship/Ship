//! Payload schema validation for known event types.
//!
//! Phase 4a (keystone A — domain-owned events): the validator for each event
//! is CARRIED BY its registration (`EventDef::validate_payload`, populated by
//! `register_event!` / `#[derive(EventPayload)]` from the declared payload
//! type) and collected here from the linked registry. There is no
//! hand-maintained name→type match anymore — the substrate validates
//! whatever is registered in the linked binary, including L1 registrations
//! (canonical `session.*` / `op.*` from agent-def) that the old L0 match
//! could never see.
//!
//! The kernel-routed emit path AND the off-kernel
//! `SqliteEventStore::append_enforced` path both funnel through
//! [`apply_stamp_chokepoint`], which stamps author identity, enforces scope,
//! and validates the payload schema in one call. If the kernel path validates
//! payloads, so does everyone — closing blueprint #91 by construction.
//! Unknown event types (custom skill events) pass through the schema step.

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use crate::events::validator::{EmitContext, EventValidator, ValidationError};
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use ship_types::Stamped;

/// The universal pre-persist chokepoint.
///
/// Every event that lands in any scope db goes through this function — the
/// kernel-routed `emit` chokepoint and `SqliteEventStore::append_enforced`
/// both call it. Steps, in order:
///
///   1. `stamp_author` — set `author_device_id` / `author_account_id` from
///      the binding (the kernel-emit path's own `validators` chain runs
///      BEFORE this, in the wrapper; this is the only stamp).
///   2. `enforce_scope` — fail-closed: the event's chosen scope must be in
///      the type's allowed-set AND the binding must be authorized for it.
///      `enforce_scope` then stamps the authoritative `scope_key` for the
///      Runtime / Account / Space axes.
///   3. `PayloadSchemaValidator::validate` — deserializes the payload into
///      the registered payload type (the one carried by `EventDef::
///      validate_payload`). Unknown event types pass through.
///   4. (optional, when `ship_dir` is provided) strict scope registration:
///      a Space `scope_key` must name a row in the `spaces` registry.
///
/// On success, returns a [`Stamped`] proof — the only production path that
/// mints one. The errors from steps 2–4 surface as [`StampError`]; step 1
/// has no failure mode.
pub fn apply_stamp_chokepoint(
    event: &mut EventEnvelope,
    binding: &ship_types::ScopeBinding,
    ctx: &EmitContext,
) -> Result<Stamped, StampError> {
    apply_stamp_chokepoint_at(event, binding, ctx, None)
}

/// Like [`apply_stamp_chokepoint`], and when `ship_dir` is `Some` also
/// rejects Space events whose `scope_key` is not a registered space id.
pub fn apply_stamp_chokepoint_at(
    event: &mut EventEnvelope,
    binding: &ship_types::ScopeBinding,
    _ctx: &EmitContext,
    ship_dir: Option<&Path>,
) -> Result<Stamped, StampError> {
    // Sole production mint of `Stamped` lives in ship-types so the
    // constructor stays crate-private there (type-state seal).
    let stamped = ship_types::apply_stamp(event, binding).map_err(|e| match e {
        ship_types::TypesStampError::Scope(s) => StampError::Scope(s),
        ship_types::TypesStampError::Schema { event_type, error } => {
            StampError::Schema(ValidationError::SchemaViolation { event_type, error })
        }
    })?;
    if let Some(root) = ship_dir {
        validate_scope_registered(stamped.as_envelope(), root).map_err(StampError::Scope)?;
    }
    Ok(stamped)
}

/// Reject a Space-scoped event whose `scope_key` has no `spaces` registry row.
///
/// Runtime / Account scopes do not consult the spaces registry. Missing
/// `scope_key` on a Space event is already rejected by `enforce_scope` /
/// `PartitionedSink::route`.
pub fn validate_scope_registered(
    event: &EventEnvelope,
    ship_dir: &Path,
) -> Result<(), ship_types::ScopeViolation> {
    if event.scope != EventScope::Space {
        return Ok(());
    }
    let Some(key) = event.scope_key.as_deref() else {
        return Err(ship_types::ScopeViolation::NoSpaceForSpaceEvent);
    };
    match crate::db::spaces::get_at(ship_dir, key) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(ship_types::ScopeViolation::UnregisteredScope {
            scope_key: key.to_string(),
        }),
        Err(e) => Err(ship_types::ScopeViolation::UnregisteredScope {
            scope_key: format!("{key} (registry read failed: {e})"),
        }),
    }
}

/// The single error type for [`apply_stamp_chokepoint`] — unifies the two
/// failure modes (scope reject, schema reject) the universal gate guards.
#[derive(Debug, thiserror::Error)]
pub enum StampError {
    #[error(transparent)]
    Scope(ship_types::ScopeViolation),
    #[error(transparent)]
    Schema(ValidationError),
}

/// Build the `EmitContext` the universal chokepoint needs when an off-kernel
/// caller (e.g. `SqliteEventStore::append_enforced`) has only a binding, not a
/// full caller kind. The schema validator ignores `caller_kind` / `skill_id`
/// / `session_id`, so `Runtime` + `None` is the conservative default.
pub fn off_kernel_ctx(binding: ship_types::ScopeBinding) -> EmitContext {
    EmitContext {
        caller_kind: ship_types::CallerKind::Runtime,
        skill_id: None,
        binding,
        session_id: None,
    }
}

/// Validates payloads for known event types at emit time. Unknown event types
/// pass through — custom skill events are the caller's responsibility.
pub struct PayloadSchemaValidator;

impl EventValidator for PayloadSchemaValidator {
    fn validate(&self, event: &EventEnvelope, _ctx: &EmitContext) -> Result<(), ValidationError> {
        if let Some(validate_fn) = schema_for_event_type(&event.event_type) {
            validate_fn(&event.payload_json).map_err(|error| ValidationError::SchemaViolation {
                event_type: event.event_type.clone(),
                error,
            })?;
        }
        Ok(())
    }
}

/// Registration-carried payload validator (deserializes into the payload
/// type the event was registered with).
type PayloadValidator = fn(&str) -> Result<(), String>;

/// Look up the registration-carried validator for an event type. `None` for
/// unregistered/custom event types (they pass through). The index is built
/// once from the linked registry — registrations from EVERY linked crate
/// (L0 payload_registry + L1 agent-def) participate.
fn schema_for_event_type(event_type: &str) -> Option<PayloadValidator> {
    static INDEX: LazyLock<HashMap<&'static str, PayloadValidator>> = LazyLock::new(|| {
        ship_types::event_registry::registry()
            .map(|d| (d.name, d.validate_payload))
            .collect()
    });
    INDEX.get(event_type).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::validator::CallerKind;

    fn caller_ctx(kind: CallerKind) -> EmitContext {
        EmitContext {
            caller_kind: kind,
            skill_id: None,
            binding: ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk),
            session_id: None,
        }
    }

    #[test]
    fn valid_payload_passes() {
        // Use actor.created — L0-registered (Runtime-scoped), stable.
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "actor.created",
            "actor-1",
            &serde_json::json!({ "kind": "worker", "environment_type": "cli" }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Runtime);
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn malformed_payload_rejected() {
        // Use actor.created — L0-registered; garbage payload missing required fields.
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "actor.created",
            "actor-1",
            &serde_json::json!({ "garbage": true }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Runtime);
        let err = v.validate(&event, &ctx).unwrap_err();
        assert!(matches!(err, ValidationError::SchemaViolation { .. }));
    }

    #[test]
    fn unknown_event_type_passes_schema() {
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "my-analytics.page_view",
            "page-1",
            &serde_json::json!({ "anything": "goes" }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Mcp);
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn apply_stamp_chokepoint_rejects_malformed_payload() {
        // The shared chokepoint must reject a malformed payload for a
        // known event type — this is what closes blueprint #91 by
        // construction: every write path (kernel emit AND
        // `append_enforced`) runs the same `stamp_author` →
        // `enforce_scope` → `PayloadSchemaValidator` gate.
        let mut event = EventEnvelope::new(
            "actor.created",
            "actor-1",
            &serde_json::json!({ "garbage": true }),
        )
        .unwrap();
        let binding = ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk);
        let ctx = off_kernel_ctx(binding.clone());
        let err = apply_stamp_chokepoint(&mut event, &binding, &ctx).unwrap_err();
        assert!(matches!(err, StampError::Schema(_)));
    }

    #[test]
    fn apply_stamp_chokepoint_stamps_author_when_valid() {
        // The shared chokepoint stamps author identity on a valid
        // event — the kernel emit and the off-kernel append_enforced
        // paths both observe the same author.
        let mut event = EventEnvelope::new(
            "actor.created",
            "actor-1",
            &serde_json::json!({ "kind": "worker", "environment_type": "cli" }),
        )
        .unwrap();
        assert!(event.author_device_id.is_none());
        let binding = ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk);
        let ctx = off_kernel_ctx(binding.clone());
        let stamped = apply_stamp_chokepoint(&mut event, &binding, &ctx).unwrap();
        assert!(event.author_device_id.is_some());
        assert!(stamped.as_envelope().author_device_id.is_some());
    }

    #[test]
    fn validate_scope_registered_rejects_fabricated_space_id() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Runtime db exists (empty spaces registry).
        let _ = crate::db::open_runtime_db(root);
        let event = EventEnvelope::new(
            "test.space",
            "e1",
            &serde_json::json!({}),
        )
        .unwrap()
        .with_scope(ship_types::EventScope::Space, Some("fabricated-no-row"));
        let err = validate_scope_registered(&event, root).unwrap_err();
        assert!(
            matches!(err, ship_types::ScopeViolation::UnregisteredScope { .. }),
            "got {err:?}"
        );

        // After registering, the same key is accepted.
        crate::testutil::register_space(root, "fabricated-no-row").unwrap();
        validate_scope_registered(&event, root).expect("registered id must pass");
    }

    #[test]
    fn apply_stamp_chokepoint_at_rejects_unregistered_space() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let _ = crate::db::open_runtime_db(root);
        let space = "ghost-space";
        let mut event = EventEnvelope::new(
            "test.space.event",
            "e1",
            &serde_json::json!({}),
        )
        .unwrap()
        .with_scope(ship_types::EventScope::Space, Some(space));
        let binding = ship_types::ScopeBinding::daemon_internal(
            Some(ship_types::SpaceId::new(space.to_string())),
            agent_schema::Authority::Sdk,
        );
        let ctx = off_kernel_ctx(binding.clone());
        let err = apply_stamp_chokepoint_at(&mut event, &binding, &ctx, Some(root)).unwrap_err();
        assert!(matches!(err, StampError::Scope(_)), "got {err:?}");
    }

    // The former `all_known_event_types_have_schemas` (every `ALL` name has a
    // validator, modulo an L1-pending mirror list) is retired with `ALL`
    // itself: Phase 4a made the validator a MANDATORY field of every
    // registration (type-level guarantee), and shipd's full-linkage
    // `tests/event_vocabulary.rs` behaviorally sweeps every registered
    // validator with NO exception lists.
}
