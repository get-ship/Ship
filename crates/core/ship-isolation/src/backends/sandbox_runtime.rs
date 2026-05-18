//! `SandboxRuntime` backend — wraps Anthropic's `@anthropic-ai/sandbox-runtime`
//! (`srt`) via a Bun subprocess.
//!
//! Schema verified against `@anthropic-ai/sandbox-runtime@0.0.51`:
//! settings JSON has the shape
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
//! CLI surface: `srt [--settings <path>] [-c <cmd>] [command...]`. We invoke
//! `bun run <srt-entry> --settings <tmp-settings> <bin> <args...>`. No `--`
//! separator — `command...` is the positional tail.
//!
//! Runtime deps the backend cannot install itself:
//! - Bun (Ship already ships this).
//! - `srt` CLI: `bun add -g @anthropic-ai/sandbox-runtime`. The backend
//!   takes an explicit path; a follow-up wires toolchain install.
//! - **On Linux:** `bubblewrap` and `socat` system packages. Without these,
//!   `srt` exits with "Sandbox dependencies not available." The backend
//!   does not probe for them — `srt` does, at run time.

use async_trait::async_trait;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;

use ship_substrate::isolation::{ExecHandle, SandboxHandle, SandboxId};
use ship_substrate::isolation::{MountMode, NetworkPolicy, SandboxPolicy};
use ship_substrate::isolation::{Isolator, IsolatorError, LaunchCommand};

#[derive(Debug, Clone)]
pub struct SandboxRuntimeConfig {
    pub bun_bin: PathBuf,
    pub srt_entry: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SandboxRuntime {
    config: SandboxRuntimeConfig,
}

impl SandboxRuntime {
    pub fn new(config: SandboxRuntimeConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SrtSettings {
    pub(crate) filesystem: SrtFsPolicy,
    pub(crate) network: SrtNetPolicy,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct SrtFsPolicy {
    #[serde(rename = "denyRead")]
    pub(crate) deny_read: Vec<String>,
    #[serde(rename = "allowRead", skip_serializing_if = "Vec::is_empty")]
    pub(crate) allow_read: Vec<String>,
    #[serde(rename = "allowWrite")]
    pub(crate) allow_write: Vec<String>,
    #[serde(rename = "denyWrite")]
    pub(crate) deny_write: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct SrtNetPolicy {
    #[serde(rename = "allowedDomains")]
    pub(crate) allowed_domains: Vec<String>,
    #[serde(rename = "deniedDomains")]
    pub(crate) denied_domains: Vec<String>,
}

pub(crate) fn render_settings(p: &SandboxPolicy) -> SrtSettings {
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

#[async_trait]
impl Isolator for SandboxRuntime {
    fn name(&self) -> &'static str {
        "sandbox-runtime"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        if !self.config.bun_bin.exists() {
            return Err(IsolatorError::Unavailable(format!(
                "bun not found at {}",
                self.config.bun_bin.display()
            )));
        }
        if !self.config.srt_entry.exists() {
            return Err(IsolatorError::Unavailable(format!(
                "sandbox-runtime entry not found at {}",
                self.config.srt_entry.display()
            )));
        }
        #[cfg(target_os = "linux")]
        {
            for tool in ["bwrap", "socat"] {
                if which_on_path(tool).is_none() {
                    return Err(IsolatorError::Unavailable(format!(
                        "linux sandbox requires `{tool}` on PATH; install with your package manager"
                    )));
                }
            }
        }
        Ok(())
    }

    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let rendered = render_settings(policy);
        let json = serde_json::to_string_pretty(&rendered)
            .map_err(|e| IsolatorError::PolicyRejected(e.to_string()))?;

        let dir = std::env::temp_dir().join(format!("ship-srt-{}", nanoid::nanoid!(8)));
        tokio::fs::create_dir_all(&dir).await?;
        let settings_path = dir.join("settings.json");
        tokio::fs::write(&settings_path, json.as_bytes()).await?;

        let root = policy
            .mounts
            .first()
            .map(|m| m.host_path.clone())
            .unwrap_or_else(std::env::temp_dir);

        Ok(SandboxHandle {
            id: SandboxId::new(),
            backend: "sandbox-runtime",
            root,
            state: Arc::new(SandboxRuntimeState { settings_path, dir }),
        })
    }

    async fn exec(
        &self,
        sandbox: &SandboxHandle,
        cmd: LaunchCommand,
    ) -> Result<ExecHandle, IsolatorError> {
        let state = sandbox
            .state
            .downcast_ref::<SandboxRuntimeState>()
            .ok_or_else(|| IsolatorError::Sidecar("sandbox state shape mismatch".into()))?;

        let mut c = Command::new(&self.config.bun_bin);
        c.arg("run")
            .arg(&self.config.srt_entry)
            .arg("--settings")
            .arg(&state.settings_path)
            .arg(&cmd.bin)
            .args(&cmd.args)
            .envs(&cmd.env)
            .current_dir(&cmd.cwd);

        let child = c.spawn().map_err(IsolatorError::from)?;
        Ok(ExecHandle {
            sandbox: sandbox.id.clone(),
            pid: child.id(),
            started_at: std::time::SystemTime::now(),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(state) = sandbox.state.downcast_ref::<SandboxRuntimeState>() {
            let _ = tokio::fs::remove_dir_all(&state.dir).await;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct SandboxRuntimeState {
    settings_path: PathBuf,
    dir: PathBuf,
}

#[cfg(target_os = "linux")]
fn which_on_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|p| p.is_file())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn cfg() -> SandboxRuntimeConfig {
        SandboxRuntimeConfig {
            bun_bin: PathBuf::from("/nonexistent/bun"),
            srt_entry: PathBuf::from("/nonexistent/srt.js"),
        }
    }

    #[tokio::test]
    async fn reports_unavailable_when_bun_missing() {
        let sb = SandboxRuntime::new(cfg());
        let err = sb.available().await.unwrap_err();
        assert!(matches!(err, IsolatorError::Unavailable(_)));
    }

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
        assert!(!s.contains("snake_case"));
        assert!(!s.contains("\"allow_write\""));
    }

    #[tokio::test]
    async fn prepare_writes_a_settings_file_with_real_schema() {
        let sb = SandboxRuntime::new(cfg());
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let h = sb.prepare(&pol).await.unwrap();
        let state = h.state.downcast_ref::<SandboxRuntimeState>().unwrap();
        let body = tokio::fs::read_to_string(&state.settings_path).await.unwrap();
        assert!(body.contains("\"allowWrite\""));
        assert!(body.contains("\"allowedDomains\""));
        sb.shutdown(h).await.unwrap();
    }

    #[tokio::test]
    async fn exec_errors_when_bun_missing() {
        let sb = SandboxRuntime::new(cfg());
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let h = sb.prepare(&pol).await.unwrap();
        let cmd = LaunchCommand {
            bin: PathBuf::from("/bin/true"),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: std::env::temp_dir(),
        };
        let err = sb.exec(&h, cmd).await.unwrap_err();
        assert!(matches!(err, IsolatorError::Io(_)));
    }
}
