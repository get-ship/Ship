//! ConnectRPC op surface — the daemon's implementation of `ship.ops.v1`
//! (`ship-ops-rpc`), mounted additively beside the REST/SSE routes. Proto owns
//! op shapes + policy; this owns the handlers and (later) gate enforcement. The
//! first op, `Meta` (C0), proves the proto → trait → mount pipeline end to end.
//! See docs/ship-architecture/references/docs/future/agent-access-and-skills-api.md.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use buffa::{Message, UnknownFieldData};
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use connectrpc::{RequestContext, Response, ServiceResult};
use ship_ops_rpc::{MetaResponse, OpsService, OpsServiceExt, OwnedMetaRequestView};

/// Daemon-side `OpsService`. Stateless for the C0 `Meta` op; later ops will
/// carry the daemon's shared state.
pub struct OpsImpl;

impl OpsService for OpsImpl {
    // The trait declares `-> impl Encodable<MetaResponse>`; the documented impl
    // pattern is a plain `async fn` returning the concrete type, which refines
    // the trait's return. Intentional and benign.
    #[allow(refining_impl_trait)]
    async fn meta(
        &self,
        _ctx: RequestContext,
        _request: OwnedMetaRequestView,
    ) -> ServiceResult<MetaResponse> {
        Ok(Response::new(MetaResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            // BUILD_GIT_SHA is emitted by build-info's emit_build_env when wired;
            // "dev" is the honest fallback for an unwired local build.
            build: option_env!("BUILD_GIT_SHA").unwrap_or("dev").to_string(),
            ..Default::default()
        }))
    }
}

/// The ConnectRPC op surface as an `axum::Router`, merged into the daemon's
/// authenticated routes so `auth_middleware` wraps it (the gate will read the
/// injected `AuthContext`). Wire paths: `/ship.ops.v1.OpsService/<Method>`.
pub fn router() -> axum::Router {
    Arc::new(OpsImpl)
        .register(connectrpc::Router::new())
        .into_axum_router()
}

// ── Policy gate ─────────────────────────────────────────────────────────────
//
// Proto owns op policy (#67a): the capability / blast-radius / sensitive method
// options are compiled into the FileDescriptorSet. We read them once at first
// use rather than from generated Rust (the runtime drops custom options in
// normal codegen). Keyed by the Connect wire-path body "<package>.<Svc>/<Method>".

// Custom option field numbers — must match proto/ship/ops/v1/policy.proto.
const FIELD_CAPABILITY: u32 = 50001;
const FIELD_BLAST_RADIUS: u32 = 50002;
const FIELD_SENSITIVE: u32 = 50003;

/// Blast-radius class (mirrors `ship.ops.v1.BlastRadius`). Drives the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastRadius {
    Unspecified,
    /// Read / own-scope; harmless to Ship.
    C0Safe,
    /// Mutates the caller's own space — the user's own risk.
    C1Workspace,
    /// Ship/cloud-dangerous; auth gate (off the agent surface).
    C2Elevated,
}

impl BlastRadius {
    fn from_proto(v: u64) -> Self {
        match v {
            1 => Self::C0Safe,
            2 => Self::C1Workspace,
            3 => Self::C2Elevated,
            _ => Self::Unspecified,
        }
    }
}

/// Policy for one op, extracted from its proto method options.
#[derive(Debug, Clone)]
pub struct OpPolicy {
    pub capability: String,
    pub blast_radius: BlastRadius,
    pub sensitive: bool,
}

/// Decode the descriptor set into a `wire-path → policy` map. Called once.
fn build_policy_map() -> HashMap<String, OpPolicy> {
    let mut map = HashMap::new();
    let set = match FileDescriptorSet::decode_from_slice(ship_ops_rpc::FILE_DESCRIPTOR_SET) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("ops gate: failed to decode descriptor set: {e}");
            return map;
        }
    };
    for file in &set.file {
        let package = file.package.as_deref().unwrap_or("");
        for svc in &file.service {
            let Some(svc_name) = svc.name.as_deref() else {
                continue;
            };
            for m in &svc.method {
                let Some(method) = m.name.as_deref() else {
                    continue;
                };
                let (mut capability, mut blast, mut sensitive) =
                    (String::new(), BlastRadius::Unspecified, false);
                if let Some(opts) = m.options.as_option() {
                    for uf in opts.__buffa_unknown_fields.iter() {
                        match (uf.number, &uf.data) {
                            (FIELD_CAPABILITY, UnknownFieldData::LengthDelimited(b)) => {
                                capability = String::from_utf8_lossy(b).into_owned();
                            }
                            (FIELD_BLAST_RADIUS, UnknownFieldData::Varint(v)) => {
                                blast = BlastRadius::from_proto(*v);
                            }
                            (FIELD_SENSITIVE, UnknownFieldData::Varint(v)) => {
                                sensitive = *v != 0;
                            }
                            _ => {}
                        }
                    }
                }
                map.insert(
                    format!("{package}.{svc_name}/{method}"),
                    OpPolicy { capability, blast_radius: blast, sensitive },
                );
            }
        }
    }
    map
}

/// The op-policy map, built once from the descriptor at first lookup.
pub static OP_POLICY: LazyLock<HashMap<String, OpPolicy>> = LazyLock::new(build_policy_map);

/// Look up policy by Connect wire-path, e.g. `ship.ops.v1.OpsService/Meta`
/// (leading slash optional).
pub fn policy_for(path: &str) -> Option<&'static OpPolicy> {
    OP_POLICY.get(path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_op_policy_read_from_proto_descriptor() {
        // Proves #67a end to end: the method options declared in ops.proto are
        // compiled into the descriptor and recovered by the gate.
        let p = policy_for("ship.ops.v1.OpsService/Meta")
            .expect("Meta policy must be present in the descriptor");
        assert_eq!(p.capability, "ship.meta.read");
        assert_eq!(p.blast_radius, BlastRadius::C0Safe);
        assert!(!p.sensitive);
    }
}
