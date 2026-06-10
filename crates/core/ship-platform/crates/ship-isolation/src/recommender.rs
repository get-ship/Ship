//! Maps (capabilities, OS, user_pref, agent_override, run_override) to a
//! concrete `IsolationStrategy`.
//!
//! Per `docs/strategy.md` §4 + §6. Honest defaults; loud refusals when a
//! user pin can't be satisfied; recorded downgrades when a soft preference
//! falls back.
//!
//! The recommender is pure. All inputs are passed in; it never reads the
//! filesystem or env. Probing is `capabilities::Capabilities::probe()`,
//! one layer up.

use serde::{Deserialize, Serialize};

use crate::capabilities::{Capabilities, HostOs};
use crate::tier::{IsolationProfile, IsolationStrategy, IsolationTier};

/// What the recommender returns. The orchestrator turns this into a
/// concrete `Isolator` via the registry; the UI uses `tier` + `reason` +
/// `downgraded` to communicate honestly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedIsolation {
    pub strategy: IsolationStrategy,
    pub tier: IsolationTier,
    pub reason: ResolutionReason,
    pub downgraded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResolutionReason {
    /// Explicit per-run override (`ship use --isolation=…`).
    RunOverride,
    /// Pinned in agent profile YAML.
    AgentOverride,
    /// User's daemon-wide default.
    UserDefault,
    /// Per-OS recommended tier from `strategy.md` §4.
    OsDefault { os: HostOs },
    /// The previous choice wasn't available; this is the fallback.
    Downgrade { from: IsolationTier, why: String },
    /// Already inside someone else's sandbox; no point double-wrapping.
    AlreadyInSandbox,
}

/// What couldn't be satisfied. The orchestrator surfaces this to the
/// user instead of silently falling back when the human pinned something.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RecommenderError {
    #[error("strategy '{0}' is pinned but not available on this host")]
    StrategyUnavailable(IsolationStrategy),
    #[error("tier '{0}' is pinned but no strategy in that tier is available")]
    TierUnavailable(IsolationTier),
    #[error(
        "no strategy available and Tier 0 is forbidden by isolation.allow_tier_0 = false"
    )]
    NoFallback,
}

/// Inputs to `recommend()`. Narrowest wins: `run` > `agent` > `user` >
/// `os default`.
#[derive(Debug, Clone)]
pub struct RecommendInput<'a> {
    pub caps: &'a Capabilities,
    pub run_override: Option<&'a IsolationProfile>,
    pub agent_override: Option<&'a IsolationProfile>,
    pub user_pref: Option<&'a IsolationProfile>,
    /// When false (the user's `isolation.allow_tier_0` setting is off),
    /// the recommender refuses to fall back to `Noop` and returns
    /// `NoFallback` instead.
    pub allow_tier_0: bool,
}

impl<'a> RecommendInput<'a> {
    /// Minimal input: just caps, everything else defaulted (no overrides,
    /// Tier 0 allowed). Callers layer on `with_*` methods.
    pub fn new(caps: &'a Capabilities) -> Self {
        Self {
            caps,
            run_override: None,
            agent_override: None,
            user_pref: None,
            allow_tier_0: true,
        }
    }

    pub fn with_run_override(mut self, p: &'a IsolationProfile) -> Self {
        self.run_override = Some(p);
        self
    }

    pub fn with_agent_override(mut self, p: &'a IsolationProfile) -> Self {
        self.agent_override = Some(p);
        self
    }

    pub fn with_user_pref(mut self, p: &'a IsolationProfile) -> Self {
        self.user_pref = Some(p);
        self
    }

    pub fn with_allow_tier_0(mut self, v: bool) -> Self {
        self.allow_tier_0 = v;
        self
    }
}

/// Compute the resolved strategy. See module doc for algorithm.
pub fn recommend(input: RecommendInput<'_>) -> Result<ResolvedIsolation, RecommenderError> {
    let caps = input.caps;

    if caps.in_sandbox && input.run_override.is_none() && input.agent_override.is_none() {
        return Ok(ResolvedIsolation {
            strategy: IsolationStrategy::Noop,
            tier: IsolationTier::None,
            reason: ResolutionReason::AlreadyInSandbox,
            downgraded: false,
        });
    }

    if let Some(profile) = input.run_override {
        return pin_or_error(caps, profile, ResolutionReason::RunOverride);
    }
    if let Some(profile) = input.agent_override {
        return pin_or_error(caps, profile, ResolutionReason::AgentOverride);
    }
    if let Some(profile) = input.user_pref {
        if let Some(resolved) = try_profile(caps, profile, ResolutionReason::UserDefault) {
            return Ok(resolved);
        }
    }

    let preferred_tier = default_tier_for_os(caps.os);
    let reason = ResolutionReason::OsDefault { os: caps.os };
    if let Some(strategy) = preferred_strategy_in_tier(caps, preferred_tier) {
        return Ok(ResolvedIsolation {
            strategy,
            tier: preferred_tier,
            reason,
            downgraded: false,
        });
    }

    downgrade_chain(caps, preferred_tier, input.allow_tier_0)
}

fn pin_or_error(
    caps: &Capabilities,
    profile: &IsolationProfile,
    reason: ResolutionReason,
) -> Result<ResolvedIsolation, RecommenderError> {
    if let Some(strategy) = profile.strategy {
        if caps.available_strategies().contains(&strategy) {
            return Ok(ResolvedIsolation {
                strategy,
                tier: strategy.tier(),
                reason,
                downgraded: false,
            });
        }
        return Err(RecommenderError::StrategyUnavailable(strategy));
    }
    if let Some(tier) = profile.tier {
        if let Some(strategy) = preferred_strategy_in_tier(caps, tier) {
            return Ok(ResolvedIsolation {
                strategy,
                tier,
                reason,
                downgraded: false,
            });
        }
        return Err(RecommenderError::TierUnavailable(tier));
    }
    // Empty profile — fall through to OS default.
    let preferred_tier = default_tier_for_os(caps.os);
    if let Some(strategy) = preferred_strategy_in_tier(caps, preferred_tier) {
        return Ok(ResolvedIsolation {
            strategy,
            tier: preferred_tier,
            reason: ResolutionReason::OsDefault { os: caps.os },
            downgraded: false,
        });
    }
    Err(RecommenderError::TierUnavailable(preferred_tier))
}

fn try_profile(
    caps: &Capabilities,
    profile: &IsolationProfile,
    reason: ResolutionReason,
) -> Option<ResolvedIsolation> {
    if let Some(strategy) = profile.strategy {
        if caps.available_strategies().contains(&strategy) {
            return Some(ResolvedIsolation {
                strategy,
                tier: strategy.tier(),
                reason,
                downgraded: false,
            });
        }
        return None;
    }
    if let Some(tier) = profile.tier {
        if let Some(strategy) = preferred_strategy_in_tier(caps, tier) {
            return Some(ResolvedIsolation {
                strategy,
                tier,
                reason,
                downgraded: false,
            });
        }
    }
    None
}

/// Within a tier, the preference order is per-OS-context-aware. We pick
/// the smallest-surface strategy that's actually available.
///
/// Tier 1 preference order is HarnessNative → SrtWrapper → legacy
/// HarnessSandbox alias. Recommender doesn't know the agent's harness
/// id here; that narrowing happens one layer up via
/// `ship_platform::isolation::decide_for_agent`, which consults
/// `harness::tier1_mechanism_for(harness_id, os)` and pins the
/// resulting strategy as `run_override`.
fn preferred_strategy_in_tier(
    caps: &Capabilities,
    tier: IsolationTier,
) -> Option<IsolationStrategy> {
    let candidates: &[IsolationStrategy] = match tier {
        IsolationTier::None => &[IsolationStrategy::Noop],
        IsolationTier::Process => &[
            IsolationStrategy::HarnessNative,
            IsolationStrategy::SrtWrapper,
            IsolationStrategy::HarnessSandbox,
        ],
        IsolationTier::Container => match caps.os {
            HostOs::Windows => &[IsolationStrategy::DockerEngine, IsolationStrategy::Wsl2],
            _ => &[IsolationStrategy::DockerEngine, IsolationStrategy::YoukiRootless],
        },
        IsolationTier::Vm => match caps.os {
            // Linux/WSL2 → Firecracker microVM (own kernel, cheap boot).
            HostOs::Linux | HostOs::Wsl2 => &[IsolationStrategy::Firecracker],
            // mac arm64 → Lima (wired) first, Apple Container (stronger:
            // microVM-per-container) second. The order flips to prefer Apple
            // Container once its backend is wired — today has_impl=false, so
            // auto-preferring it would refuse under enforce.
            HostOs::MacosArm => &[IsolationStrategy::LimaVm, IsolationStrategy::AppleContainer],
            // Intel mac (+ anything else) → Lima via qemu.
            _ => &[IsolationStrategy::LimaVm],
        },
        IsolationTier::Cloud => &[IsolationStrategy::FlyMachines],
    };
    let available = caps.available_strategies();
    candidates
        .iter()
        .copied()
        .find(|s| available.contains(s))
}

/// Does this OS provide an in-place Tier 1 (host process-level) sandbox?
/// Windows has none — `@anthropic-ai/sandbox-runtime` is mac+Linux only and
/// there's no portable Win32 primitive. `shipd` still runs native on Windows;
/// isolation there means escalating to a Linux substrate at *runtime*
/// (Docker-on-Windows or WSL — never an install target). The per-OS default
/// routes Windows to Container; this names the policy and drives the nudge
/// when neither substrate is installed.
pub fn os_provides_tier1(os: HostOs) -> bool {
    !matches!(os, HostOs::Windows | HostOs::Unknown)
}

/// Per-OS default tier table. Mirrors `strategy.md` §4.
pub fn default_tier_for_os(os: HostOs) -> IsolationTier {
    match os {
        HostOs::Linux | HostOs::Wsl2 => IsolationTier::Container,
        HostOs::MacosArm | HostOs::MacosIntel => IsolationTier::Process,
        HostOs::Windows => IsolationTier::Container,
        HostOs::Unknown => IsolationTier::None,
    }
}

fn downgrade_chain(
    caps: &Capabilities,
    from_tier: IsolationTier,
    allow_tier_0: bool,
) -> Result<ResolvedIsolation, RecommenderError> {
    // Walk down by tier strength.
    let descending: [IsolationTier; 5] = [
        IsolationTier::Cloud,
        IsolationTier::Vm,
        IsolationTier::Container,
        IsolationTier::Process,
        IsolationTier::None,
    ];
    for tier in descending.into_iter().filter(|t| *t < from_tier) {
        if tier == IsolationTier::None && !allow_tier_0 {
            continue;
        }
        if let Some(strategy) = preferred_strategy_in_tier(caps, tier) {
            // Landing on Noop on Windows = the user got no isolation because
            // Windows has no Tier 1 and no substrate is installed. Nudge WSL2
            // rather than present host-process as a normal fallback.
            let why = if tier == IsolationTier::None && matches!(caps.os, HostOs::Windows) {
                "no isolation strategy available — Windows has no Tier 1; enable WSL2 to get \
                 the Ship substrate"
                    .to_string()
            } else {
                format!("no {} strategy available", from_tier.as_id())
            };
            return Ok(ResolvedIsolation {
                strategy,
                tier,
                reason: ResolutionReason::Downgrade { from: from_tier, why },
                downgraded: true,
            });
        }
    }
    if allow_tier_0 {
        // Strange — Noop should always be available. Treat as bug.
        return Ok(ResolvedIsolation {
            strategy: IsolationStrategy::Noop,
            tier: IsolationTier::None,
            reason: ResolutionReason::Downgrade {
                from: from_tier,
                why: "no isolation strategy available; falling back to host process".into(),
            },
            downgraded: true,
        });
    }
    Err(RecommenderError::NoFallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn empty_caps() -> Capabilities {
        Capabilities {
            os: HostOs::Linux,
            in_sandbox: false,
            ..Capabilities::default()
        }
    }

    fn caps_with_docker() -> Capabilities {
        Capabilities { docker: true, ..empty_caps() }
    }

    fn caps_with_youki() -> Capabilities {
        Capabilities {
            youki: Some(PathBuf::from("/tmp/youki")),
            youki_rootfs: Some(PathBuf::from("/tmp/rootfs")),
            ..empty_caps()
        }
    }

    #[test]
    fn already_in_sandbox_is_noop_unless_run_override() {
        let mut caps = caps_with_docker();
        caps.in_sandbox = true;
        let out = recommend(RecommendInput::new(&caps)).unwrap();
        assert_eq!(out.strategy, IsolationStrategy::Noop);
        assert_eq!(out.reason, ResolutionReason::AlreadyInSandbox);
    }

    #[test]
    fn run_override_overrides_in_sandbox() {
        let mut caps = caps_with_docker();
        caps.in_sandbox = true;
        let profile = IsolationProfile::pin_strategy(IsolationStrategy::DockerEngine);
        let out = recommend(RecommendInput::new(&caps).with_run_override(&profile)).unwrap();
        assert_eq!(out.strategy, IsolationStrategy::DockerEngine);
        assert_eq!(out.reason, ResolutionReason::RunOverride);
    }

    #[test]
    fn run_override_with_unavailable_strategy_errors_loud() {
        let caps = empty_caps();
        let profile = IsolationProfile::pin_strategy(IsolationStrategy::DockerEngine);
        let err = recommend(RecommendInput::new(&caps).with_run_override(&profile)).unwrap_err();
        assert!(matches!(err, RecommenderError::StrategyUnavailable(_)));
    }

    #[test]
    fn user_pref_downgrades_silently_if_tier_unavailable() {
        // empty_caps() has Linux OS + no docker/youki. Container tier
        // is unavailable; recommender walks down to Process tier where
        // HarnessNative is always-available on Linux (per-harness
        // narrowing happens one layer up in decide_for_agent).
        let caps = empty_caps();
        let profile = IsolationProfile::pin_tier(IsolationTier::Container);
        let out = recommend(RecommendInput::new(&caps).with_user_pref(&profile)).unwrap();
        assert_eq!(out.tier, IsolationTier::Process);
        assert_eq!(out.strategy, IsolationStrategy::HarnessNative);
        assert!(out.downgraded);
    }

    #[test]
    fn prefer_docker_over_youki_in_container_tier_on_linux() {
        let mut caps = caps_with_youki();
        caps.docker = true;
        let out = recommend(RecommendInput::new(&caps)).unwrap();
        assert_eq!(out.strategy, IsolationStrategy::DockerEngine);
    }

    #[test]
    fn youki_picked_when_docker_absent_on_linux() {
        let caps = caps_with_youki();
        let out = recommend(RecommendInput::new(&caps)).unwrap();
        assert_eq!(out.strategy, IsolationStrategy::YoukiRootless);
    }

    #[test]
    fn firecracker_picked_for_vm_tier_on_linux() {
        // KVM-accessible Linux + a Vm-tier request → Firecracker (the Linux
        // microVM). LimaVm is the mac path, so it must NOT be chosen here.
        let caps = Capabilities { firecracker: true, ..empty_caps() };
        let profile = IsolationProfile::pin_tier(IsolationTier::Vm);
        let out = recommend(RecommendInput::new(&caps).with_user_pref(&profile)).unwrap();
        assert_eq!(out.tier, IsolationTier::Vm);
        assert_eq!(out.strategy, IsolationStrategy::Firecracker);
    }

    #[test]
    fn vm_tier_unavailable_without_kvm_downgrades() {
        // No firecracker capability on Linux → Vm tier has no available
        // strategy; a soft Vm preference walks down rather than picking an
        // unspawnable strategy.
        let caps = empty_caps();
        let profile = IsolationProfile::pin_tier(IsolationTier::Vm);
        let out = recommend(RecommendInput::new(&caps).with_user_pref(&profile)).unwrap();
        assert_ne!(out.strategy, IsolationStrategy::Firecracker);
        assert!(out.downgraded);
    }

    #[test]
    fn macos_default_is_process_tier() {
        let caps = Capabilities {
            os: HostOs::MacosArm,
            srt: true,
            ..Capabilities::default()
        };
        let out = recommend(RecommendInput::new(&caps)).unwrap();
        assert_eq!(out.tier, IsolationTier::Process);
        // Tier 1 preference order: HarnessNative wins over SrtWrapper
        // and the legacy HarnessSandbox alias.
        assert_eq!(out.strategy, IsolationStrategy::HarnessNative);
    }

    #[test]
    fn no_fallback_when_tier_0_disallowed_and_no_tier_1_either() {
        // Windows + no caps -> no Tier 1 (HarnessNative gated off
        // Windows; no srt) and no Tier 2+. allow_tier_0=false should
        // refuse outright.
        let caps = Capabilities {
            os: HostOs::Windows,
            ..Capabilities::default()
        };
        let err = recommend(RecommendInput::new(&caps).with_allow_tier_0(false)).unwrap_err();
        assert!(matches!(err, RecommenderError::NoFallback));
    }

    #[test]
    fn downgrade_reason_carries_from_tier() {
        // Use Windows + empty caps so Container is unreachable AND
        // there's no Tier 1 fallback — recommender ends at Noop with
        // the downgrade reason chained back to the OS default tier.
        let caps = Capabilities {
            os: HostOs::Windows,
            ..Capabilities::default()
        };
        let out = recommend(RecommendInput::new(&caps)).unwrap();
        match out.reason {
            ResolutionReason::Downgrade { from, .. } => {
                assert_eq!(from, IsolationTier::Container);
            }
            other => panic!("expected Downgrade, got {other:?}"),
        }
    }

    fn windows_caps_with_wsl2() -> Capabilities {
        Capabilities { os: HostOs::Windows, wsl2: true, ..empty_caps() }
    }

    #[test]
    fn windows_has_no_tier1() {
        assert!(!os_provides_tier1(HostOs::Windows));
        assert!(os_provides_tier1(HostOs::Linux));
        assert!(os_provides_tier1(HostOs::MacosArm));
    }

    #[test]
    fn windows_pushes_to_wsl_substrate_not_noop() {
        // No Tier 1 on Windows → the OS default is the WSL substrate, not a
        // silent drop to Noop.
        let caps = windows_caps_with_wsl2();
        let out = recommend(RecommendInput::new(&caps)).unwrap();
        assert_eq!(out.tier, IsolationTier::Container);
        assert_eq!(out.strategy, IsolationStrategy::Wsl2);
        assert!(!out.downgraded);
    }

    #[test]
    fn windows_soft_process_pref_escalates_to_wsl() {
        // A soft preference for Tier 1 (which Windows lacks) must not error
        // or drop to Noop — it lands in the WSL substrate (the push).
        let caps = windows_caps_with_wsl2();
        let pref = IsolationProfile::pin_tier(IsolationTier::Process);
        let out = recommend(RecommendInput::new(&caps).with_user_pref(&pref)).unwrap();
        assert_eq!(out.strategy, IsolationStrategy::Wsl2);
    }

    #[test]
    fn windows_without_substrate_nudges_wsl() {
        // No WSL2, no Docker → Noop, but the reason nudges enabling WSL2
        // rather than presenting host-process as a normal fallback.
        let caps = Capabilities { os: HostOs::Windows, ..empty_caps() };
        let out = recommend(RecommendInput::new(&caps).with_allow_tier_0(true)).unwrap();
        assert_eq!(out.strategy, IsolationStrategy::Noop);
        match out.reason {
            ResolutionReason::Downgrade { why, .. } => {
                assert!(why.contains("WSL2"), "expected a WSL2 nudge, got: {why}");
            }
            other => panic!("expected Downgrade reason, got {other:?}"),
        }
    }
}
