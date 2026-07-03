//! `srt` settings rendering — the typed model for Anthropic's
//! `@anthropic-ai/sandbox-runtime` config JSON.
//!
//! This is **not a backend.** Tier 1 (host OS-level srt/bubblewrap) runs
//! as an argv transformation at spawn (`settings::wrap_launch` +
//! `spawn_process`), not through the container-shaped `Isolator` trait —
//! there is no container to `prepare`/`shutdown`. What's shared across
//! the Tier-1 path is the *settings rendering*: turn a `SandboxPolicy`
//! into the JSON `srt --settings` expects. That's this module.
//!
//! Schema verified against `@anthropic-ai/sandbox-runtime@0.0.51`:
//!
//! ```jsonc
//! {
//!   "filesystem": { "denyRead": [], "allowWrite": ["."], "denyWrite": [] },
//!   "network":    { "allowedDomains": [], "deniedDomains": [] }
//! }
//! ```
//!
//! Network is allow-only — an empty `allowedDomains` means no network.
//! Filesystem write is allow-only; read is deny-then-allow (broad denies,
//! then specific re-allows; `allowRead` overrides `denyRead`).
//!
//! Runtime deps (Linux): `bubblewrap` + `socat`; `srt` probes for them at
//! run time and exits "Sandbox dependencies not available" if absent.

use serde::Serialize;

use isolation_port::{MountMode, NetworkPolicy, SandboxPolicy};

#[derive(Debug, Serialize)]
pub struct SrtSettings {
    pub filesystem: SrtFsPolicy,
    pub network: SrtNetPolicy,
}

#[derive(Debug, Serialize, Default)]
pub struct SrtFsPolicy {
    #[serde(rename = "denyRead")]
    pub deny_read: Vec<String>,
    #[serde(rename = "allowRead", skip_serializing_if = "Vec::is_empty")]
    pub allow_read: Vec<String>,
    #[serde(rename = "allowWrite")]
    pub allow_write: Vec<String>,
    #[serde(rename = "denyWrite")]
    pub deny_write: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct SrtNetPolicy {
    #[serde(rename = "allowedDomains")]
    pub allowed_domains: Vec<String>,
    #[serde(rename = "deniedDomains")]
    pub denied_domains: Vec<String>,
}

/// Render a `SandboxPolicy` into the `srt` settings JSON model. The single
/// source of truth for srt config — the Tier-1 spawn path serializes this
/// and passes it via `srt --settings`.
pub fn render_settings(p: &SandboxPolicy) -> SrtSettings {
    let mut fs = SrtFsPolicy::default();
    for m in &p.mounts {
        let s = m.host_path.display().to_string();
        match m.mode {
            MountMode::ReadWrite => fs.allow_write.push(s),
            MountMode::ReadOnly => fs.allow_read.push(s),
        }
    }
    let network = match &p.network {
        NetworkPolicy::DenyAll | NetworkPolicy::Proxy { .. } => SrtNetPolicy::default(),
        NetworkPolicy::Allowlist { hosts } => SrtNetPolicy {
            allowed_domains: hosts.clone(),
            denied_domains: Vec::new(),
        },
        NetworkPolicy::Open => SrtNetPolicy {
            allowed_domains: vec!["*".into()],
            denied_domains: Vec::new(),
        },
    };
    SrtSettings { filesystem: fs, network }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn renders_empty_allowed_domains_for_deny_all() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let rendered = render_settings(&pol);
        assert!(rendered.network.allowed_domains.is_empty());
        assert_eq!(rendered.filesystem.allow_write, vec!["/tmp/x".to_string()]);
    }

    #[test]
    fn renders_allowlist_hosts_into_allowed_domains() {
        let mut pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        pol.network = NetworkPolicy::Allowlist {
            hosts: vec!["api.anthropic.com".into()],
        };
        let rendered = render_settings(&pol);
        assert_eq!(rendered.network.allowed_domains, vec!["api.anthropic.com"]);
    }

    #[test]
    fn serializes_to_srt_settings_schema() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let rendered = render_settings(&pol);
        let s = serde_json::to_string(&rendered).unwrap();
        assert!(s.contains("\"filesystem\""));
        assert!(s.contains("\"allowWrite\""));
        assert!(s.contains("\"allowedDomains\""));
        assert!(s.contains("\"deniedDomains\""));
        assert!(!s.contains("\"allow_write\""));
    }
}
