use serde::{Deserialize, Serialize};

/// Macro-defined opaque string newtype for a scope identity axis.
/// `DeviceId`, `AccountId`, `SpaceId` are the three keyed scope axes the
/// substrate routes on (System carries no key). They are distinct types so
/// the scope binding cannot transpose one axis's key into another.
macro_rules! scope_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[cfg_attr(feature = "specta", derive(specta::Type))]
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

/// Actor identity — unique instance IDs and stable labels.
#[derive(Debug, Clone)]
pub struct ActorIdentity {
    pub instance_id: String,
    pub label: String,
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorType {
    Agent,
    Service,
    App,
    Cli,
}
