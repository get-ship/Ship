//! Per-strategy configuration blob.
//!
//! The matrix is genuinely complex — each strategy has its own knobs
//! (docker image, fly org, youki rootfs path, lima instance, etc.).
//! Stuffing every one of those as a flat `isolation.docker_image`
//! setting key explodes the namespace. Instead we keep ONE nested
//! JSON blob under `isolation.strategies` and parse it into typed
//! sub-structs.
//!
//! User-facing globals (`isolation.default_tier`,
//! `isolation.default_strategy`, `isolation.enforce`,
//! `isolation.allow_tier_0`) stay flat — they're orthogonal and
//! conceptually different (preferences vs strategy implementations).
//!
//! Migration: this is additive. The flat keys we shipped in Phase A
//! aren't widely persisted in practice (the branch is days old); we
//! cut over without a migration path. If we later need one, write it
//! against this nested shape.

use serde::{Deserialize, Serialize};

/// Top-level settings blob. Stored as JSON at the daemon-settings key
/// `isolation.strategies`. Every field is `Option` so the user only
/// has to override what they care about; defaults come from
/// `Default::default()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct StrategiesSettings {
    pub docker: DockerSettings,
    pub youki: YoukiSettings,
    pub harness_native: HarnessNativeSettings,
    pub srt: SrtSettings,
    pub lima: LimaSettings,
    pub wsl: WslSettings,
    pub firecracker: FirecrackerSettings,
    pub fly: FlySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DockerSettings {
    /// `docker` or `podman` or absolute path.
    pub engine_bin: String,
    /// Container image. Our published image bakes harness CLIs +
    /// sidecar. Power users can override with their own.
    pub image: String,
    /// Container name prefix (`ship-agent-<slug>-<nanoid>`).
    pub name_prefix: String,
    /// Resource caps applied to the container. None per field means
    /// "no limit"; defaults pick conservative values appropriate for
    /// a typical agent workload. See threat-model.md §7.3.
    pub resources: DockerResources,
    /// Host path to the `ship-sidecar` binary bind-mounted into the
    /// container. Empty = auto-resolve (next to the daemon binary,
    /// then `/usr/local/bin/ship-sidecar`). The container runs this
    /// as PID 1 and the supervisor drives actors over ship.sidecar.v1.
    pub sidecar_bin: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DockerResources {
    /// Memory cap in bytes. Default 4 GiB.
    pub memory_bytes: Option<u64>,
    /// CPU quota in microseconds per period. Default = 2 cpus
    /// (`2_000_000` μs per `1_000_000` μs period; the backend
    /// converts to docker's `--cpus` decimal form).
    pub cpu_quota_micros: Option<u64>,
    /// Max process count. Default 1024.
    pub pids_max: Option<u32>,
}

impl Default for DockerSettings {
    fn default() -> Self {
        Self {
            engine_bin: "docker".into(),
            image: "ghcr.io/ship/agent-runner:latest".into(),
            name_prefix: "ship-agent".into(),
            resources: DockerResources::default(),
            sidecar_bin: String::new(),
        }
    }
}

impl Default for DockerResources {
    fn default() -> Self {
        Self {
            memory_bytes: Some(4 * 1024 * 1024 * 1024),
            cpu_quota_micros: Some(2_000_000),
            pids_max: Some(1024),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct YoukiSettings {
    /// Path to the youki binary.
    pub bin: String,
    /// Path to the agent rootfs.
    pub rootfs: String,
    /// Path to the spec template (`youki spec --rootless` output).
    pub spec_template: String,
    /// Where youki keeps per-container state.
    pub state_root: String,
}

impl Default for YoukiSettings {
    fn default() -> Self {
        Self {
            bin: "/tmp/youki".into(),
            rootfs: "/tmp/ship-rootfs/rootfs".into(),
            spec_template: "/tmp/claude-bundle/config.json".into(),
            state_root: "/tmp/ship-youki-state".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HarnessNativeSettings {
    /// Per-harness flag overrides. Keys are harness ids
    /// (`claude`, `codex`, ...); values override the default from
    /// `harness::tier1_mechanism_for`. Use sparingly — the table is
    /// usually right; this exists for users who've patched their
    /// harness to a fork with different flags.
    pub flag_overrides: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SrtSettings {
    /// Override path to `bun` (`which bun` is the default).
    pub bun_bin: Option<String>,
    /// Override path to the srt entry point in the global bun install.
    pub srt_entry: Option<String>,
}

impl Default for SrtSettings {
    fn default() -> Self {
        Self {
            bun_bin: None,
            srt_entry: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LimaSettings {
    /// Path to `limactl` (defaults to `limactl` on PATH).
    pub bin: String,
    /// Instance name. The same instance is reused across agent runs;
    /// destroying it requires `limactl delete`.
    pub instance: String,
    /// Absolute path to the sidecar binary inside the Lima instance (baked
    /// into the Ship substrate image).
    pub sidecar_path: String,
}

impl Default for LimaSettings {
    fn default() -> Self {
        Self {
            bin: "limactl".into(),
            instance: "ship-default".into(),
            sidecar_path: "/usr/bin/ship-sidecar".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WslSettings {
    /// Path to `wsl.exe` (defaults to `wsl.exe` on PATH).
    pub bin: String,
    /// The dedicated Ship substrate distro — imported from a published
    /// rootfs; reused across runs. NEVER the user's own distro.
    pub distro: String,
    /// Absolute path to the sidecar binary inside the distro (baked into the
    /// substrate rootfs).
    pub sidecar_path: String,
}

impl Default for WslSettings {
    fn default() -> Self {
        Self {
            bin: "wsl.exe".into(),
            distro: "ship-substrate".into(),
            sidecar_path: "/usr/bin/ship-sidecar".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FirecrackerSettings {
    /// Path to the `firecracker` binary. Empty = auto-resolve (next to the
    /// daemon binary, then `firecracker` on PATH).
    pub bin: String,
    /// Guest kernel (`vmlinux`). Empty until a pinned/signed image ships
    /// (Phase F); the Isolator's `available()` refuses if absent.
    pub kernel: String,
    /// Pre-baked rootfs (ext4) with `ship-sidecar` as init on vsock.
    pub rootfs: String,
    /// Resource caps (mem/cpu) for the microVM. Reuses the docker shape.
    pub resources: DockerResources,
}

impl Default for FirecrackerSettings {
    fn default() -> Self {
        Self {
            bin: String::new(),
            kernel: String::new(),
            rootfs: String::new(),
            resources: DockerResources::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FlySettings {
    /// Fly.io organisation slug. Required for Tier 4.
    pub org: String,
    /// Fly app name. Default `ship-agents`.
    pub app: String,
    /// Machine size hint (e.g. "shared-cpu-2x").
    pub machine_size: String,
    /// Region hint (e.g. "ord", "lhr").
    pub region: Option<String>,
}

impl Default for FlySettings {
    fn default() -> Self {
        Self {
            org: String::new(),
            app: "ship-agents".into(),
            machine_size: "shared-cpu-2x".into(),
            region: None,
        }
    }
}

/// Daemon-settings key for the JSON blob.
pub const SETTINGS_KEY: &str = "isolation.strategies";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_serialize_to_valid_json() {
        let s = StrategiesSettings::default();
        let json = serde_json::to_value(&s).unwrap();
        assert!(json.is_object());
        assert!(json.get("docker").is_some());
        assert!(json.get("fly").is_some());
    }

    #[test]
    fn partial_blob_inherits_defaults_for_missing_fields() {
        // Only docker.image set; everything else defaults.
        let raw = serde_json::json!({
            "docker": { "image": "my-org/agent:1.0" }
        });
        let s: StrategiesSettings = serde_json::from_value(raw).unwrap();
        assert_eq!(s.docker.image, "my-org/agent:1.0");
        // Untouched defaults still present.
        assert_eq!(s.docker.engine_bin, "docker");
        assert_eq!(s.fly.app, "ship-agents");
    }

    #[test]
    fn empty_blob_yields_all_defaults() {
        let raw = serde_json::json!({});
        let s: StrategiesSettings = serde_json::from_value(raw).unwrap();
        assert_eq!(s, StrategiesSettings::default());
    }

    #[test]
    fn flag_override_for_harness_native() {
        let raw = serde_json::json!({
            "harness_native": {
                "flag_overrides": { "claude": "--unsafe-sandbox-mode" }
            }
        });
        let s: StrategiesSettings = serde_json::from_value(raw).unwrap();
        assert_eq!(
            s.harness_native.flag_overrides.get("claude"),
            Some(&"--unsafe-sandbox-mode".to_string())
        );
    }

    #[test]
    fn round_trip_through_json() {
        let mut s = StrategiesSettings::default();
        s.fly.org = "my-org".into();
        s.fly.region = Some("ord".into());
        s.docker.image = "private.registry/agent:2026.05".into();
        let json = serde_json::to_value(&s).unwrap();
        let back: StrategiesSettings = serde_json::from_value(json).unwrap();
        assert_eq!(back, s);
    }
}
