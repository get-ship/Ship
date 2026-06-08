//! ConnectRPC wire layer for `IsolationService`.
//!
//! `IsolationWire` is stateless: all four ops call `ship_isolation::decision`
//! or `ship_isolation::decision_ops` functions directly. Shipd mounts it via
//! `IsolationServiceExt::register`, replacing `impl IsolationService for OpsImpl`
//! in `apps/shipd/isolation_rpc.rs`.
//!
//! All ops are C0 reads (edge-authenticated); no gate closure is needed.
//!
//! ## Wire mapping
//!
//! Per ADR `rpc-is-the-typed-contract`, each RPC returns a typed proto message
//! mirroring the `ship_isolation` serde types. The serde types stay the source
//! of truth in the crate; this wire constructs the proto messages by hand
//! from the same serde values. Enum-shaped fields (`os`, `tier`, `strategy`)
//! ride as `string` in the proto (see `isolation.proto`), so no i32-enum
//! mapping is needed — the wire just passes the snake_case serde id through
//! via `as_id()`. `ResolutionReason` is a proto `oneof`, so this wire maps
//! each serde variant to its matching `Reason` arm.
//!
//! If a future proto change introduces a non-string enum field, this module
//! is the place to add the explicit mapping (mirroring `ship_search::wire`).
//!
//! Each handler additionally round-trips the response through
//! `serde_json::from_value` against the generated proto type. That is the
//! "self-check": the conformance gate (`cargo xtask gen-fixtures --check`)
//! catches proto ↔ serde drift via fixture values, and this self-check
//! catches any divergence introduced by hand-mapping inside the wire.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_ops_rpc::__buffa::oneof::isolation_resolution_reason::Reason as ProtoReason;
use ship_ops_rpc::{
    IsolationCapabilities as ProtoCapabilities, IsolationDecision as ProtoDecision,
    IsolationPreviewResponse as ProtoPreviewResponse,
    IsolationProfile as ProtoProfile, IsolationRecommendationsResponse as ProtoRecommendations,
    IsolationResolutionAlreadyInSandbox, IsolationResolutionAgentOverride,
    IsolationResolutionDowngrade, IsolationResolutionOsDefault,
    IsolationResolutionReason as ProtoResolutionReason, IsolationResolutionRunOverride,
    IsolationResolutionUserDefault, IsolationResolved as ProtoResolved,
    IsolationService as IsolationServiceTrait, IsolationStatusResponse as ProtoStatusResponse,
    IsolationStrategyRow as ProtoStrategyRow, IsolationTierRow as ProtoTierRow,
    IsolationVerifyResponse as ProtoVerifyResponse, IsolationVerifyStep as ProtoVerifyStep,
    OwnedIsolationPreviewRequestView, OwnedIsolationRecommendationsRequestView,
    OwnedIsolationStatusRequestView, OwnedIsolationVerifyRequestView,
};

use crate::capabilities::Capabilities;
use crate::decision::{IsolationDecision, IsolationDecisionError};
use crate::decision_ops::{Recommendations, StrategyRow, TierRow, VerifyResult, VerifyStep};
use crate::recommender::{ResolutionReason, ResolvedIsolation};

fn iso_err(e: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(format!("isolation: {e}"))
}

/// Round-trip self-check: decode the JSON form of the typed proto response
/// through the generated `Deserialize` impl. Catches any drift between the
/// hand-mapped field assignment and the proto message schema. The conformance
/// gate is the upstream guard; this is the wire-time last line of defense.
fn self_check<T: serde::de::DeserializeOwned + serde::Serialize>(
    resp: &T,
    ctx: &str,
) -> Result<(), ConnectError> {
    let v = serde_json::to_value(resp).map_err(iso_err)?;
    serde_json::from_value::<T>(v)
        .map(|_| ())
        .map_err(|e| ConnectError::internal(format!("isolation wire self-check ({ctx}): {e}")))
}

// ── Capabilities ──────────────────────────────────────────────────────────────

fn capabilities_to_proto(caps: &Capabilities) -> ProtoCapabilities {
    ProtoCapabilities {
        os: host_os_id(caps.os),
        in_sandbox: caps.in_sandbox,
        docker: caps.docker,
        podman: caps.podman,
        youki: caps.youki.as_ref().map(path_to_string),
        youki_rootfs: caps.youki_rootfs.as_ref().map(path_to_string),
        lima: caps.lima.as_ref().map(path_to_string),
        tart: caps.tart.as_ref().map(path_to_string),
        wsl2: caps.wsl2,
        srt: caps.srt,
        sandbox_exec: caps.sandbox_exec,
        vz_framework: caps.vz_framework,
        firecracker: caps.firecracker,
        apple_container: caps.apple_container,
        fly_cli: caps.fly_cli,
        fly_authed: caps.fly_authed,
        ..Default::default()
    }
}

fn path_to_string(p: &std::path::PathBuf) -> String {
    p.to_string_lossy().into_owned()
}

// ── HostOs / Tier / Strategy id helpers ──────────────────────────────────────

fn host_os_id(os: crate::capabilities::HostOs) -> String {
    use crate::capabilities::HostOs;
    match os {
        HostOs::Linux => "linux",
        HostOs::MacosArm => "macos_arm",
        HostOs::MacosIntel => "macos_intel",
        HostOs::Windows => "windows",
        HostOs::Wsl2 => "wsl2",
        HostOs::Unknown => "unknown",
    }
    .to_string()
}

fn tier_id(t: crate::tier::IsolationTier) -> String {
    crate::tier::IsolationTier::as_id(&t).to_string()
}

fn strategy_id(s: crate::tier::IsolationStrategy) -> String {
    crate::tier::IsolationStrategy::as_id(&s).to_string()
}

// ── ResolutionReason oneof ───────────────────────────────────────────────────

fn resolution_reason_to_proto(r: &ResolutionReason) -> ProtoResolutionReason {
    let reason = match r {
        ResolutionReason::RunOverride => {
            ProtoReason::RunOverride(Box::new(IsolationResolutionRunOverride::default()))
        }
        ResolutionReason::AgentOverride => {
            ProtoReason::AgentOverride(Box::new(IsolationResolutionAgentOverride::default()))
        }
        ResolutionReason::UserDefault => {
            ProtoReason::UserDefault(Box::new(IsolationResolutionUserDefault::default()))
        }
        ResolutionReason::OsDefault { os } => {
            ProtoReason::OsDefault(Box::new(IsolationResolutionOsDefault {
                os: host_os_id(*os),
                ..Default::default()
            }))
        }
        ResolutionReason::Downgrade { from, why } => {
            ProtoReason::Downgrade(Box::new(IsolationResolutionDowngrade {
                from: tier_id(*from),
                why: why.clone(),
                ..Default::default()
            }))
        }
        ResolutionReason::AlreadyInSandbox => {
            ProtoReason::AlreadyInSandbox(Box::new(IsolationResolutionAlreadyInSandbox::default()))
        }
    };
    ProtoResolutionReason {
        reason: Some(reason),
        ..Default::default()
    }
}

// ── ResolvedIsolation ────────────────────────────────────────────────────────

fn resolved_to_proto(r: &ResolvedIsolation) -> ProtoResolved {
    ProtoResolved {
        strategy: strategy_id(r.strategy),
        tier: tier_id(r.tier),
        reason: buffa::MessageField::some(resolution_reason_to_proto(&r.reason)),
        downgraded: r.downgraded,
        ..Default::default()
    }
}

// ── IsolationProfile ────────────────────────────────────────────────────────

fn profile_to_proto(p: &crate::tier::IsolationProfile) -> ProtoProfile {
    ProtoProfile {
        tier: p.tier.map(tier_id),
        strategy: p.strategy.map(strategy_id),
        policy_json: p
            .policy
            .as_ref()
            .and_then(|pol| serde_json::to_string(pol).ok()),
        ..Default::default()
    }
}

// ── IsolationDecision ───────────────────────────────────────────────────────

fn decision_to_proto(d: &IsolationDecision) -> ProtoDecision {
    ProtoDecision {
        resolved: buffa::MessageField::some(resolved_to_proto(&d.resolved)),
        capabilities: buffa::MessageField::some(capabilities_to_proto(&d.capabilities)),
        user_profile_used: buffa::MessageField::some(profile_to_proto(&d.user_profile_used)),
        ..Default::default()
    }
}

// ── Status ──────────────────────────────────────────────────────────────────
//
// The previous `JsonResponse` shape wrapped a `decision: IsolationDecision |
// { error: string }`. The typed wire surfaces the underlying
// `IsolationDecisionError` as a `ConnectError`; the UI maps the error
// category into its own "unavailable" / "error" state.

fn status_response_from(
    decision: Result<IsolationDecision, IsolationDecisionError>,
) -> Result<ProtoStatusResponse, ConnectError> {
    let decision = decision.map_err(iso_err)?;
    Ok(ProtoStatusResponse {
        capabilities: buffa::MessageField::some(capabilities_to_proto(&decision.capabilities)),
        decision: buffa::MessageField::some(decision_to_proto(&decision)),
        ..Default::default()
    })
}

// ── Recommendations ────────────────────────────────────────────────────────

fn strategy_row_to_proto(s: &StrategyRow) -> ProtoStrategyRow {
    ProtoStrategyRow {
        strategy: strategy_id(s.strategy),
        available: s.available,
        unavailable_reason: s.unavailable_reason.clone().unwrap_or_default(),
        ..Default::default()
    }
}

fn tier_row_to_proto(t: &TierRow) -> ProtoTierRow {
    ProtoTierRow {
        tier: tier_id(t.tier),
        label: t.label.clone(),
        strategies: t.strategies.iter().map(strategy_row_to_proto).collect(),
        would_resolve_to: t
            .would_resolve_to
            .as_ref()
            .map(resolved_to_proto)
            .map(buffa::MessageField::some)
            .unwrap_or_default(),
        recommended: t.recommended,
        ..Default::default()
    }
}

fn recommendations_to_proto(r: &Recommendations) -> ProtoRecommendations {
    ProtoRecommendations {
        host_os: host_os_id(r.host_os),
        os_default_tier: tier_id(r.os_default_tier),
        tiers: r.tiers.iter().map(tier_row_to_proto).collect(),
        ..Default::default()
    }
}

// ── Verify ─────────────────────────────────────────────────────────────────

fn verify_step_to_proto(s: &VerifyStep) -> ProtoVerifyStep {
    ProtoVerifyStep {
        name: s.name.clone(),
        ok: s.ok,
        detail: s.detail.clone(),
        ..Default::default()
    }
}

fn verify_result_to_proto(r: &VerifyResult) -> ProtoVerifyResponse {
    ProtoVerifyResponse {
        ok: r.ok,
        steps: r.steps.iter().map(verify_step_to_proto).collect(),
        resolved: r
            .resolved
            .as_ref()
            .map(decision_to_proto)
            .map(buffa::MessageField::some)
            .unwrap_or_default(),
        ..Default::default()
    }
}

// ── Wire trait ─────────────────────────────────────────────────────────────

/// ConnectRPC wire for `IsolationService`. Stateless: calls `decision` /
/// `decision_ops` functions in the owning crate — no shipd state needed.
pub struct IsolationWire;

#[allow(refining_impl_trait)]
impl IsolationServiceTrait for IsolationWire {
    async fn status(
        &self,
        _ctx: RequestContext,
        request: OwnedIsolationStatusRequestView,
    ) -> ServiceResult<ProtoStatusResponse> {
        let caps = if request.rescan {
            crate::decision::rescan_capabilities().map_err(iso_err)?
        } else {
            crate::decision::cached_capabilities().map_err(iso_err)?
        };
        let decided = crate::decision::decide(None, None);
        // Pin the decision's `capabilities` to the snapshot we just used
        // to drive the recommender, so the response is internally
        // consistent even if the recommender re-probed.
        let decided = decided.map(|mut d| {
            d.capabilities = caps;
            d
        });
        let resp = status_response_from(decided)?;
        self_check(&resp, "status")?;
        Ok(Response::new(resp))
    }

    async fn recommendations(
        &self,
        _ctx: RequestContext,
        _request: OwnedIsolationRecommendationsRequestView,
    ) -> ServiceResult<ProtoRecommendations> {
        let recs = crate::decision_ops::recommendations().map_err(iso_err)?;
        let resp = recommendations_to_proto(&recs);
        self_check(&resp, "recommendations")?;
        Ok(Response::new(resp))
    }
    async fn preview(
        &self,
        _ctx: RequestContext,
        request: OwnedIsolationPreviewRequestView,
    ) -> ServiceResult<ProtoPreviewResponse> {
        let tier = request
            .tier
            .as_deref()
            .and_then(crate::tier::IsolationTier::from_id);
        let strategy = request
            .strategy
            .as_deref()
            .and_then(crate::tier::IsolationStrategy::from_id);
        let profile = crate::tier::IsolationProfile { tier, strategy, policy: None };
        let decision = crate::decision_ops::preview(profile).map_err(iso_err)?;
        let resp = ProtoPreviewResponse {
            decision: buffa::MessageField::some(decision_to_proto(&decision)),
            ..Default::default()
        };
        self_check(&resp, "preview")?;
        Ok(Response::new(resp))
    }

    async fn verify(
        &self,
        _ctx: RequestContext,
        _request: OwnedIsolationVerifyRequestView,
    ) -> ServiceResult<ProtoVerifyResponse> {
        let result = crate::decision_ops::verify_decision();
        let resp = verify_result_to_proto(&result);
        self_check(&resp, "verify")?;
        Ok(Response::new(resp))
    }
}
