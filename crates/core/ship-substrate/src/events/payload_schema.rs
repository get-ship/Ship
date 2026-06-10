//! Payload schema validation for known event types.
//!
//! Phase 4a (keystone A — domain-owned events): the validator for each event
//! is CARRIED BY its registration (`EventDef::validate_payload`, populated by
//! `register_event!` / `#[derive(EventPayload)]` from the declared payload
//! type) and collected here from the linked registry. There is no
//! hand-maintained name→type match anymore — the substrate validates
//! whatever is registered in the linked binary, including L1 registrations
//! (canonical `session.*` / `op.*` from agent-def) that the old L0 match
//! could never see. If a known event type has a malformed payload, it is
//! rejected before persistence. Unknown event types (custom skill events)
//! pass through.

use std::collections::HashMap;
use std::sync::OnceLock;

use ship_types::envelope::EventEnvelope;
use crate::events::validator::{EmitContext, EventValidator, ValidationError};

/// Validates payloads for known event types at emit time. Unknown event types
/// pass through — custom skill events are the caller's responsibility.
pub struct PayloadSchemaValidator;

impl EventValidator for PayloadSchemaValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        _ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if let Some(validate_fn) = schema_for_event_type(&event.event_type) {
            validate_fn(&event.payload_json).map_err(|error| {
                ValidationError::SchemaViolation {
                    event_type: event.event_type.clone(),
                    error,
                }
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
    static INDEX: OnceLock<HashMap<&'static str, PayloadValidator>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            ship_types::event_registry::registry()
                .map(|d| (d.name, d.validate_payload))
                .collect()
        })
        .get(event_type)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::validator::CallerKind;

    fn caller_ctx(kind: CallerKind) -> EmitContext {
        EmitContext {
            caller_kind: kind,
            skill_id: None,
            binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
            session_id: None,
        }
    }

    #[test]
    fn valid_payload_passes() {
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "workspace.created",
            "ws-1",
            &serde_json::json!({ "workspace_id": "ws-1" }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Runtime);
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn malformed_payload_rejected() {
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "workspace.created",
            "ws-1",
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

    // The former `all_known_event_types_have_schemas` (every `ALL` name has a
    // validator, modulo an L1-pending mirror list) is retired with `ALL`
    // itself: Phase 4a made the validator a MANDATORY field of every
    // registration (type-level guarantee), and shipd's full-linkage
    // `tests/event_vocabulary.rs` behaviorally sweeps every registered
    // validator with NO exception lists.

}
