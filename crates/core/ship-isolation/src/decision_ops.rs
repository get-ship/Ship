//! Operations the wizard / settings UI needs that go beyond
//! `decide()` — recommendations, preview, verify, init.
//!
//! Lives next to `decision.rs` to keep that file at the cap. The
//! shipd endpoints in `apps/shipd/src/routes/isolation_api.rs` are
//! thin wrappers over these functions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ship_substrate::db::daemon_settings;
use crate::capabilities::HostOs;
use crate::recommender::{
    RecommendInput, ResolvedIsolation, default_tier_for_os, recommend,
};
use crate::tier::{IsolationProfile, IsolationStrategy, IsolationTier};

use crate::decision::{
    IsolationDecision, IsolationDecisionError, cached_capabilities, decide, user_isolation_profile,
};

/// Per-tier availability + preview, for the UI picker.
///
/// One row per tier (None..Cloud). Each row lists the strategies in
/// that tier with their availability + a preview of what would happen
/// if the user picked the tier (which strategy the recommender would
/// resolve to, downgrade if any).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendations {
    pub host_os: HostOs,
    pub os_default_tier: IsolationTier,
    pub tiers: Vec<TierRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRow {
    pub tier: IsolationTier,
    pub label: String,
    pub strategies: Vec<StrategyRow>,
    /// What `decide()` would return if the user pinned this tier. None
    /// when no strategy in this tier is available on this host.
    pub would_resolve_to: Option<ResolvedIsolation>,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRow {
    pub strategy: IsolationStrategy,
    pub available: bool,
    /// Short reason a strategy isn't available — for the tooltip.
    /// None when `available` is true.
    pub unavailable_reason: Option<String>,
}

/// Build the full recommendations payload. Static read against the
/// cached capability snapshot — no probes, no spawns.
pub fn recommendations() -> Result<Recommendations, IsolationDecisionError> {
    let caps = cached_capabilities().map_err(IsolationDecisionError::Capabilities)?;
    let available = caps.available_strategies();
    let os_default_tier = default_tier_for_os(caps.os);
    let all_tiers = [
        IsolationTier::None,
        IsolationTier::Process,
        IsolationTier::Container,
        IsolationTier::Vm,
        IsolationTier::Cloud,
    ];
    let tiers = all_tiers
        .into_iter()
        .map(|tier| {
            let strategies = strategies_for_tier(tier, caps.os)
                .into_iter()
                .map(|strategy| StrategyRow {
                    strategy,
                    available: available.contains(&strategy),
                    unavailable_reason: if available.contains(&strategy) {
                        None
                    } else {
                        Some(unavailable_reason(strategy, &caps))
                    },
                })
                .collect::<Vec<_>>();
            let would_resolve_to = preview_internal(
                IsolationProfile {
                    tier: Some(tier),
                    strategy: None,
                    policy: None,
                },
            )
            .ok()
            .map(|d| d.resolved);
            TierRow {
                tier,
                label: tier.label().to_string(),
                strategies,
                would_resolve_to,
                recommended: tier == os_default_tier,
            }
        })
        .collect();
    Ok(Recommendations {
        host_os: caps.os,
        os_default_tier,
        tiers,
    })
}

/// "If I picked this profile, what would happen?" — runs the
/// recommender with the given profile as a hypothetical
/// `run_override`. No side effects.
pub fn preview(profile: IsolationProfile) -> Result<IsolationDecision, IsolationDecisionError> {
    preview_internal(profile)
}

fn preview_internal(
    profile: IsolationProfile,
) -> Result<IsolationDecision, IsolationDecisionError> {
    let caps = cached_capabilities().map_err(IsolationDecisionError::Capabilities)?;
    let (user_pref, allow_tier_0) =
        user_isolation_profile(caps.os).map_err(IsolationDecisionError::Settings)?;
    let input = RecommendInput::new(&caps)
        .with_allow_tier_0(allow_tier_0)
        .with_user_pref(&user_pref)
        .with_run_override(&profile);
    let resolved = recommend(input).map_err(IsolationDecisionError::Recommender)?;
    Ok(IsolationDecision {
        resolved,
        capabilities: caps,
        user_profile_used: user_pref,
    })
}

/// Verifies that the current configuration is sound *without* running
/// containers. Cheap; safe to call from a UI button. Checks:
///   - decision resolves with the live settings
///   - resolved strategy is one we have an implementation for
///   - strategy's required capabilities are present
///
/// Does NOT call `prepare()` / `exec()` — that's the expensive
/// roundtrip path (separate endpoint, gated by user click).
pub fn verify_decision() -> VerifyResult {
    let mut steps = Vec::new();
    let decision = match decide(None, None) {
        Ok(d) => {
            steps.push(VerifyStep {
                name: "decision_resolves".into(),
                ok: true,
                detail: format!(
                    "resolved to {} ({}); reason: {:?}",
                    d.resolved.strategy, d.resolved.tier, d.resolved.reason
                ),
            });
            d
        }
        Err(e) => {
            steps.push(VerifyStep {
                name: "decision_resolves".into(),
                ok: false,
                detail: format!("recommender failed: {e}"),
            });
            return VerifyResult { ok: false, steps, resolved: None };
        }
    };

    // Strategy implementation check. The Registry is built lazily in
    // ship-supervisor; here we hard-code the set of strategies we have
    // backends for. Falls out of sync at compile time if backends are
    // added — keep the match exhaustive.
    let strategy = decision.resolved.strategy;
    let has_impl = match strategy {
        IsolationStrategy::Noop => true,
        IsolationStrategy::HarnessNative => true,
        IsolationStrategy::SrtWrapper => true,
        IsolationStrategy::HarnessSandbox => true,
        IsolationStrategy::DockerEngine => true,
        IsolationStrategy::YoukiRootless => true,
        // WSL is spawn-wired (process.rs `wire_sidecar_sandbox`); the Windows
        // substrate. Selection is Windows-gated; the code compiles everywhere.
        IsolationStrategy::Wsl2 => true,
        // Lima is spawn-wired (process.rs `wire_sidecar_sandbox`) and Linux
        // POC-verified; live-green on macOS is gated on a Mac. `lima`
        // capability + macOS gate selection.
        IsolationStrategy::LimaVm => true,
        // Apple Container backend is deferred (no `container` CLI driver yet) —
        // recommendable but refuses under enforce until built on Apple Silicon.
        IsolationStrategy::AppleContainer => false,
        // Firecracker is spawn-wired (process.rs `wire_sidecar_sandbox`) +
        // live-tested (`firecracker_live`). Linux/KVM only; the `firecracker`
        // capability gates actual selection.
        IsolationStrategy::Firecracker => true,
        IsolationStrategy::FlyMachines => false,
    };
    steps.push(VerifyStep {
        name: "backend_implemented".into(),
        ok: has_impl,
        detail: if has_impl {
            format!("backend for {strategy} is implemented")
        } else {
            format!("backend for {strategy} is not yet implemented")
        },
    });

    if decision.resolved.downgraded {
        steps.push(VerifyStep {
            name: "no_silent_downgrade".into(),
            ok: false,
            detail: format!(
                "decision was downgraded: {:?} — enforcement would refuse to launch",
                decision.resolved.reason
            ),
        });
    } else {
        steps.push(VerifyStep {
            name: "no_silent_downgrade".into(),
            ok: true,
            detail: "decision matches the configured tier".into(),
        });
    }

    let ok = steps.iter().all(|s| s.ok);
    VerifyResult { ok, steps, resolved: Some(decision) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub ok: bool,
    pub steps: Vec<VerifyStep>,
    pub resolved: Option<IsolationDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyStep {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

/// Apply user choices in one atomic call. UI wizard's "Save" button
/// calls this; if enforcement is being turned on, verify must pass
/// before settings persist.
#[derive(Debug, Clone, Deserialize)]
pub struct InitRequest {
    pub tier: Option<String>,
    pub strategy: Option<String>,
    pub enforce: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitResult {
    pub ok: bool,
    pub applied: Vec<String>,
    pub verify: VerifyResult,
}

pub fn init(req: InitRequest) -> Result<InitResult> {
    let mut applied = Vec::new();

    if let Some(tier) = req.tier.as_deref() {
        daemon_settings::set(
            "isolation.default_tier",
            &serde_json::Value::String(tier.to_string()),
        )?;
        applied.push(format!("isolation.default_tier = {tier}"));
    }
    if let Some(strategy) = req.strategy.as_deref() {
        daemon_settings::set(
            "isolation.default_strategy",
            &serde_json::Value::String(strategy.to_string()),
        )?;
        applied.push(format!("isolation.default_strategy = {strategy}"));
    }

    // Verify BEFORE flipping enforce — if we'd refuse to launch, don't
    // turn enforcement on.
    let verify = verify_decision();
    if let Some(true) = req.enforce {
        if !verify.ok {
            return Ok(InitResult {
                ok: false,
                applied,
                verify,
            });
        }
        daemon_settings::set(
            "isolation.enforce",
            &serde_json::Value::String("true".into()),
        )?;
        applied.push("isolation.enforce = true".into());
    } else if let Some(false) = req.enforce {
        daemon_settings::set(
            "isolation.enforce",
            &serde_json::Value::String("false".into()),
        )?;
        applied.push("isolation.enforce = false".into());
    }

    Ok(InitResult { ok: true, applied, verify })
}

/// Strategies that *could* live in a tier on this OS — independent of
/// whether they're available. Drives the picker's full option list.
fn strategies_for_tier(tier: IsolationTier, os: HostOs) -> Vec<IsolationStrategy> {
    match tier {
        IsolationTier::None => vec![IsolationStrategy::Noop],
        IsolationTier::Process => vec![
            IsolationStrategy::HarnessNative,
            IsolationStrategy::SrtWrapper,
        ],
        IsolationTier::Container => match os {
            HostOs::Windows => {
                vec![IsolationStrategy::DockerEngine, IsolationStrategy::Wsl2]
            }
            _ => vec![
                IsolationStrategy::DockerEngine,
                IsolationStrategy::YoukiRootless,
            ],
        },
        IsolationTier::Vm => vec![IsolationStrategy::LimaVm],
        IsolationTier::Cloud => vec![IsolationStrategy::FlyMachines],
    }
}

fn unavailable_reason(strategy: IsolationStrategy, caps: &crate::Capabilities) -> String {
    match strategy {
        IsolationStrategy::Noop => "always available".into(),
        IsolationStrategy::HarnessNative => {
            if matches!(caps.os, HostOs::Windows | HostOs::Unknown) {
                "no harness supports native sandbox on this OS".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::SrtWrapper => {
            if !caps.srt {
                "sandbox-runtime not installed (bun add -g @anthropic-ai/sandbox-runtime)".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::HarnessSandbox => {
            if !caps.srt {
                "sandbox-runtime not installed (bun add -g @anthropic-ai/sandbox-runtime)".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::DockerEngine => {
            if !caps.docker && !caps.podman {
                "docker or podman daemon not reachable".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::YoukiRootless => {
            if caps.youki.is_none() {
                "youki binary not installed".into()
            } else if caps.youki_rootfs.is_none() {
                "agent rootfs not built (scripts/build_agent_rootfs.sh)".into()
            } else if matches!(caps.os, HostOs::Windows) {
                "youki does not support Windows".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::Wsl2 => {
            if !caps.wsl2 {
                "WSL2 not installed / not configured".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::Firecracker => {
            if !caps.firecracker {
                "/dev/kvm not accessible (Linux/KVM only; check the kvm group)".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::LimaVm => {
            if caps.lima.is_none() {
                "limactl not installed".into()
            } else if !matches!(caps.os, HostOs::MacosArm | HostOs::MacosIntel) {
                "Lima is the macOS substrate (arm64 via vz, Intel via qemu)".into()
            } else {
                "unknown".into()
            }
        }
        IsolationStrategy::AppleContainer => {
            if !caps.apple_container {
                "Apple `container` CLI not present (macOS 15+ Apple Silicon)".into()
            } else {
                "Apple Container backend not yet wired".into()
            }
        }
        IsolationStrategy::FlyMachines => {
            if !caps.fly_cli {
                "flyctl not installed".into()
            } else if !caps.fly_authed {
                "flyctl not authenticated (run `flyctl auth login`)".into()
            } else {
                "unknown".into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendations_returns_all_tiers() {
        let r = recommendations().expect("smoke");
        assert_eq!(r.tiers.len(), 5);
        assert!(r.tiers.iter().any(|t| t.tier == IsolationTier::None));
        assert!(r.tiers.iter().any(|t| t.tier == IsolationTier::Cloud));
    }

    #[test]
    fn recommendations_marks_recommended_tier() {
        let r = recommendations().expect("smoke");
        let recommended: Vec<_> = r.tiers.iter().filter(|t| t.recommended).collect();
        assert_eq!(recommended.len(), 1);
        assert_eq!(recommended[0].tier, r.os_default_tier);
    }

    #[test]
    fn preview_with_tier_none_resolves_to_noop() {
        let profile = IsolationProfile::pin_tier(IsolationTier::None);
        let d = preview(profile).expect("smoke");
        assert_eq!(d.resolved.strategy, IsolationStrategy::Noop);
        assert_eq!(d.resolved.tier, IsolationTier::None);
    }

    #[test]
    fn verify_decision_smokes_without_panic() {
        let v = verify_decision();
        // ok or not, the steps list should always have entries.
        assert!(!v.steps.is_empty());
    }
}
