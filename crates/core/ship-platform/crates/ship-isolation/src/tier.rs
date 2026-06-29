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
    /// Every tier variant, in strength order. The single authoritative
    /// enumeration — schema generators and conformance gates derive the
    /// tier id set from `ALL.map(as_id)` rather than re-listing it. A new
    /// variant must be appended here or `all_tiers_enumerated` fails.
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::Process,
        Self::Container,
        Self::Vm,
        Self::Cloud,
    ];

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
///
/// Serde uses the same short ids as `as_id()` / `from_id()` (e.g.
/// "docker", "youki") — what users type in YAML, what the CLI prints.
///
/// Tier 1 split (see `docs/strategy.md` §3.1):
///   - `HarnessNative` — harness sandboxes itself (claude `--sandbox`,
///     codex Seatbelt). Preferred where supported.
///   - `SrtWrapper` — Ship wraps with `@anthropic-ai/sandbox-runtime`
///     when the harness has no native mechanism.
///   - `HarnessSandbox` (legacy alias) — kept for backwards
///     compatibility with config files; new code uses one of the two
///     above. Recommender treats it as a request for the per-harness
///     best-fit and dispatches accordingly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IsolationStrategy {
    #[serde(rename = "noop")]
    Noop,
    /// Harness sandboxes itself; Ship passes the appropriate flag.
    #[serde(rename = "harness_native", alias = "native", alias = "harness-native")]
    HarnessNative,
    /// Ship wraps with `@anthropic-ai/sandbox-runtime`.
    #[serde(rename = "srt", alias = "srt_wrapper", alias = "sandbox-runtime", alias = "srt-wrapper")]
    SrtWrapper,
    /// Legacy: "pick the right Tier 1 for this harness." Kept so old
    /// configs don't break. New code uses HarnessNative or SrtWrapper
    /// directly. Recommender resolves to one of those via the
    /// `ship_isolation::harness::tier1_mechanism_for` table.
    #[serde(rename = "harness", alias = "harness_sandbox")]
    HarnessSandbox,
    #[serde(rename = "docker", alias = "docker_engine", alias = "podman")]
    DockerEngine,
    #[serde(rename = "youki", alias = "youki_rootless")]
    YoukiRootless,
    #[serde(rename = "wsl2", alias = "wsl")]
    Wsl2,
    #[serde(rename = "lima", alias = "lima_vm", alias = "tart", alias = "vz")]
    LimaVm,
    /// Apple Container — microVM-per-container on Apple Silicon (own kernel,
    /// stronger inter-actor isolation than a shared Lima VM). macOS 15+ arm64.
    /// Recommendable; backend deferred until built + tested on real hardware.
    #[serde(rename = "apple_container", alias = "apple", alias = "container")]
    AppleContainer,
    /// Firecracker microVM (Linux/KVM) — own-kernel per sandbox; the strong
    /// Tier-3 local path and the cloud model.
    #[serde(rename = "firecracker", alias = "fc")]
    Firecracker,
    #[serde(rename = "fly", alias = "fly_machines", alias = "flyio")]
    FlyMachines,
}

impl IsolationStrategy {
    /// Every strategy variant. The single authoritative enumeration —
    /// schema generators and conformance gates derive the strategy id set
    /// from `ALL.map(as_id)` (which yields the serde `rename` ids) rather
    /// than re-listing it. A new variant must be appended here or
    /// `all_strategies_enumerated` fails.
    pub const ALL: &'static [Self] = &[
        Self::Noop,
        Self::HarnessNative,
        Self::SrtWrapper,
        Self::HarnessSandbox,
        Self::DockerEngine,
        Self::YoukiRootless,
        Self::Wsl2,
        Self::LimaVm,
        Self::AppleContainer,
        Self::Firecracker,
        Self::FlyMachines,
    ];

    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Noop => "noop",
            Self::HarnessNative => "harness_native",
            Self::SrtWrapper => "srt",
            Self::HarnessSandbox => "harness",
            Self::DockerEngine => "docker",
            Self::YoukiRootless => "youki",
            Self::Wsl2 => "wsl2",
            Self::LimaVm => "lima",
            Self::AppleContainer => "apple_container",
            Self::Firecracker => "firecracker",
            Self::FlyMachines => "fly",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "noop" | "none" => Some(Self::Noop),
            "harness_native" | "native" | "harness-native" => Some(Self::HarnessNative),
            "srt" | "sandbox-runtime" | "srt-wrapper" | "srt_wrapper" => {
                Some(Self::SrtWrapper)
            }
            "harness" | "harness_sandbox" => Some(Self::HarnessSandbox),
            "docker" | "docker_engine" | "podman" => Some(Self::DockerEngine),
            "youki" | "youki_rootless" => Some(Self::YoukiRootless),
            "wsl2" | "wsl" => Some(Self::Wsl2),
            "lima" | "lima_vm" | "tart" | "vz" => Some(Self::LimaVm),
            "apple_container" | "apple" | "container" => Some(Self::AppleContainer),
            "firecracker" | "fc" => Some(Self::Firecracker),
            "fly" | "fly_machines" | "flyio" => Some(Self::FlyMachines),
            _ => None,
        }
    }

    pub fn tier(&self) -> IsolationTier {
        match self {
            Self::Noop => IsolationTier::None,
            Self::HarnessNative | Self::SrtWrapper | Self::HarnessSandbox => {
                IsolationTier::Process
            }
            Self::DockerEngine | Self::YoukiRootless | Self::Wsl2 => IsolationTier::Container,
            Self::LimaVm | Self::Firecracker | Self::AppleContainer => IsolationTier::Vm,
            Self::FlyMachines => IsolationTier::Cloud,
        }
    }

    /// Display-friendly label for UI surfaces. "Sandboxed by Claude"
    /// reads better than "harness_native" — but we keep the id stable
    /// for serde / scripting.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Noop => "Not sandboxed",
            Self::HarnessNative => "Harness-native sandbox",
            Self::SrtWrapper => "Process sandbox (srt)",
            Self::HarnessSandbox => "Process sandbox (auto)",
            Self::DockerEngine => "Container (Docker)",
            Self::YoukiRootless => "Container (youki)",
            Self::Wsl2 => "Container (WSL2)",
            Self::LimaVm => "Virtual machine (Lima)",
            Self::Firecracker => "microVM (Firecracker)",
            Self::AppleContainer => "microVM (Apple Container)",
            Self::FlyMachines => "Cloud (Fly)",
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
    pub policy: Option<ship_substrate::isolation::SandboxPolicy>,
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
    fn all_tiers_enumerated() {
        // `ALL` must list every variant. The exhaustive match makes adding
        // a variant a compile error here until it is appended to `ALL`, so
        // downstream schema/conformance derivations stay complete.
        for t in IsolationTier::ALL {
            match t {
                IsolationTier::None
                | IsolationTier::Process
                | IsolationTier::Container
                | IsolationTier::Vm
                | IsolationTier::Cloud => {}
            }
        }
        assert_eq!(IsolationTier::ALL.len(), 5);
    }

    #[test]
    fn all_strategies_enumerated() {
        for s in IsolationStrategy::ALL {
            match s {
                IsolationStrategy::Noop
                | IsolationStrategy::HarnessNative
                | IsolationStrategy::SrtWrapper
                | IsolationStrategy::HarnessSandbox
                | IsolationStrategy::DockerEngine
                | IsolationStrategy::YoukiRootless
                | IsolationStrategy::Wsl2
                | IsolationStrategy::LimaVm
                | IsolationStrategy::AppleContainer
                | IsolationStrategy::Firecracker
                | IsolationStrategy::FlyMachines => {}
            }
        }
        assert_eq!(IsolationStrategy::ALL.len(), 11);
        // `as_id()` yields the serde `rename` id — pin a couple so the
        // strategy enum set the schema mirrors stays the serialized form.
        assert_eq!(IsolationStrategy::SrtWrapper.as_id(), "srt");
        assert_eq!(IsolationStrategy::HarnessSandbox.as_id(), "harness");
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
        // Post-split: "sandbox-runtime" maps to SrtWrapper (the new
        // canonical name). Legacy "harness" stays bound to
        // HarnessSandbox so old configs don't surprise users.
        assert_eq!(
            IsolationStrategy::from_id("sandbox-runtime"),
            Some(IsolationStrategy::SrtWrapper)
        );
        assert_eq!(
            IsolationStrategy::from_id("native"),
            Some(IsolationStrategy::HarnessNative)
        );
        assert_eq!(
            IsolationStrategy::from_id("harness"),
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
