//! `ship-types` — foundational identity, event-envelope, and scope types (L0).
//!
//! The lowest crate in the stack: no runtime deps (the test in this crate
//! asserts this). Defines `AccountId`, `DeviceId`, `SpaceId`, `ActorIdentity`,
//! `EventEnvelope`, `ScopeBinding` (the sole-minter auth seam), `Decision`,
//! `EventScope`, the event registry, and the `Vector` / `Scope` types used
//! across all layers.

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
pub mod scope;
pub mod vector;

// Re-export inventory so the `register_event!` macro can reference it as
// `::ship_types::inventory::submit!` without downstream crates needing to
// depend on inventory directly (the macro is `#[macro_export]` from ship-types).
pub use inventory;

pub use binding::{Authority, ScopeBinding};
pub use caller::{ActorConfig, CallerKind, EmitContext};
pub use decision::{AskReason, Decision, ScopeViolation};
pub use enforce::{enforce_scope, stamp_author};
pub use envelope::EventEnvelope;
pub use hash::sha256_hex;
pub use identity::{
    local_device_id, set_local_device_id, AccountId, ActorIdentity, ActorType, DeviceId,
    SpaceId,
};
pub use scope::Scope;
pub use vector::Vector;

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
