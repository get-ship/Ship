//! `enforce_scope` — the ring-2 stamp-or-reject rule, and `stamp_author` — the
//! author-identity chokepoint.
//!
//! Model C (RATIFIED): an event TYPE declares an **allowed-set** of scopes; the
//! emission's `EventEnvelope::scope` picks ONE of them (defaulted to the set's
//! primary at `EventEnvelope::new`, overridable via `with_scope`).
//! `enforce_scope`:
//!   1. requires the chosen scope ∈ the type's allowed-set (`ScopeNotAllowed`);
//!   2. requires the binding is authorized for the chosen scope (the existing
//!      Space cross-check rules) and stamps the authoritative `scope_key`.
//!
//! Unregistered event types (ad-hoc / test events) carry an EMPTY allowed-set —
//! the membership check is skipped (no declared restriction), preserving the
//! pre-Model-C behavior where unregistered events were never scope-rejected. A
//! size-1 set is identical to the old single-scope model.

use crate::binding::ScopeBinding;
use crate::decision::ScopeViolation;
use crate::event_registry::{self, EventScope};
use crate::envelope::EventEnvelope;

/// Stamp the envelope's `scope_key` from the binding, or reject.
///
/// - **Chosen scope ∉ the type's allowed-set** → `ScopeNotAllowed` (skipped for
///   unregistered types, whose allowed-set is empty).
/// - **Space event, key absent** → stamp `binding.space_id` (reject
///   `NoSpaceForSpaceEvent` if the binding is space-unbound).
/// - **Space event, key present == binding space** → ok, untouched.
/// - **Space event, key present != binding space** → `CrossSpace` reject.
/// - **Space-unbound caller emitting a Space event** → `NoSpaceForSpaceEvent`.
/// - **Account / Runtime event** → stamp the matching binding key; no cross-check.
pub fn enforce_scope(
    env: &mut EventEnvelope,
    binding: &ScopeBinding,
) -> Result<(), ScopeViolation> {
    // 1. Allowed-set membership. The chosen scope is the one on the envelope
    //    (primary by default; a poly-scopic emitter sets it via `with_scope`).
    let allowed = event_registry::allowed_scopes(&env.event_type);
    if !allowed.is_empty() && !allowed.contains(&env.scope) {
        return Err(ScopeViolation::ScopeNotAllowed {
            event_type: env.event_type.clone(),
            chosen: env.scope,
            allowed: allowed.to_vec(),
        });
    }

    // 2. Binding authorization for the chosen scope + key stamping.
    match env.scope {
        EventScope::Space => {
            let bound = binding
                .space_id()
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

/// Stamp the author identity onto an envelope from its binding — the single
/// chokepoint where author-identity lands. Sets `author_device_id` always (the
/// binding always carries a device id) and `author_account_id` when the binding
/// has an account. `signature` is left `None` — signing is a later phase
/// (see `EventEnvelope::signing_bytes` + the `signature` field doc-comment).
pub fn stamp_author(env: &mut EventEnvelope, binding: &ScopeBinding) {
    env.author_device_id = Some(binding.device_id().0.clone());
    env.author_account_id = binding.account_id().map(|a| a.0.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::Authority;
    use crate::identity::{AccountId, DeviceId, SpaceId};

    // T9 — stamp_author sources from the binding.
    #[test]
    fn stamp_author_account_bearing_binding_sets_both() {
        let binding = ScopeBinding::from_session(
            DeviceId::new("dev-1"),
            Some(AccountId::new("acct-1")),
            Some(SpaceId::new("space-A")),
            Authority::Space {
                space_id: SpaceId::new("space-A"),
            },
        );
        let mut env =
            EventEnvelope::new("workspace.created", "ws-1", &serde_json::json!({})).unwrap();
        stamp_author(&mut env, &binding);
        assert_eq!(env.author_device_id.as_deref(), Some("dev-1"));
        assert_eq!(env.author_account_id.as_deref(), Some("acct-1"));
        assert_eq!(env.signature, None);
    }

    #[test]
    fn stamp_author_device_only_binding_leaves_account_none() {
        let binding = ScopeBinding::daemon_internal(None, Authority::Sdk);
        let mut env =
            EventEnvelope::new("workspace.created", "ws-1", &serde_json::json!({})).unwrap();
        stamp_author(&mut env, &binding);
        assert!(env.author_device_id.is_some());
        assert_eq!(env.author_account_id, None);
        assert_eq!(env.signature, None);
    }
}
