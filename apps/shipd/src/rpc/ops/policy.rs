//! Policy introspection (not the enforcement path).
//!
//! Enforcement is `auth::gate` via the `GateFn` closure in `super::router`. The
//! map below reads the capability / blast-radius / sensitive method options that
//! `gen-ops` projects into the proto `FileDescriptorSet` — kept so the contract
//! is visible to every generated client (TS/docs/SDK) and conformance-tested,
//! NOT to gate calls.
//!
//! Proto owns op policy (#67a): the capability / blast-radius / sensitive method
//! options are compiled into the FileDescriptorSet. We read them once at first
//! use rather than from generated Rust (the runtime drops custom options in
//! normal codegen). Keyed by the Connect wire-path body "<package>.<Svc>/<Method>".

use std::collections::HashMap;
use std::sync::LazyLock;

/// Policy for one op, extracted from its proto method options. `blast_radius` is
/// the shared `ship_ops_rpc::BlastRadius` (one enum across the gate + this map).
#[derive(Debug, Clone)]
pub struct OpPolicy {
    pub capability: String,
    pub blast_radius: ship_ops_rpc::BlastRadius,
    pub sensitive: bool,
}

/// Build the `wire-path → policy` map. Delegates to the single crate-level
/// descriptor reader (`ship_ops_rpc::policy::read_policies`) — one decode of the
/// FileDescriptorSet, shared with the gate's op-name-keyed lookup. Called once.
fn build_policy_map() -> HashMap<String, OpPolicy> {
    ship_ops_rpc::policy::read_policies()
        .into_iter()
        .map(|p| {
            (
                p.wire_path(),
                OpPolicy {
                    capability: p.capability,
                    blast_radius: p.blast,
                    sensitive: p.sensitive,
                },
            )
        })
        .collect()
}

/// The op-policy map, built once from the descriptor at first lookup.
pub static OP_POLICY: LazyLock<HashMap<String, OpPolicy>> = LazyLock::new(build_policy_map);

/// Look up policy by Connect wire-path, e.g. `ship.ops.v1.OpsService/Meta`
/// (leading slash optional).
pub fn policy_for(path: &str) -> Option<&'static OpPolicy> {
    OP_POLICY.get(path.trim_start_matches('/'))
}
