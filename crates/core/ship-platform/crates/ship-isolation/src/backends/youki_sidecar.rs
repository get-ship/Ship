//! Long-lived `Youki` sandbox driven through the in-container
//! `ship-sidecar` over ConnectRPC. Container PID 1 is the sidecar;
//! the agent is one of its children, launched via
//! `Processes.Create`. Multiple processes may run against one
//! sandbox; stdout / stderr / exit stream out over `Events.Stream`.
//!
//! **Transport.** TCP over the shared host network namespace + a
//! free port. Production wants a Unix-socket-or-vsock bind-mounted
//! into the container netns so the host can talk to the sidecar
//! without sharing the host network — separate chunk.
//!
//! Lifecycle:
//!   1. `start(policy)` patches the OCI bundle: sidecar bin
//!      bind-mounted in, `process.args` overridden to launch the
//!      sidecar on a free port. Spawns `youki run` as a detached
//!      child, polls `GET /v1/health` until the sidecar answers.
//!   2. `exec(...)`, `wait_for_exit(...)`, `signal(...)`,
//!      `write_stdin(...)` drive the in-container sidecar through
//!      the generated `ProcessesClient`. `run_capture(...)` uses
//!      the unary `Processes.Run` RPC (which captures full output
//!      in one round-trip).
//!   3. `shutdown()` tears the container down via
//!      `youki delete --force` (no separate sidecar shutdown RPC
//!      in v1 — youki's SIGKILL to PID 1 reaps everything).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use connectrpc::client::{ClientConfig, HttpClient};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};

use crate::backends::youki::{Youki, YoukiConfig, patch_spec};
use crate::net::{HostMatcher, ProxyHandle, spawn_allowlist_proxy};
use crate::sidecar::proto::*;
use crate::sidecar::proto::__buffa::view::oneof::envelope as env_view;
use isolation_port::{
    FsMount, IsolatorError, MountMode, NetworkPolicy, SandboxId, SandboxPolicy,
};

const SIDECAR_INSIDE: &str = "/usr/local/bin/ship-sidecar";

/// Host-facing exec receipt. Mirrors the v1 `ProcessInfo` slice
/// callers care about so live tests don't need to touch buffa views.
#[derive(Debug, Clone)]
pub struct ExecHandle {
    pub exec_id: String,
    pub pid: Option<u32>,
}

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
    procs: Option<ProcessesClient<HttpClient>>,
    events: Option<EventsClient<HttpClient>>,
    health: reqwest::Client,
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
            procs: None,
            events: None,
            health: reqwest::Client::new(),
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
            sidecar_env.insert("NO_PROXY".into(), "localhost,127.0.0.1,::1".into());
        }

        let sidecar_cmd = isolation_port::LaunchCommand {
            bin: PathBuf::from(SIDECAR_INSIDE),
            args: vec![
                "--addr".into(),
                format!("{}:{}", self.side_cfg.bind_addr, port),
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

        self.wait_until_healthy().await?;

        let http = HttpClient::plaintext();
        let uri: ::http::Uri = self
            .base_url()
            .parse()
            .map_err(|e: ::http::uri::InvalidUri| IsolatorError::Sidecar(e.to_string()))?;
        let config = ClientConfig::new(uri);
        self.procs = Some(ProcessesClient::new(http.clone(), config.clone()));
        self.events = Some(EventsClient::new(http, config));
        Ok(())
    }

    async fn wait_until_healthy(&self) -> Result<(), IsolatorError> {
        let url = format!("{}/v1/health", self.base_url());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        while tokio::time::Instant::now() < deadline {
            if let Ok(r) = self.health.get(&url).send().await
                && r.status().is_success()
            {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        Err(IsolatorError::Sidecar(
            "sidecar did not answer /v1/health within 15s".into(),
        ))
    }

    fn procs_client(&self) -> Result<&ProcessesClient<HttpClient>, IsolatorError> {
        self.procs
            .as_ref()
            .ok_or_else(|| IsolatorError::Sidecar("sidecar not started".into()))
    }

    fn events_client(&self) -> Result<&EventsClient<HttpClient>, IsolatorError> {
        self.events
            .as_ref()
            .ok_or_else(|| IsolatorError::Sidecar("sidecar not started".into()))
    }

    /// Spawn a long-lived process via `Processes.Create`. Caller is
    /// expected to follow up with `wait_for_exit` if it cares about
    /// the exit code or wants to drain output via the event stream.
    pub async fn exec(
        &self,
        bin: impl Into<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: impl Into<String>,
    ) -> Result<ExecHandle, IsolatorError> {
        let mut req = ProcessCreateRequest::default();
        req.bin = bin.into();
        req.args = args;
        req.env.extend(env);
        req.cwd = cwd.into();
        let resp = self
            .procs_client()?
            .create(req)
            .await
            .map_err(|e| IsolatorError::Sidecar(format!("create: {e}")))?;
        let v = resp.into_view();
        Ok(ExecHandle {
            exec_id: v.id.to_string(),
            pid: if v.pid == 0 { None } else { Some(v.pid) },
        })
    }

    pub async fn signal(&self, exec_id: &str, signal: u8) -> Result<(), IsolatorError> {
        let procs = self.procs_client()?;
        match signal {
            9 => {
                let mut req = ProcessKillRequest::default();
                req.id = exec_id.into();
                procs
                    .kill(req)
                    .await
                    .map_err(|e| IsolatorError::Sidecar(format!("kill: {e}")))?;
            }
            _ => {
                let mut req = ProcessStopRequest::default();
                req.id = exec_id.into();
                procs
                    .stop(req)
                    .await
                    .map_err(|e| IsolatorError::Sidecar(format!("stop: {e}")))?;
            }
        }
        Ok(())
    }

    pub async fn write_stdin(&self, exec_id: &str, data: &[u8]) -> Result<(), IsolatorError> {
        let mut req = ProcessWriteInputRequest::default();
        req.id = exec_id.into();
        req.data = data.to_vec();
        self.procs_client()?
            .write_input(req)
            .await
            .map_err(|e| IsolatorError::Sidecar(format!("write_input: {e}")))?;
        Ok(())
    }

    /// Block until the named process exits, return its exit code.
    ///
    /// Subscribe-then-Get to close the race window: the broadcast
    /// channel doesn't replay, so a process that exits before the
    /// subscription is established would otherwise time out. We
    /// subscribe first, then check current state — any exit
    /// before the subscription is caught by the Get, any exit
    /// after is caught by the stream.
    pub async fn wait_for_exit(&self, exec_id: &str) -> Result<Option<i32>, IsolatorError> {
        let mut stream = self
            .events_client()?
            .stream()
            .await
            .map_err(|e| IsolatorError::Sidecar(format!("events.stream: {e}")))?;

        // Check current state. The sidecar retains process records
        // after exit so Get always returns ProcessInfo, even for a
        // process that completed before we subscribed.
        let mut get_req = ProcessGetRequest::default();
        get_req.id = exec_id.into();
        let info = self
            .procs_client()?
            .get(get_req)
            .await
            .map_err(|e| IsolatorError::Sidecar(format!("get: {e}")))?;
        let view = info.into_view();
        if matches!(
            view.state.as_known(),
            Some(ProcessState::PROCESS_STATE_EXITED) | Some(ProcessState::PROCESS_STATE_KILLED)
        ) {
            return Ok(Some(view.exit_code));
        }

        let deadline = tokio::time::Instant::now() + Duration::from_secs(600);
        while tokio::time::Instant::now() < deadline {
            let msg = tokio::time::timeout(Duration::from_secs(60), stream.message()).await;
            let env = match msg {
                Ok(Ok(Some(env))) => env,
                Ok(Ok(None)) => return Ok(None),
                Ok(Err(e)) => {
                    return Err(IsolatorError::Sidecar(format!("events.stream: {e}")));
                }
                Err(_) => continue,
            };
            if let Some(env_view::Payload::ProcessExited(x)) = env.payload.as_ref()
                && x.process_id == exec_id
            {
                return Ok(Some(x.exit_code));
            }
        }
        Err(IsolatorError::Sidecar("wait_for_exit timed out".into()))
    }

    /// One-shot run + full capture. Uses the unary `Processes.Run`
    /// RPC — no event subscription needed, no SSE chunk parsing,
    /// no inter-chunk timing fragility.
    pub async fn run_capture(
        &self,
        bin: impl Into<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: impl Into<String>,
    ) -> Result<(Option<i32>, Vec<u8>, Vec<u8>), IsolatorError> {
        let mut req = ProcessRunRequest::default();
        req.bin = bin.into();
        req.args = args;
        req.env.extend(env);
        req.cwd = cwd.into();
        let resp = self
            .procs_client()?
            .run(req)
            .await
            .map_err(|e| IsolatorError::Sidecar(format!("run: {e}")))?;
        let v = resp.into_view();
        let exit = if v.exit_code == -1 {
            None
        } else {
            Some(v.exit_code)
        };
        Ok((exit, v.stdout.to_vec(), v.stderr.to_vec()))
    }

    pub async fn proxy_log_entries(&self) -> Option<Vec<(String, crate::net::ProxyDecision)>> {
        let proxy = self.proxy.as_ref()?;
        Some(proxy.log.lock().await.entries().to_vec())
    }

    pub async fn shutdown(mut self) -> Result<(), IsolatorError> {
        // The v1 sidecar has no Shutdown RPC — youki's SIGKILL to PID 1
        // tears down the sidecar and reaps every child.
        let _ = Command::new(&self.youki_cfg.youki_bin)
            .arg("-r")
            .arg(&self.state_dir)
            .arg("delete")
            .arg("--force")
            .arg(&self.container_id)
            .status()
            .await;

        if let Some(mut child) = self.child.take() {
            let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
            let _ = child.kill().await;
        }

        if let Some(proxy) = self.proxy.take() {
            proxy.shutdown().await;
        }

        let _ = tokio::fs::remove_dir_all(&self.bundle_dir).await;
        let _ = tokio::fs::remove_dir_all(&self.state_dir).await;
        Ok(())
    }
}
