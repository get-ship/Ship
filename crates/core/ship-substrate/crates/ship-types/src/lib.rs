pub mod caller;
pub mod envelope;
pub mod event_registry;
pub mod event_types;
pub mod hash;
pub mod identity;
pub mod payload;
pub mod scope;
pub mod vector;

pub use caller::{ActorConfig, CallerKind, EmitContext};
pub use envelope::EventEnvelope;
pub use hash::sha256_hex;
pub use identity::{ActorIdentity, ActorType};
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
