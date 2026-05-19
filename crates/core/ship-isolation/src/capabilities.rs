//! Capability detection — what isolation primitives are usable on this
//! host right now?
//!
//! Per `docs/strategy.md` §5. The probe is cheap (a handful of
//! `which`-style lookups + cheap subprocess version queries) and the
//! result is meant to be cached on the daemon. Calling `Capabilities::
//! probe()` returns the snapshot synchronously; orchestrating callers
//! cache + invalidate.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::tier::{IsolationStrategy, IsolationTier};

/// Detected host capabilities. JSON-serializable so it can be stored in
/// `daemon_settings` and round-tripped to the UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub os: HostOs,
    /// `true` if we appear to already be running inside someone else's
    /// sandbox (CI container, codespace, Docker, etc.). Recommender
    /// treats this as "Tier 0; don't double-wrap" (per strategy.md §4).
    pub in_sandbox: bool,
    pub docker: bool,
    pub podman: bool,
    pub youki: Option<PathBuf>,
    pub youki_rootfs: Option<PathBuf>,
    pub lima: Option<PathBuf>,
    pub tart: Option<PathBuf>,
    pub wsl2: bool,
    pub srt: bool,
    pub sandbox_exec: bool,
    pub vz_framework: bool,
    pub fly_cli: bool,
    pub fly_authed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostOs {
    Linux,
    MacosArm,
    MacosIntel,
    Windows,
    Wsl2,
    #[default]
    Unknown,
}

impl HostOs {
    pub fn detect() -> Self {
        if cfg!(target_os = "linux") {
            if std::env::var("WSL_DISTRO_NAME").is_ok()
                || std::fs::read_to_string("/proc/version")
                    .map(|s| s.to_lowercase().contains("microsoft"))
                    .unwrap_or(false)
            {
                Self::Wsl2
            } else {
                Self::Linux
            }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                Self::MacosArm
            } else {
                Self::MacosIntel
            }
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unknown
        }
    }
}

impl Capabilities {
    /// Run the full probe. Synchronous; the underlying commands have
    /// short bounded runtimes (a few ms each in the happy path; ~1s if
    /// `docker version` is sluggish). Callers should cache the result.
    pub fn probe() -> Self {
        let os = HostOs::detect();
        Self {
            os,
            in_sandbox: detect_in_sandbox(),
            docker: probe_command(&["docker", "version", "--format", "{{.Server.Version}}"]),
            podman: probe_command(&["podman", "version", "--format", "{{.Server.Version}}"]),
            youki: probe_path("youki").or_else(default_youki_path),
            youki_rootfs: default_youki_rootfs_if_exists(),
            lima: probe_path("limactl").or_else(|| probe_path("lima")),
            tart: probe_path("tart"),
            wsl2: probe_wsl2(os),
            srt: probe_srt(),
            sandbox_exec: probe_path("sandbox-exec").is_some(),
            vz_framework: matches!(os, HostOs::MacosArm),
            fly_cli: probe_path("flyctl").is_some() || probe_path("fly").is_some(),
            fly_authed: probe_fly_authed(),
        }
    }

    /// Which strategies are usable on this host? Order-independent.
    ///
    /// `HarnessNative` is always listed for non-Windows hosts because
    /// availability is per-harness, not per-host — the recommender's
    /// adapter table decides at agent-run time. The Capabilities probe
    /// stays host-level; per-harness probing happens in
    /// `harness::tier1_mechanism_for` driven by the resolved agent's
    /// harness id.
    pub fn available_strategies(&self) -> Vec<IsolationStrategy> {
        let mut out = vec![IsolationStrategy::Noop];
        // HarnessNative — at least some harness on this OS can sandbox
        // itself; the recommender narrows per agent.
        if !matches!(self.os, HostOs::Windows | HostOs::Unknown) {
            out.push(IsolationStrategy::HarnessNative);
        }
        if self.srt {
            out.push(IsolationStrategy::SrtWrapper);
            // Legacy alias: HarnessSandbox is the "pick whichever Tier 1
            // works for this harness" strategy. Listed when at least one
            // Tier 1 mechanism is reachable.
            out.push(IsolationStrategy::HarnessSandbox);
        } else if !matches!(self.os, HostOs::Windows | HostOs::Unknown) {
            // Even without srt, HarnessSandbox is reachable via
            // HarnessNative path for harnesses that support it.
            out.push(IsolationStrategy::HarnessSandbox);
        }
        if self.docker || self.podman {
            out.push(IsolationStrategy::DockerEngine);
        }
        if self.youki.is_some() && self.youki_rootfs.is_some() && !matches!(self.os, HostOs::Windows) {
            out.push(IsolationStrategy::YoukiRootless);
        }
        if self.wsl2 {
            out.push(IsolationStrategy::Wsl2);
        }
        if self.lima.is_some() && matches!(self.os, HostOs::MacosArm) {
            out.push(IsolationStrategy::LimaVm);
        }
        if self.fly_cli && self.fly_authed {
            out.push(IsolationStrategy::FlyMachines);
        }
        out
    }

    /// Which tiers have at least one usable strategy?
    pub fn available_tiers(&self) -> Vec<IsolationTier> {
        let mut tiers: Vec<IsolationTier> = self
            .available_strategies()
            .into_iter()
            .map(|s| s.tier())
            .collect();
        tiers.sort();
        tiers.dedup();
        tiers
    }
}

fn detect_in_sandbox() -> bool {
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }
    if std::env::var("CODESPACES").is_ok() {
        return true;
    }
    if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return true;
    }
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        return true;
    }
    if let Ok(contents) = std::fs::read_to_string("/proc/1/cgroup") {
        if contents.contains("/docker/") || contents.contains("/kubepods/") {
            return true;
        }
    }
    false
}

fn probe_command(argv: &[&str]) -> bool {
    let Some((bin, rest)) = argv.split_first() else { return false };
    Command::new(bin)
        .args(rest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn probe_path(bin: &str) -> Option<PathBuf> {
    let which = if cfg!(target_os = "windows") { "where" } else { "which" };
    let out = Command::new(which).arg(bin).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let first = s.lines().next()?.trim();
    if first.is_empty() {
        return None;
    }
    Some(PathBuf::from(first))
}

fn default_youki_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("SHIP_YOUKI_BIN") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    let candidate = PathBuf::from("/tmp/youki");
    if candidate.is_file() {
        return Some(candidate);
    }
    None
}

fn default_youki_rootfs_if_exists() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("SHIP_AGENT_ROOTFS") {
        let p = PathBuf::from(p);
        if p.is_dir() {
            return Some(p);
        }
    }
    let candidate = PathBuf::from("/tmp/ship-rootfs/rootfs");
    if candidate.is_dir() {
        return Some(candidate);
    }
    None
}

fn probe_wsl2(os: HostOs) -> bool {
    if !matches!(os, HostOs::Windows) {
        return false;
    }
    Command::new("wsl")
        .args(["--status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn probe_srt() -> bool {
    if probe_path("bun").is_none() {
        return false;
    }
    if let Ok(home) = std::env::var("HOME") {
        let probe = PathBuf::from(home)
            .join(".bun/install/global/node_modules/@anthropic-ai/sandbox-runtime");
        if probe.is_dir() {
            return true;
        }
    }
    false
}

fn probe_fly_authed() -> bool {
    let bin = if probe_path("flyctl").is_some() {
        "flyctl"
    } else if probe_path("fly").is_some() {
        "fly"
    } else {
        return false;
    };
    Command::new(bin)
        .args(["auth", "whoami"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_os_detect_returns_something_known_on_this_host() {
        let os = HostOs::detect();
        assert_ne!(os, HostOs::Unknown);
    }

    #[test]
    fn probe_runs_without_panicking() {
        let _ = Capabilities::probe();
    }

    #[test]
    fn noop_always_available() {
        let caps = Capabilities::default();
        assert!(caps.available_strategies().contains(&IsolationStrategy::Noop));
    }

    #[test]
    fn no_strategies_implies_only_tier_none() {
        let caps = Capabilities::default();
        let tiers = caps.available_tiers();
        assert_eq!(tiers, vec![IsolationTier::None]);
    }

    #[test]
    fn youki_requires_both_bin_and_rootfs() {
        let caps = Capabilities {
            os: HostOs::Linux,
            youki: Some(PathBuf::from("/tmp/youki")),
            ..Capabilities::default()
        };
        assert!(!caps
            .available_strategies()
            .contains(&IsolationStrategy::YoukiRootless));
        let caps = Capabilities {
            youki_rootfs: Some(PathBuf::from("/tmp/ship-rootfs/rootfs")),
            ..caps
        };
        assert!(caps
            .available_strategies()
            .contains(&IsolationStrategy::YoukiRootless));
    }

    #[test]
    fn fly_requires_cli_and_auth() {
        let caps = Capabilities { fly_cli: true, ..Capabilities::default() };
        assert!(!caps
            .available_strategies()
            .contains(&IsolationStrategy::FlyMachines));
        let caps = Capabilities { fly_authed: true, ..caps };
        assert!(caps
            .available_strategies()
            .contains(&IsolationStrategy::FlyMachines));
    }
}
