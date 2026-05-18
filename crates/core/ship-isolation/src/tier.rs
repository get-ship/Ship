//! Isolation tiers + strategies.
//!
//! Per `docs/strategy.md`. Users pick a tier; the recommender picks a
//! strategy within it unless the user pins one.
//!
//! Naming: `IsolationTier` — not just `Tier` — to avoid collision with the
//! existing `permissions_tier` (standard/autonomous/read-only) on agents.
//! The two compose orthogonally: permissions = what the agent may attempt;
//! isolation = where the agent runs.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Where an agent runs. Higher numbers = stronger isolation.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum IsolationTier {
    /// Host process. No sandbox. The agent is the user.
    #[default]
    None,
    /// Host process with a per-process FS + syscall sandbox.
    /// (Seatbelt / bubblewrap / harness-native.)
    Process,
    /// OCI container — separate FS, separate netns, cgroup limits,
    /// host kernel.
    Container,
    /// MicroVM or full VM — separate kernel.
    Vm,
    /// Vendor-served sandbox. Sidecar runs in vendor's container/VM;
    /// host is untouched.
    Cloud,
}

impl IsolationTier {
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Process => "process",
            Self::Container => "container",
            Self::Vm => "vm",
            Self::Cloud => "cloud",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "none" | "0" => Some(Self::None),
            "process" | "1" => Some(Self::Process),
            "container" | "2" => Some(Self::Container),
            "vm" | "3" => Some(Self::Vm),
            "cloud" | "4" => Some(Self::Cloud),
            _ => None,
        }
    }

    /// Human-friendly label for UI chips. Never "Tier N" alone.
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "Not sandboxed",
            Self::Process => "Process sandbox",
            Self::Container => "Container",
            Self::Vm => "Virtual machine",
            Self::Cloud => "Cloud sandbox",
        }
    }
}

impl fmt::Display for IsolationTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_id())
    }
}

/// One concrete implementation of a tier. Each strategy implements
/// `Isolator`. Strategies are capability-gated by the recommender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationStrategy {
    /// Tier 0 — baseline. Spawns unsandboxed.
    Noop,
    /// Tier 1 — wraps `@anthropic-ai/sandbox-runtime` (Seatbelt or bwrap).
    HarnessSandbox,
    /// Tier 2 — host `docker` or `podman`.
    DockerEngine,
    /// Tier 2 — bundled `youki` + Ubuntu rootfs.
    YoukiRootless,
    /// Tier 2 — host WSL2 distro (Windows path when WSL2 present).
    Wsl2,
    /// Tier 3 — Lima/Vz on macOS arm64.
    LimaVm,
    /// Tier 4 — Fly Machines.
    FlyMachines,
}

impl IsolationStrategy {
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Noop => "noop",
            Self::HarnessSandbox => "harness",
            Self::DockerEngine => "docker",
            Self::YoukiRootless => "youki",
            Self::Wsl2 => "wsl2",
            Self::LimaVm => "lima",
            Self::FlyMachines => "fly",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "noop" | "none" => Some(Self::Noop),
            "harness" | "srt" | "sandbox-runtime" => Some(Self::HarnessSandbox),
            "docker" | "podman" => Some(Self::DockerEngine),
            "youki" => Some(Self::YoukiRootless),
            "wsl2" | "wsl" => Some(Self::Wsl2),
            "lima" | "tart" | "vz" => Some(Self::LimaVm),
            "fly" | "fly_machines" | "flyio" => Some(Self::FlyMachines),
            _ => None,
        }
    }

    pub fn tier(&self) -> IsolationTier {
        match self {
            Self::Noop => IsolationTier::None,
            Self::HarnessSandbox => IsolationTier::Process,
            Self::DockerEngine | Self::YoukiRootless | Self::Wsl2 => IsolationTier::Container,
            Self::LimaVm => IsolationTier::Vm,
            Self::FlyMachines => IsolationTier::Cloud,
        }
    }
}

impl fmt::Display for IsolationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_id())
    }
}

/// User-facing profile that bundles a tier choice with overrides. May be
/// embedded in an agent definition, the per-run override, or the user
/// defaults. Narrowest wins (see strategy.md §7).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsolationProfile {
    /// Preferred tier. `None` means inherit from the next level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<IsolationTier>,
    /// Optional pin within the tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<IsolationStrategy>,
    /// Optional policy override (mounts, network, limits).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<crate::policy::SandboxPolicy>,
}

impl IsolationProfile {
    pub fn pin_tier(tier: IsolationTier) -> Self {
        Self { tier: Some(tier), ..Default::default() }
    }

    pub fn pin_strategy(strategy: IsolationStrategy) -> Self {
        Self {
            tier: Some(strategy.tier()),
            strategy: Some(strategy),
            policy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_ordering_is_strength() {
        assert!(IsolationTier::None < IsolationTier::Process);
        assert!(IsolationTier::Process < IsolationTier::Container);
        assert!(IsolationTier::Container < IsolationTier::Vm);
        assert!(IsolationTier::Vm < IsolationTier::Cloud);
    }

    #[test]
    fn tier_id_roundtrip() {
        for t in [
            IsolationTier::None,
            IsolationTier::Process,
            IsolationTier::Container,
            IsolationTier::Vm,
            IsolationTier::Cloud,
        ] {
            assert_eq!(IsolationTier::from_id(t.as_id()), Some(t));
        }
        assert_eq!(IsolationTier::from_id("2"), Some(IsolationTier::Container));
        assert_eq!(IsolationTier::from_id("bogus"), None);
    }

    #[test]
    fn strategy_tier_mapping() {
        assert_eq!(IsolationStrategy::Noop.tier(), IsolationTier::None);
        assert_eq!(IsolationStrategy::HarnessSandbox.tier(), IsolationTier::Process);
        assert_eq!(IsolationStrategy::DockerEngine.tier(), IsolationTier::Container);
        assert_eq!(IsolationStrategy::YoukiRootless.tier(), IsolationTier::Container);
        assert_eq!(IsolationStrategy::Wsl2.tier(), IsolationTier::Container);
        assert_eq!(IsolationStrategy::LimaVm.tier(), IsolationTier::Vm);
        assert_eq!(IsolationStrategy::FlyMachines.tier(), IsolationTier::Cloud);
    }

    #[test]
    fn strategy_aliases_resolve() {
        assert_eq!(
            IsolationStrategy::from_id("sandbox-runtime"),
            Some(IsolationStrategy::HarnessSandbox)
        );
        assert_eq!(
            IsolationStrategy::from_id("podman"),
            Some(IsolationStrategy::DockerEngine)
        );
        assert_eq!(
            IsolationStrategy::from_id("vz"),
            Some(IsolationStrategy::LimaVm)
        );
    }

    #[test]
    fn pin_strategy_carries_tier() {
        let p = IsolationProfile::pin_strategy(IsolationStrategy::DockerEngine);
        assert_eq!(p.tier, Some(IsolationTier::Container));
        assert_eq!(p.strategy, Some(IsolationStrategy::DockerEngine));
    }

    #[test]
    fn profile_serde_skips_none() {
        let p = IsolationProfile::pin_tier(IsolationTier::Container);
        let s = serde_json::to_string(&p).unwrap();
        // tier present, strategy + policy omitted
        assert!(s.contains("container"));
        assert!(!s.contains("strategy"));
        assert!(!s.contains("policy"));
    }
}
