//! `enforce_scope` — the ring-2 stamp-or-reject rule.
//!
//! The registry has already stamped the envelope's home `scope` (at
//! `EventEnvelope::new`, fail-closed Device). This function supplies the
//! authoritative `scope_key` from the binding and rejects any envelope that
//! claims a Space it isn't bound to. RATIFIED hard-reject rule, north-star §2.1.

use ship_types::event_registry::EventScope;
use ship_types::EventEnvelope;

use crate::binding::ScopeBinding;
use crate::decision::ScopeViolation;

/// Stamp the envelope's `scope_key` from the binding, or reject.
///
/// - **Space event, key absent** → stamp `binding.space_id` (reject
///   `NoSpaceForSpaceEvent` if the binding is space-unbound).
/// - **Space event, key present == binding space** → ok, untouched.
/// - **Space event, key present != binding space** → `CrossSpace` reject.
/// - **Space-unbound caller emitting a Space event** → `NoSpaceForSpaceEvent`.
/// - **Account / Device / System event** → stamp the matching binding key
///   (System → `None`); no cross-check.
pub fn enforce_scope(
    env: &mut EventEnvelope,
    binding: &ScopeBinding,
) -> Result<(), ScopeViolation> {
    match env.scope {
        EventScope::Space => {
            let bound = binding
                .space_id
                .as_ref()
                .ok_or(ScopeViolation::NoSpaceForSpaceEvent)?;
            match env.scope_key.as_deref() {
                None => {
                    env.scope_key = Some(bound.0.clone());
                    Ok(())
                }
                Some(got) if got == bound.0 => Ok(()),
                Some(got) => Err(ScopeViolation::CrossSpace {
                    expected: bound.0.clone(),
                    got: got.to_string(),
                }),
            }
        }
        scope => {
            env.scope_key = binding.key_for(scope);
            Ok(())
        }
    }
}
