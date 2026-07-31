//! `ship-types` — foundational identity, event-envelope, and scope types (L0).
//!
//! The lowest crate in the stack: no runtime deps (the test in this crate
//! asserts this). Defines `AccountId`, `DeviceId`, `SpaceId`,
//! `EventEnvelope`, `ScopeBinding` (the sole-minter auth seam), `Decision`,
//! `EventScope`, and the event registry, used across all layers.

pub mod binding;
pub mod caller;
pub mod decision;
pub mod enforce;
pub mod envelope;
pub mod event_registry;
pub mod event_types;
pub mod hash;
pub mod identity;
pub mod payload;
pub mod stamp;
pub mod stamped;

// Re-export inventory so the `register_event!` macro can reference it as
// `$crate::inventory::submit!` without downstream crates needing to depend
// on inventory directly. (`register_event!` is `#[macro_export]` from
// `event_registry` and is available as `ship_types::register_event!`.)
pub use inventory;

pub use binding::ScopeBinding;
pub use caller::{ActorConfig, CallerKind, EmitContext};
pub use decision::{AskReason, Decision, ScopeViolation};
pub use enforce::{enforce_scope, stamp_author};
pub use envelope::EventEnvelope;
pub use stamp::{StampError as TypesStampError, apply_stamp};
pub use stamped::Stamped;
pub use event_registry::{
    EventDef, EventDefSource, EventRegistration, EventScope, Retention, Stability, all_names,
    allowed_scopes, lookup, registry, scope_for,
};
pub use hash::sha256_hex;
pub use identity::{AccountId, DeviceId, SpaceId, local_device_id, set_local_device_id};

use async_trait::async_trait;

/// Port: emit an event envelope. Implementations are host-specific (Ship's
/// kernel router, a test mock, etc.).
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Persist + route a single envelope. Implementations log their own
    /// failures; the unit return matches the fire-and-forget call sites.
    async fn emit(&self, envelope: EventEnvelope);
}

#[cfg(test)]
mod tests {
    #[test]
    fn ship_types_has_no_runtime_dep() {
        let cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        let deps_start = cargo_toml
            .find("[dependencies]")
            .expect("Cargo.toml must have [dependencies]");
        let deps_section = &cargo_toml[deps_start..];
        let deps_end = deps_section[1..]
            .find("\n[")
            .map(|i| i + 1)
            .unwrap_or(deps_section.len());
        let deps_only = &deps_section[..deps_end];
        assert!(
            !deps_only.contains("runtime"),
            "ship-types must not depend on runtime crate"
        );
    }
}
