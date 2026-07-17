//! The scope binding — *which* principal acts in *which* scope.
//!
//! The binding is the inner ring of the policy primitive (north-star §2.6): it
//! supplies the authoritative scope *key* that `enforce_scope` stamps onto an
//! envelope. The registry picks the event's home scope; the binding validates
//! same-scope (Space) emits and fills the key for the others.
//!
//! **Sole-minter (auth-is-the-sole-minter, #196).** The fields are private and
//! there is NO arbitrary public constructor. A binding is minted only through a
//! named seam:
//! - [`ScopeBinding::from_session`] — the L1 auth seam, from a verified session.
//! - [`ScopeBinding::daemon_internal`] — TRUSTED INTERNAL ONLY, for daemon-process
//!   background tasks.
//! Untrusted agents/apps are out-of-process (RPC) and cannot name either minter;
//! the daemon mints on their behalf through the auth seam. This removes the
//! in-process arbitrary-construction path.

use crate::event_registry::EventScope;
use crate::identity::local_device_id;
use crate::{AccountId, DeviceId, SpaceId};

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
/// Fields are private; mint via [`ScopeBinding::from_session`] or
/// [`ScopeBinding::daemon_internal`]. The kernel API takes `&ScopeBinding`,
/// never a bare string key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBinding {
    device_id: DeviceId,
    account_id: Option<AccountId>,
    space_id: Option<SpaceId>,
    authority: Authority,
}

impl ScopeBinding {
    /// The L1 auth seam — the SOLE session→binding constructor. Called only by
    /// `AuthContext::binding` from a verified session. Not for ad-hoc use.
    pub fn from_session(
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

    /// TRUSTED INTERNAL ONLY — the daemon-process background minter.
    ///
    /// Only daemon-process code (L0/L1/shipd) can name this; agents/apps go
    /// through RPC and the daemon mints via the auth seam for them. Sets a
    /// stable per-install device id (installed by `set_local_device_id` at
    /// boot from `state_dir()/device-id`, mode 0600; falls back to the
    /// `"local-device"` placeholder when no install has happened yet), no
    /// account, the given space + authority.
    pub fn daemon_internal(space: Option<SpaceId>, authority: Authority) -> Self {
        Self {
            device_id: local_device_id(),
            // Anonymous account until the server-auth arc lands.
            account_id: None,
            space_id: space,
            authority,
        }
    }

    pub fn device_id(&self) -> &DeviceId {
        &self.device_id
    }

    pub fn account_id(&self) -> Option<&AccountId> {
        self.account_id.as_ref()
    }

    pub fn space_id(&self) -> Option<&SpaceId> {
        self.space_id.as_ref()
    }

    pub fn authority(&self) -> &Authority {
        &self.authority
    }

    /// The binding's key for a given scope axis:
    /// Space→space_id, Account→account_id, Runtime→Some(machine/device id).
    ///
    /// Returns `None` when the binding carries no key for that axis (e.g. a
    /// space-unbound caller asked for the Space key). `enforce_scope` turns an
    /// absent Space key into a hard reject; the others stamp whatever this
    /// returns.
    pub fn key_for(&self, scope: EventScope) -> Option<String> {
        match scope {
            EventScope::Space => self.space_id.as_ref().map(|s| s.0.clone()),
            EventScope::Account => self.account_id.as_ref().map(|a| a.0.clone()),
            EventScope::Runtime => Some(self.device_id.0.clone()),
        }
    }
}
