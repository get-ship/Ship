use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Macro-defined opaque string newtype for a scope identity axis.
/// `DeviceId`, `AccountId`, `SpaceId` are the three keyed scope axes the
/// substrate routes on (System carries no key). They are distinct types so
/// the scope binding cannot transpose one axis's key into another.
macro_rules! scope_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

scope_id! {
    /// Stable per-install device identity. Present in every scope binding.
    DeviceId
}
scope_id! {
    /// The user's account identity. Anonymous-by-default per install, claimed
    /// when the user opts into web services/sync.
    AccountId
}
scope_id! {
    /// Per-space (project) identity. Present only when the caller is space-bound.
    SpaceId
}

/// Process-wide installation handle for the per-machine device identity.
/// `set_local_device_id` populates it at daemon boot (mint-once in
/// `~/.ship/machine/device-id`, mode 0600); the `local_device_id` getter
/// returns that value, falling back to `"local-device"` so tests and
/// pre-boot callers (e.g. `caller.rs::daemon_internal_space_key_roundtrips`)
/// keep working without the install step.
static LOCAL_DEVICE_ID: OnceLock<DeviceId> = OnceLock::new();

/// Install the per-machine device identity. Set-once and idempotent: a
/// second call is silently ignored so daemon boot is safe to invoke from
/// tests that build multiple `ShipHome` paths. Returns `true` if this
/// call was the install, `false` if a value was already in place.
pub fn set_local_device_id(id: DeviceId) -> bool {
    LOCAL_DEVICE_ID.set(id).is_ok()
}

/// The stable per-install device identity. Returns the value installed
/// by `set_local_device_id` at daemon boot, or the `"local-device"`
/// fallback when no install has happened yet.
pub fn local_device_id() -> DeviceId {
    LOCAL_DEVICE_ID.get().cloned().unwrap_or_else(|| DeviceId::new("local-device"))
}

