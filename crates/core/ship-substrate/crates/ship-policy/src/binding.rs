//! The scope binding — *which* principal acts in *which* scope.
//!
//! The binding is the inner ring of the policy primitive (north-star §2.6): it
//! supplies the authoritative scope *key* that `enforce_scope` stamps onto an
//! envelope. The registry picks the event's home scope; the binding validates
//! same-scope (Space) emits and fills the key for the others.

use ship_types::event_registry::EventScope;
use ship_types::{AccountId, DeviceId, SpaceId};

/// Operator-class and caller-class of a principal — replaces the
/// `is_operator: bool` / `Scope::Full` conflation with one named state.
///
/// The on-disk daemon/SDK token resolves to a *capped* authority (`Sdk`),
/// never `Operator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Authority {
    /// Operator-class: the human at the console / fully-trusted local control.
    Operator,
    /// Acting within a specific space.
    Space { space_id: SpaceId },
    /// An agent actor, identified by its instance id.
    Agent { id: String },
    /// The on-disk daemon/SDK token — capped, never operator-class.
    Sdk,
    /// The cloud-sync ingest path.
    CloudSync,
}

/// Authoritative binding of a principal to its scope keys.
///
/// **Non-forgeable intent (#196):** in the built system this is constructed
/// *only* by the L1 auth seam from a verified session — no in-process code can
/// mint a binding for an arbitrary space. Phase 2 makes that auth seam the sole
/// minter; the plain constructor here is the greenfield stand-in until then. The
/// kernel API takes `&ScopeBinding`, never a bare string key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBinding {
    pub device_id: DeviceId,
    pub account_id: Option<AccountId>,
    pub space_id: Option<SpaceId>,
    pub authority: Authority,
}

impl ScopeBinding {
    pub fn new(
        device_id: DeviceId,
        account_id: Option<AccountId>,
        space_id: Option<SpaceId>,
        authority: Authority,
    ) -> Self {
        Self {
            device_id,
            account_id,
            space_id,
            authority,
        }
    }

    /// The binding's key for a given scope axis:
    /// Space→space_id, Account→account_id, Device→Some(device_id), System→None.
    ///
    /// Returns `None` when the binding carries no key for that axis (e.g. a
    /// space-unbound caller asked for the Space key, or any caller asked for
    /// the keyless System scope). `enforce_scope` turns an absent Space key
    /// into a hard reject; the others stamp whatever this returns.
    pub fn key_for(&self, scope: EventScope) -> Option<String> {
        match scope {
            EventScope::Space => self.space_id.as_ref().map(|s| s.0.clone()),
            EventScope::Account => self.account_id.as_ref().map(|a| a.0.clone()),
            EventScope::Device => Some(self.device_id.0.clone()),
            EventScope::System => None,
        }
    }
}
