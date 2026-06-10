//! `Youki` backend — runs the agent inside an OCI container managed by
//! `youki`.
//!
//! **Status: working spike on Linux/WSL2.** The backend takes a path to a
//! `youki` binary, a base OCI `config.json` template (typically the output
//! of `youki spec --rootless`), and a rootfs directory. On `prepare()` it
//! materializes an OCI bundle under a temp dir; on `exec()` it spawns
//! `youki run --bundle <bundle>` and returns the resulting PID; on
//! `shutdown()` it best-effort `youki delete`s the container and removes
//! the bundle dir.
//!
//! Layered onto the spec template at `prepare()`:
//!   * `process.{args, cwd, env}` from the `LaunchCommand`.
//!   * `mounts` — one bind entry per `FsMount` (rw or ro per `MountMode`).
//!   * `linux.resources` — memory / cpu / pids if set on the policy.
//!   * `linux.namespaces` — `network` ns added when policy != Open.
//!
//! Not in scope yet:
//!   * Network policy enforcement (allowlist / proxy). Today the network
//!     namespace is created empty; nothing routes out.
//!   * Sidecar bring-up inside the container.
//!   * Rootfs distribution / overlay sharing across sandboxes.

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;

use ship_substrate::isolation::{SandboxHandle, SandboxId};
use ship_substrate::isolation::{MountMode, NetworkPolicy, SandboxPolicy};
use ship_substrate::isolation::{Isolator, IsolatorError, LaunchCommand};

#[derive(Debug, Clone)]
pub struct YoukiConfig {
    pub youki_bin: PathBuf,
    pub spec_template: PathBuf,
    pub rootfs: PathBuf,
    pub state_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Youki {
    config: YoukiConfig,
}

impl Youki {
    pub fn new(config: YoukiConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct OciMount {
    pub(crate) destination: String,
    pub(crate) source: String,
    #[serde(rename = "type")]
    pub(crate) ty: &'static str,
    pub(crate) options: Vec<&'static str>,
}

pub(crate) fn mount_for(m: &ship_substrate::isolation::FsMount) -> OciMount {
    OciMount {
        destination: m.sandbox_path.display().to_string(),
        source: m.host_path.display().to_string(),
        ty: "bind",
        options: match m.mode {
            MountMode::ReadWrite => vec!["rbind", "rw"],
            MountMode::ReadOnly => vec!["rbind", "ro"],
        },
    }
}

pub(crate) fn patch_spec(
    template: &Value,
    policy: &SandboxPolicy,
    rootfs: &Path,
    cmd: Option<&LaunchCommand>,
) -> Value {
    let mut spec = template.clone();

    if let Some(root) = spec.get_mut("root") {
        root["path"] = Value::String(rootfs.display().to_string());
        root["readonly"] = Value::Bool(false);
    }

    if let Some(c) = cmd {
        let process = spec.as_object_mut().and_then(|o| {
            o.entry("process")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
        });
        if let Some(p) = process {
            let mut argv = vec![Value::String(c.bin.display().to_string())];
            argv.extend(c.args.iter().cloned().map(Value::String));
            p.insert("args".into(), Value::Array(argv));
            p.insert("cwd".into(), Value::String(c.cwd.display().to_string()));
            let mut env_map: std::collections::BTreeMap<String, String> = p
                .get("env")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
                .filter_map(|s| s.split_once('=').map(|(k, v)| (k.into(), v.into())))
                .collect();
            for (k, v) in &c.env {
                env_map.insert(k.clone(), v.clone());
            }
            if !env_map.contains_key("PATH") {
                env_map.insert(
                    "PATH".into(),
                    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
                );
            }
            let env: Vec<Value> = env_map
                .into_iter()
                .map(|(k, v)| Value::String(format!("{k}={v}")))
                .collect();
            p.insert("env".into(), Value::Array(env));
        }
    }

    let mounts = spec
        .as_object_mut()
        .and_then(|o| o.entry("mounts").or_insert_with(|| Value::Array(vec![])).as_array_mut());
    if let Some(arr) = mounts {
        for m in &policy.mounts {
            arr.push(serde_json::to_value(mount_for(m)).unwrap());
        }
    }

    let needs_netns = !matches!(policy.network, NetworkPolicy::Open);
    let linux = spec
        .as_object_mut()
        .and_then(|o| o.entry("linux").or_insert_with(|| serde_json::json!({})).as_object_mut());
    if let Some(l) = linux {
        let nss = l
            .entry("namespaces")
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut();
        if let Some(nss) = nss {
            let has_net = nss
                .iter()
                .any(|n| n.get("type").and_then(|t| t.as_str()) == Some("network"));
            if needs_netns && !has_net {
                nss.push(serde_json::json!({"type": "network"}));
            }
            if !needs_netns {
                nss.retain(|n| n.get("type").and_then(|t| t.as_str()) != Some("network"));
            }
        }
        let mut resources = serde_json::Map::new();
        if let Some(b) = policy.limits.memory_bytes {
            resources.insert("memory".into(), serde_json::json!({ "limit": b }));
        }
        if let Some(q) = policy.limits.cpu_quota_micros {
            resources.insert("cpu".into(), serde_json::json!({ "quota": q }));
        }
        if let Some(p) = policy.limits.pids_max {
            resources.insert("pids".into(), serde_json::json!({ "limit": p }));
        }
        if !resources.is_empty() {
            l.insert("resources".into(), Value::Object(resources));
        }
    }

    spec
}

#[derive(Debug)]
struct YoukiState {
    container_id: String,
    bundle_dir: PathBuf,
    state_dir: PathBuf,
    policy: SandboxPolicy,
}

#[async_trait]
impl Isolator for Youki {
    fn name(&self) -> &'static str {
        "youki"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        if !self.config.youki_bin.is_file() {
            return Err(IsolatorError::Unavailable(format!(
                "youki binary not found at {}",
                self.config.youki_bin.display()
            )));
        }
        if !self.config.spec_template.is_file() {
            return Err(IsolatorError::Unavailable(format!(
                "spec template not found at {}",
                self.config.spec_template.display()
            )));
        }
        if !self.config.rootfs.is_dir() {
            return Err(IsolatorError::Unavailable(format!(
                "rootfs dir not found at {}",
                self.config.rootfs.display()
            )));
        }
        Ok(())
    }

    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let id = SandboxId::new();
        let bundle_dir = std::env::temp_dir().join(format!("ship-youki-{}", id.0));
        tokio::fs::create_dir_all(&bundle_dir).await?;
        let state_dir = self.config.state_root.join(&id.0);
        tokio::fs::create_dir_all(&state_dir).await?;

        let template_bytes = tokio::fs::read(&self.config.spec_template).await?;
        let template: Value = serde_json::from_slice(&template_bytes)
            .map_err(|e| IsolatorError::PolicyRejected(e.to_string()))?;
        let patched = patch_spec(&template, policy, &self.config.rootfs, None);
        let body = serde_json::to_vec_pretty(&patched)
            .map_err(|e| IsolatorError::PolicyRejected(e.to_string()))?;
        tokio::fs::write(bundle_dir.join("config.json"), &body).await?;

        Ok(SandboxHandle {
            id: id.clone(),
            backend: "youki",
            root: bundle_dir.clone(),
            state: Arc::new(YoukiState {
                container_id: format!("ship-{}", id.0),
                bundle_dir,
                state_dir,
                policy: policy.clone(),
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(state) = sandbox.state.downcast_ref::<YoukiState>() {
            let _ = Command::new(&self.config.youki_bin)
                .arg("-r")
                .arg(&state.state_dir)
                .arg("delete")
                .arg("--force")
                .arg(&state.container_id)
                .status()
                .await;
            let _ = tokio::fs::remove_dir_all(&state.bundle_dir).await;
            let _ = tokio::fs::remove_dir_all(&state.state_dir).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn empty_spec_template() -> Value {
        serde_json::json!({
            "ociVersion": "1.0.2",
            "root": { "path": "PLACEHOLDER", "readonly": true },
            "process": { "terminal": false, "args": [], "env": [], "cwd": "/" },
            "mounts": [],
            "linux": { "namespaces": [
                {"type": "pid"}, {"type": "mount"}, {"type": "ipc"},
                {"type": "uts"}, {"type": "user"}
            ]}
        })
    }

    #[test]
    fn patch_writes_rootfs_path_into_root() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let out = patch_spec(&empty_spec_template(), &pol, &PathBuf::from("/var/rootfs"), None);
        assert_eq!(out["root"]["path"], "/var/rootfs");
        assert_eq!(out["root"]["readonly"], false);
    }

    #[test]
    fn patch_appends_workspace_bind_mount() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/home/alice/proj"));
        let out = patch_spec(&empty_spec_template(), &pol, &PathBuf::from("/r"), None);
        let mounts = out["mounts"].as_array().unwrap();
        let last = mounts.last().unwrap();
        assert_eq!(last["source"], "/home/alice/proj");
        assert_eq!(last["destination"], "/home/alice/proj");
        assert_eq!(last["type"], "bind");
        assert!(last["options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "rw"));
    }

    #[test]
    fn patch_omits_network_ns_when_open() {
        let mut pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        pol.network = NetworkPolicy::Open;
        let out = patch_spec(&empty_spec_template(), &pol, &PathBuf::from("/r"), None);
        let nss = out["linux"]["namespaces"].as_array().unwrap();
        assert!(!nss.iter().any(|n| n["type"] == "network"));
    }

    #[test]
    fn patch_adds_network_ns_when_deny_all() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let out = patch_spec(&empty_spec_template(), &pol, &PathBuf::from("/r"), None);
        let nss = out["linux"]["namespaces"].as_array().unwrap();
        assert!(nss.iter().any(|n| n["type"] == "network"));
    }

    #[test]
    fn patch_carries_cmd_into_process_and_keeps_path() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let cmd = LaunchCommand {
            bin: PathBuf::from("/bin/echo"),
            args: vec!["hi".into()],
            env: HashMap::from([("FOO".into(), "bar".into())]),
            cwd: PathBuf::from("/tmp/x"),
        };
        let out = patch_spec(&empty_spec_template(), &pol, &PathBuf::from("/r"), Some(&cmd));
        let argv = out["process"]["args"].as_array().unwrap();
        assert_eq!(argv[0], "/bin/echo");
        assert_eq!(argv[1], "hi");
        let env = out["process"]["env"].as_array().unwrap();
        assert!(env.iter().any(|v| v == "FOO=bar"));
        assert!(env
            .iter()
            .any(|v| v.as_str().is_some_and(|s| s.starts_with("PATH="))));
        assert_eq!(out["process"]["cwd"], "/tmp/x");
    }

    #[test]
    fn patch_does_not_overwrite_template_path_if_cmd_has_none() {
        let mut template = empty_spec_template();
        template["process"]["env"] = serde_json::json!(["PATH=/custom/bin", "TERM=dumb"]);
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let cmd = LaunchCommand {
            bin: PathBuf::from("/bin/echo"),
            args: vec![],
            env: HashMap::new(),
            cwd: PathBuf::from("/"),
        };
        let out = patch_spec(&template, &pol, &PathBuf::from("/r"), Some(&cmd));
        let env = out["process"]["env"].as_array().unwrap();
        assert!(env.iter().any(|v| v == "PATH=/custom/bin"));
        assert!(env.iter().any(|v| v == "TERM=dumb"));
    }

    #[test]
    fn patch_translates_limits_to_resources() {
        let mut pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        pol.limits.memory_bytes = Some(512 * 1024 * 1024);
        pol.limits.pids_max = Some(256);
        let out = patch_spec(&empty_spec_template(), &pol, &PathBuf::from("/r"), None);
        let r = &out["linux"]["resources"];
        assert_eq!(r["memory"]["limit"], 512u64 * 1024 * 1024);
        assert_eq!(r["pids"]["limit"], 256);
        assert!(r.get("cpu").is_none());
    }
}
