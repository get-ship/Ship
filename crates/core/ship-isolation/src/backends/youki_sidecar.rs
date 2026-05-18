//! Long-lived `Youki` sandbox driven through the in-container `ship-sidecar`
//! over HTTP. Container PID 1 is the sidecar; the agent is its child,
//! launched via `POST /v1/exec`. Multiple execs may run against one
//! sandbox; stdout / stderr / exited stream out over SSE.
//!
//! **Status: spike.** Uses shared host network namespace + free TCP port
//! for the sidecar transport (`HTTPS_PROXY` story is a separate problem).
//! Production wants a Unix-socket-or-vsock transport bind-mounted into
//! the container netns, so the host can talk to the sidecar without
//! sharing host network.
//!
//! Lifecycle:
//!   1. `start(policy)` patches the bundle: sidecar bin bind-mounted in,
//!      `process.args` overridden to launch the sidecar on a free port.
//!      Spawns `youki run` as a detached child, polls `/v1/health` until
//!      the sidecar answers.
//!   2. `exec(...)`, `wait_for_exec(...)`, `signal(...)`, `write_stdin(...)`
//!      hit the sidecar's HTTP API.
//!   3. `shutdown()` POSTs `/v1/shutdown`, then `youki delete --force`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};

use crate::backends::youki::{patch_spec, Youki, YoukiConfig};
use crate::handle::SandboxId;
use crate::net::{HostMatcher, ProxyHandle, spawn_allowlist_proxy};
use crate::policy::{FsMount, MountMode, NetworkPolicy, SandboxPolicy};
use crate::sidecar::protocol::{ExecRequest, ExecResponse};
use crate::trait_def::IsolatorError;

const SIDECAR_INSIDE: &str = "/usr/local/bin/ship-sidecar";

#[derive(Debug, Clone)]
pub struct SidecarConfig {
    pub host_sidecar_bin: PathBuf,
    pub bind_addr: std::net::IpAddr,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            host_sidecar_bin: PathBuf::from("/usr/local/bin/ship-sidecar"),
            bind_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        }
    }
}

pub struct SidecarSandbox {
    _youki: Arc<Youki>,
    youki_cfg: YoukiConfig,
    side_cfg: SidecarConfig,
    id: SandboxId,
    bundle_dir: PathBuf,
    state_dir: PathBuf,
    container_id: String,
    port: u16,
    child: Option<Child>,
    http: reqwest::Client,
    proxy: Option<ProxyHandle>,
}

impl SidecarSandbox {
    pub fn new(youki: Arc<Youki>, youki_cfg: YoukiConfig, side_cfg: SidecarConfig) -> Self {
        let id = SandboxId::new();
        let bundle_dir = std::env::temp_dir().join(format!("ship-sidecar-bundle-{}", id.0));
        let state_dir = youki_cfg.state_root.join(&id.0);
        let container_id = format!("ship-side-{}", id.0);
        Self {
            _youki: youki,
            youki_cfg,
            side_cfg,
            id,
            bundle_dir,
            state_dir,
            container_id,
            port: 0,
            child: None,
            http: reqwest::Client::new(),
            proxy: None,
        }
    }

    pub fn proxy_port(&self) -> Option<u16> {
        self.proxy.as_ref().map(|p| p.port)
    }

    pub fn id(&self) -> &SandboxId {
        &self.id
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.side_cfg.bind_addr, self.port)
    }

    async fn pick_free_port(&self) -> Result<u16, IsolatorError> {
        let listener = TcpListener::bind((self.side_cfg.bind_addr, 0)).await?;
        let port = listener.local_addr()?.port();
        drop(listener);
        Ok(port)
    }

    pub async fn start(&mut self, policy: SandboxPolicy) -> Result<(), IsolatorError> {
        if !self.side_cfg.host_sidecar_bin.is_file() {
            return Err(IsolatorError::Unavailable(format!(
                "host sidecar binary not found at {}",
                self.side_cfg.host_sidecar_bin.display()
            )));
        }

        tokio::fs::create_dir_all(&self.bundle_dir).await?;
        tokio::fs::create_dir_all(&self.state_dir).await?;

        let port = self.pick_free_port().await?;
        self.port = port;

        let template_bytes = tokio::fs::read(&self.youki_cfg.spec_template).await?;
        let template: Value = serde_json::from_slice(&template_bytes)
            .map_err(|e| IsolatorError::PolicyRejected(e.to_string()))?;

        let mut sidecar_policy = policy.clone();
        sidecar_policy.network = NetworkPolicy::Open;
        sidecar_policy.mounts.push(FsMount {
            host_path: self.side_cfg.host_sidecar_bin.clone(),
            sandbox_path: PathBuf::from(SIDECAR_INSIDE),
            mode: MountMode::ReadOnly,
        });

        let proxy_env = match &policy.network {
            NetworkPolicy::Allowlist { hosts } | NetworkPolicy::Proxy { allow: hosts, .. }
                if !hosts.is_empty() =>
            {
                let matcher = HostMatcher::new(hosts.clone());
                let handle = spawn_allowlist_proxy(matcher).await?;
                let proxy_url = format!("http://{}:{}", self.side_cfg.bind_addr, handle.port);
                self.proxy = Some(handle);
                Some(proxy_url)
            }
            _ => None,
        };

        let mut sidecar_env: HashMap<String, String> = HashMap::from([(
            "PATH".into(),
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
        )]);
        if let Some(url) = &proxy_env {
            sidecar_env.insert("HTTP_PROXY".into(), url.clone());
            sidecar_env.insert("HTTPS_PROXY".into(), url.clone());
            sidecar_env.insert("http_proxy".into(), url.clone());
            sidecar_env.insert("https_proxy".into(), url.clone());
            sidecar_env.insert(
                "NO_PROXY".into(),
                "localhost,127.0.0.1,::1".into(),
            );
        }

        let sidecar_cmd = crate::trait_def::LaunchCommand {
            bin: PathBuf::from(SIDECAR_INSIDE),
            args: vec![
                "--addr".into(),
                format!("{}:{}", self.side_cfg.bind_addr, port),
                "--label".into(),
                policy.label.clone(),
            ],
            env: sidecar_env,
            cwd: PathBuf::from("/"),
        };

        let patched = patch_spec(
            &template,
            &sidecar_policy,
            &self.youki_cfg.rootfs,
            Some(&sidecar_cmd),
        );
        let body = serde_json::to_vec_pretty(&patched)
            .map_err(|e| IsolatorError::PolicyRejected(e.to_string()))?;
        tokio::fs::write(self.bundle_dir.join("config.json"), &body).await?;

        // Pre-create the bind-mount target inside the rootfs.
        if let Some(parent) = self
            .youki_cfg
            .rootfs
            .join(SIDECAR_INSIDE.trim_start_matches('/'))
            .parent()
        {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let target = self
            .youki_cfg
            .rootfs
            .join(SIDECAR_INSIDE.trim_start_matches('/'));
        if !target.exists() {
            let _ = tokio::fs::File::create(&target).await;
        }

        let child = Command::new(&self.youki_cfg.youki_bin)
            .arg("-r")
            .arg(&self.state_dir)
            .arg("run")
            .arg("--bundle")
            .arg(&self.bundle_dir)
            .arg(&self.container_id)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        self.child = Some(child);

        self.wait_until_healthy().await
    }

    async fn wait_until_healthy(&self) -> Result<(), IsolatorError> {
        let url = format!("{}/v1/health", self.base_url());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        while tokio::time::Instant::now() < deadline {
            if let Ok(r) = self.http.get(&url).send().await {
                if r.status().is_success() {
                    return Ok(());
                }
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        Err(IsolatorError::Sidecar(
            "sidecar did not answer /v1/health within 15s".into(),
        ))
    }

    pub async fn exec(
        &self,
        bin: impl Into<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: impl Into<String>,
    ) -> Result<ExecResponse, IsolatorError> {
        let req = ExecRequest {
            session_id: "ses_default".into(),
            bin: bin.into(),
            args,
            env,
            cwd: cwd.into(),
        };
        let resp = self
            .http
            .post(format!("{}/v1/exec", self.base_url()))
            .json(&req)
            .send()
            .await
            .map_err(|e| IsolatorError::Sidecar(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(IsolatorError::Sidecar(format!(
                "exec returned {}",
                resp.status()
            )));
        }
        resp.json::<ExecResponse>()
            .await
            .map_err(|e| IsolatorError::Sidecar(e.to_string()))
    }

    pub async fn signal(&self, exec_id: &str, signal: u8) -> Result<(), IsolatorError> {
        let resp = self
            .http
            .post(format!("{}/v1/signal", self.base_url()))
            .json(&serde_json::json!({"exec_id": exec_id, "signal": signal}))
            .send()
            .await
            .map_err(|e| IsolatorError::Sidecar(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(IsolatorError::Sidecar(format!(
                "signal returned {}",
                resp.status()
            )));
        }
        Ok(())
    }

    pub async fn write_stdin(&self, exec_id: &str, data: &[u8]) -> Result<(), IsolatorError> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let resp = self
            .http
            .post(format!("{}/v1/stdin", self.base_url()))
            .json(&serde_json::json!({"exec_id": exec_id, "data_b64": b64}))
            .send()
            .await
            .map_err(|e| IsolatorError::Sidecar(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(IsolatorError::Sidecar(format!(
                "stdin returned {}",
                resp.status()
            )));
        }
        Ok(())
    }

    /// Convenience: exec + capture all stdout/stderr until Exited, then
    /// return (exit_code, stdout, stderr). Bounded by a 60s overall
    /// deadline and a 5s inter-chunk timeout. Subscribes to events first
    /// to avoid broadcast races.
    pub async fn run_capture(
        &self,
        bin: impl Into<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: impl Into<String>,
    ) -> Result<(Option<i32>, Vec<u8>, Vec<u8>), IsolatorError> {
        let resp = self
            .http
            .get(format!("{}/v1/events", self.base_url()))
            .send()
            .await
            .map_err(|e| IsolatorError::Sidecar(e.to_string()))?;
        let mut stream = resp.bytes_stream();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let exec = self.exec(bin, args, env, cwd).await?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit = None;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        while tokio::time::Instant::now() < deadline {
            let chunk =
                tokio::time::timeout(Duration::from_secs(5), stream.next()).await;
            let chunk = match chunk {
                Ok(Some(Ok(c))) => c,
                Ok(Some(Err(e))) => {
                    return Err(IsolatorError::Sidecar(format!("event stream: {e}")));
                }
                Ok(None) => break,
                Err(_) => continue,
            };
            for line in std::str::from_utf8(&chunk).unwrap_or("").lines() {
                let Some(data) = line.strip_prefix("data: ") else { continue };
                let v: Value = match serde_json::from_str(data) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let ty = v["type"].as_str().unwrap_or("");
                let id = v["exec_id"].as_str().unwrap_or("");
                if id != exec.exec_id {
                    continue;
                }
                match ty {
                    "stdout" => {
                        if let Some(b) = v["data_b64"].as_str() {
                            if let Ok(bytes) =
                                base64::engine::general_purpose::STANDARD.decode(b)
                            {
                                stdout.extend_from_slice(&bytes);
                            }
                        }
                    }
                    "stderr" => {
                        if let Some(b) = v["data_b64"].as_str() {
                            if let Ok(bytes) =
                                base64::engine::general_purpose::STANDARD.decode(b)
                            {
                                stderr.extend_from_slice(&bytes);
                            }
                        }
                    }
                    "exited" => {
                        exit = v["code"].as_i64().map(|c| c as i32);
                    }
                    _ => {}
                }
            }
            if exit.is_some() {
                break;
            }
        }
        Ok((exit, stdout, stderr))
    }

    pub async fn proxy_log_entries(
        &self,
    ) -> Option<Vec<(String, crate::net::ProxyDecision)>> {
        let proxy = self.proxy.as_ref()?;
        Some(proxy.log.lock().await.entries().to_vec())
    }

    pub async fn wait_for_exec(&self, exec_id: &str) -> Result<Option<i32>, IsolatorError> {
        let resp = self
            .http
            .get(format!("{}/v1/events", self.base_url()))
            .send()
            .await
            .map_err(|e| IsolatorError::Sidecar(e.to_string()))?;
        let mut stream = resp.bytes_stream();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(600);
        while tokio::time::Instant::now() < deadline {
            let chunk = tokio::time::timeout(Duration::from_secs(60), stream.next()).await;
            let chunk = match chunk {
                Ok(Some(Ok(c))) => c,
                Ok(Some(Err(e))) => {
                    return Err(IsolatorError::Sidecar(format!("event stream: {e}")))
                }
                Ok(None) => return Ok(None),
                Err(_) => continue,
            };
            for line in std::str::from_utf8(&chunk).unwrap_or("").lines() {
                let Some(data) = line.strip_prefix("data: ") else {
                    continue;
                };
                #[derive(Deserialize)]
                #[serde(tag = "type", rename_all = "snake_case")]
                enum Ev {
                    Exited { exec_id: String, code: Option<i32> },
                    #[serde(other)]
                    Other,
                }
                if let Ok(Ev::Exited { exec_id: id, code }) =
                    serde_json::from_str::<Ev>(data)
                {
                    if id == exec_id {
                        return Ok(code);
                    }
                }
            }
        }
        Err(IsolatorError::Sidecar("wait_for_exec timed out".into()))
    }

    pub async fn shutdown(mut self) -> Result<(), IsolatorError> {
        let _ = self
            .http
            .post(format!("{}/v1/shutdown", self.base_url()))
            .json(&serde_json::json!({"session_id": "ses_default"}))
            .send()
            .await;

        if let Some(mut child) = self.child.take() {
            let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
            let _ = child.kill().await;
        }

        let _ = Command::new(&self.youki_cfg.youki_bin)
            .arg("-r")
            .arg(&self.state_dir)
            .arg("delete")
            .arg("--force")
            .arg(&self.container_id)
            .status()
            .await;

        if let Some(proxy) = self.proxy.take() {
            proxy.shutdown().await;
        }

        let _ = tokio::fs::remove_dir_all(&self.bundle_dir).await;
        let _ = tokio::fs::remove_dir_all(&self.state_dir).await;
        Ok(())
    }
}
