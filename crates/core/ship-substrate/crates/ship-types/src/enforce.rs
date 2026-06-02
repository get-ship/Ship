//! `enforce_scope` — the ring-2 stamp-or-reject rule, and `stamp_author` — the
//! author-identity chokepoint.
//!
//! The registry has already stamped the envelope's home `scope` (at
//! `EventEnvelope::new`, fail-closed Device). `enforce_scope` supplies the
//! authoritative `scope_key` from the binding and rejects any envelope that
//! claims a Space it isn't bound to. RATIFIED hard-reject rule, north-star §2.1.
//!
//! NOTE (Phase 2): `enforce_scope` is NOT yet wired onto the emit path — that is
//! Phase 3. It lives here as the binding-keyed rule the kernel will call.

use crate::binding::ScopeBinding;
use crate::decision::ScopeViolation;
use crate::event_registry::EventScope;
use crate::envelope::EventEnvelope;

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
