//! Stamp chokepoint — sole production minter of [`Stamped`].
//!
//! Lives in `ship-types` so [`Stamped`]'s private field can be initialized
//! only here (and by the feature-gated test mint). Substrate wraps this
//! with optional space-registry checks that need the db layer.

use crate::binding::ScopeBinding;
use crate::decision::ScopeViolation;
use crate::enforce::{enforce_scope, stamp_author};
use crate::envelope::EventEnvelope;
use crate::event_registry;
use crate::stamped::Stamped;

/// Error from [`apply_stamp`]: scope reject or payload-schema reject.
#[derive(Debug, thiserror::Error)]
pub enum StampError {
    #[error(transparent)]
    Scope(#[from] ScopeViolation),
    #[error("schema violation for '{event_type}': {error}")]
    Schema { event_type: String, error: String },
}

/// Universal pre-persist stamp: author identity, scope enforce, payload schema.
///
/// This is the **only** production function that constructs [`Stamped`]
/// (same-crate private field). Steps:
///
/// 1. `stamp_author` — set author device/account from the binding.
/// 2. `enforce_scope` — allowed-set membership + binding authorization + key stamp.
/// 3. Payload schema — registered `EventDef::validate_payload`; unknown types pass.
///
/// Callers that also need space-registry membership (strict scope ids) run that
/// check after this returns, with a ship-home path.
pub fn apply_stamp(
    event: &mut EventEnvelope,
    binding: &ScopeBinding,
) -> Result<Stamped, StampError> {
    stamp_author(event, binding);
    enforce_scope(event, binding)?;
    if let Some(def) = event_registry::lookup(&event.event_type)
        && let Err(error) = (def.validate_payload)(&event.payload_json)
    {
        return Err(StampError::Schema {
            event_type: event.event_type.clone(),
            error,
        });
    }
    Ok(Stamped {
        inner: event.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_schema::Authority;

    #[test]
    fn apply_stamp_mints_stamped_and_sets_author() {
        // Unregistered ad-hoc type: schema step is a no-op; author stamp still runs.
        // Schema rejection is covered in ship-substrate (full linked registry).
        let mut event =
            EventEnvelope::new("test.stamp.only", "e1", &serde_json::json!({})).unwrap();
        let binding = ScopeBinding::daemon_internal(None, Authority::Sdk);
        let stamped = apply_stamp(&mut event, &binding).unwrap();
        assert!(stamped.as_envelope().author_device_id.is_some());
    }

    #[test]
    fn apply_stamp_rejects_space_event_from_unbound_binding() {
        let mut event = EventEnvelope::new("test.space", "e1", &serde_json::json!({}))
            .unwrap()
            .with_scope(crate::event_registry::EventScope::Space, None);
        let binding = ScopeBinding::daemon_internal(None, Authority::Sdk);
        let err = apply_stamp(&mut event, &binding).unwrap_err();
        assert!(matches!(err, StampError::Scope(ScopeViolation::NoSpaceForSpaceEvent)));
    }
}
