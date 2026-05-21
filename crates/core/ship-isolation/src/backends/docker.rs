//! Docker / Podman backend.
//!
//! Smallest-surface Tier 2 strategy: `prepare()` creates a container
//! running `ship-sidecar` as PID 1 (bind-mounted in, v1 listener
//! published to a host port); the supervisor drives the actor over
//! ship.sidecar.v1. No rootfs to ship, no youki binary to bundle — just
//! "Docker is installed". No `docker exec` — the sidecar owns all
//! in-container execution.
//!
//! Same code path covers Podman: `podman` is CLI-compatible with the
//! arguments we use, and the recommender treats Podman as a
//! `DockerEngine` strategy because the user-visible behavior is the
//! same.
//!
//! What's intentionally minimal:
//!
//! - One container per `prepare()`, running the sidecar.
//! - Bind-mounts from `policy.mounts`, identical-path passthrough by
//!   convention.
//! - `NetworkPolicy::DenyAll` → `--network none`; `Open` → host
//!   default; `Allowlist` / `Proxy` → caller must spawn the host-side
//!   `ProxyHandle` and pass `HTTPS_PROXY` / `HTTP_PROXY` env, same
//!   pattern as the youki sidecar backend.
//! - `ResourceLimits` translate to `--memory`, `--cpus`, `--pids-limit`.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Mutex;

use ship_substrate::isolation::{SandboxHandle, SandboxId};
use ship_substrate::isolation::{MountMode, NetworkPolicy, SandboxPolicy};
use ship_substrate::isolation::{Isolator, IsolatorError};

/// The path the host `ship-sidecar` binary is bind-mounted to inside
/// the container. Matches the youki sidecar convention.
const SIDECAR_INSIDE: &str = "/usr/local/bin/ship-sidecar";
/// Port the sidecar's v1 listener binds inside the container.
const SIDECAR_PORT_INSIDE: u16 = 2468;
/// Hostname injected into the container (`--add-host`) that resolves to
/// the host's gateway, so an in-container agent can reach the host
/// daemon's HTTP plane (MCP / hooks / log_progress). `host-gateway` is
/// honored by both docker and rootless podman; verified live against
/// rootless podman on Linux. The zero-egress hardened tier
/// (`NetworkPolicy::DenyAll`, `--network none`) has no interface at all
/// and carries the daemon plane over the v1 typed Envelope instead — so
/// we only inject this when the container actually has a network.
const HOST_GATEWAY_ALIAS: &str = "host.docker.internal";

#[derive(Debug, Clone)]
pub struct DockerConfig {
    /// `docker` or `podman`. The recommender picks; this struct accepts
    /// the binary path directly so users can override.
    pub engine_bin: String,
    /// Image tag. Default is the bun+claude rootfs we use elsewhere,
    /// but any image with the agent CLI installed works.
    pub image: String,
    /// Container name prefix (`ship-agent-<nanoid>` by default).
    pub name_prefix: String,
    /// The host `ship-sidecar` binary, bind-mounted in as PID 1 with its
    /// v1 listener published to a host port. The supervisor drives the
    /// actor over ship.sidecar.v1 — same mechanism as `youki_sidecar`.
    /// This is the only mode: the container always runs the sidecar (no
    /// `docker exec`). Defaults to the conventional install path; tests
    /// and the daemon override it.
    pub sidecar_bin: PathBuf,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            engine_bin: "docker".into(),
            image: "ghcr.io/ship/agent-runner:latest".into(),
            name_prefix: "ship-agent".into(),
            sidecar_bin: PathBuf::from("/usr/local/bin/ship-sidecar"),
        }
    }
}

/// Docker / Podman backend. One instance per orchestrator; `prepare()`
/// allocates a fresh container.
pub struct Docker {
    cfg: DockerConfig,
}

impl Docker {
    pub fn new(cfg: DockerConfig) -> Self {
        Self { cfg }
    }

    /// The sidecar's v1 endpoint (`http://127.0.0.1:<published-port>`).
    /// `None` only when the sandbox wasn't prepared by this backend
    /// (downcast miss). The supervisor reads this to build a
    /// `SidecarActorRuntime`.
    pub fn sidecar_endpoint(sandbox: &SandboxHandle) -> Option<String> {
        let st = sandbox.state.downcast_ref::<DockerState>()?;
        Some(format!("http://127.0.0.1:{}", st.sidecar_port))
    }

    /// Rewrite a host-loopback daemon origin into a URL an actor *inside*
    /// the container can reach. The host daemon binds loopback
    /// (`127.0.0.1`/`localhost`, or `0.0.0.0` in non-dev), which inside a
    /// container points at the container's own loopback — not the host.
    /// We swap the host segment for `host.docker.internal`, which resolves
    /// to the host gateway via the `--add-host` the container is created
    /// with. A non-loopback host (already-remote daemon) is returned
    /// unchanged — it's reachable as-is.
    pub fn container_reachable_url(host_origin: &str) -> String {
        let replacement = format!("://{HOST_GATEWAY_ALIAS}");
        for loopback in ["://127.0.0.1", "://localhost", "://0.0.0.0"] {
            if host_origin.contains(loopback) {
                return host_origin.replacen(loopback, &replacement, 1);
            }
        }
        host_origin.to_string()
    }
}

/// `--add-host` args making `host.docker.internal` resolve to the host
/// gateway. Empty for `DenyAll` — that tier has no interface and uses the
/// v1 Envelope plane for daemon traffic (see `HOST_GATEWAY_ALIAS`).
fn host_gateway_args(policy: &NetworkPolicy) -> Vec<String> {
    match policy {
        NetworkPolicy::DenyAll => Vec::new(),
        _ => vec![
            "--add-host".into(),
            format!("{HOST_GATEWAY_ALIAS}:host-gateway"),
        ],
    }
}

/// Per-container state stashed in `SandboxHandle.state`. `shutdown()`
/// reaches in via `Arc::downcast`.
struct DockerState {
    name: String,
    cfg: DockerConfig,
    /// Once the container exits, the daemon GCs it; we record that we've
    /// asked for shutdown so a double-shutdown doesn't bubble an error.
    shut_down: Mutex<bool>,
    /// Host port the sidecar's v1 listener is published on.
    sidecar_port: u16,
}

/// Pick a free localhost TCP port to publish the sidecar on.
async fn pick_free_port() -> Result<u16, IsolatorError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(IsolatorError::from)?;
    let port = listener.local_addr().map_err(IsolatorError::from)?.port();
    drop(listener);
    Ok(port)
}

/// Poll a TCP connect to the published sidecar port until it accepts
/// (sidecar is listening) or the deadline elapses.
async fn wait_until_listening(port: u16) -> Result<(), IsolatorError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    Err(IsolatorError::Sidecar(format!(
        "sidecar did not accept connections on 127.0.0.1:{port} within 15s"
    )))
}

#[async_trait]
impl Isolator for Docker {
    fn name(&self) -> &'static str {
        "docker"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        let out = Command::new(&self.cfg.engine_bin)
            .arg("version")
            .arg("--format")
            .arg("{{.Server.Version}}")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(|e| {
                IsolatorError::Unavailable(format!(
                    "engine binary `{}` not found: {e}",
                    self.cfg.engine_bin
                ))
            })?;
        if !out.success() {
            return Err(IsolatorError::Unavailable(format!(
                "`{} version` returned non-zero — daemon not reachable",
                self.cfg.engine_bin
            )));
        }
        Ok(())
    }

    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let name = format!("{}-{}", self.cfg.name_prefix, nanoid::nanoid!(10));
        let root = policy
            .mounts
            .first()
            .map(|m| m.host_path.clone())
            .unwrap_or_else(std::env::temp_dir);

        // NB: `create` + `start` (below), not `run` — so no `--detach`
        // here; it's a `run`-only flag and both docker and podman reject
        // it on `create`.
        let mut argv: Vec<String> = vec![
            "create".into(),
            "--name".into(),
            name.clone(),
            "--init".into(),
        ];

        argv.extend(network_args(&policy.network));
        argv.extend(host_gateway_args(&policy.network));
        argv.extend(limit_args(policy));
        argv.extend(mount_args(policy));
        for (k, v) in &policy.env {
            argv.push("--env".into());
            argv.push(format!("{k}={v}"));
        }

        // Bind-mount the host ship-sidecar binary, publish its v1 listener
        // to a host port, and make it PID 1. The supervisor drives the
        // actor over ship.sidecar.v1 — no `docker exec`.
        let sidecar_port = pick_free_port().await?;
        argv.push("-v".into());
        argv.push(format!("{}:{SIDECAR_INSIDE}:ro", self.cfg.sidecar_bin.display()));
        argv.push("-p".into());
        argv.push(format!("127.0.0.1:{sidecar_port}:{SIDECAR_PORT_INSIDE}"));

        argv.push(self.cfg.image.clone());
        argv.push(SIDECAR_INSIDE.into());
        argv.push("--addr".into());
        argv.push(format!("0.0.0.0:{SIDECAR_PORT_INSIDE}"));

        let out = Command::new(&self.cfg.engine_bin)
            .args(&argv)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(IsolatorError::from)?;
        if !out.status.success() {
            return Err(IsolatorError::SpawnFailed(format!(
                "{} create failed: {}",
                self.cfg.engine_bin,
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }

        let start = Command::new(&self.cfg.engine_bin)
            .args(["start", &name])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(IsolatorError::from)?;
        if !start.status.success() {
            return Err(IsolatorError::SpawnFailed(format!(
                "{} start failed: {}",
                self.cfg.engine_bin,
                String::from_utf8_lossy(&start.stderr).trim()
            )));
        }

        // Wait for the published port to accept before returning — the
        // supervisor will immediately connect a v1 client against it.
        wait_until_listening(sidecar_port).await?;

        Ok(SandboxHandle {
            id: SandboxId::new(),
            backend: "docker",
            root,
            state: Arc::new(DockerState {
                name,
                cfg: self.cfg.clone(),
                shut_down: Mutex::new(false),
                sidecar_port,
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        let state = sandbox
            .state
            .downcast_ref::<DockerState>()
            .ok_or_else(|| {
                IsolatorError::PolicyRejected("sandbox was not prepared by Docker backend".into())
            })?;
        let mut guard = state.shut_down.lock().await;
        if *guard {
            return Ok(());
        }
        *guard = true;
        // `rm -f` stops + removes in one shot. We don't surface a
        // failure here as an error to the orchestrator — by the time
        // shutdown is called the agent is done; we're just freeing
        // resources.
        let _ = Command::new(&state.cfg.engine_bin)
            .args(["rm", "-f", &state.name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
        Ok(())
    }
}

fn network_args(policy: &NetworkPolicy) -> Vec<String> {
    match policy {
        NetworkPolicy::DenyAll => vec!["--network".into(), "none".into()],
        // Allowlist/Proxy: container keeps default network so it can
        // reach the host-side proxy on 127.0.0.1. The caller is
        // responsible for spawning `ProxyHandle` and passing
        // HTTPS_PROXY/HTTP_PROXY via the actor's spawn env.
        NetworkPolicy::Allowlist { .. } | NetworkPolicy::Proxy { .. } => Vec::new(),
        NetworkPolicy::Open => Vec::new(),
    }
}

fn limit_args(policy: &SandboxPolicy) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(bytes) = policy.limits.memory_bytes {
        out.push("--memory".into());
        out.push(format!("{bytes}"));
    }
    if let Some(quota_us) = policy.limits.cpu_quota_micros {
        // Docker takes a fractional CPU count, not raw cgroup quota.
        // 100_000 us = 1 CPU; convert.
        let cpus = (quota_us as f64) / 100_000.0;
        out.push("--cpus".into());
        out.push(format!("{cpus:.2}"));
    }
    if let Some(pids) = policy.limits.pids_max {
        out.push("--pids-limit".into());
        out.push(format!("{pids}"));
    }
    out
}

fn mount_args(policy: &SandboxPolicy) -> Vec<String> {
    let mut out = Vec::new();
    for m in &policy.mounts {
        let ro = matches!(m.mode, MountMode::ReadOnly).then_some(",ro").unwrap_or("");
        out.push("--mount".into());
        out.push(format!(
            "type=bind,source={},target={}{ro}",
            m.host_path.display(),
            m.sandbox_path.display()
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::isolation::{FsMount, ResourceLimits};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn policy() -> SandboxPolicy {
        SandboxPolicy {
            label: "t".into(),
            mounts: vec![FsMount {
                host_path: PathBuf::from("/tmp/work"),
                sandbox_path: PathBuf::from("/work"),
                mode: MountMode::ReadWrite,
            }],
            network: NetworkPolicy::DenyAll,
            env: HashMap::new(),
            limits: ResourceLimits {
                memory_bytes: Some(2 * 1024 * 1024 * 1024),
                cpu_quota_micros: Some(200_000),
                pids_max: Some(512),
            },
        }
    }

    #[test]
    fn deny_all_emits_network_none() {
        let args = network_args(&NetworkPolicy::DenyAll);
        assert_eq!(args, vec!["--network", "none"]);
    }

    #[test]
    fn reachable_url_rewrites_loopback_to_host_gateway() {
        assert_eq!(
            Docker::container_reachable_url("http://127.0.0.1:9315"),
            "http://host.docker.internal:9315"
        );
        assert_eq!(
            Docker::container_reachable_url("http://localhost:9315"),
            "http://host.docker.internal:9315"
        );
        assert_eq!(
            Docker::container_reachable_url("http://0.0.0.0:9315"),
            "http://host.docker.internal:9315"
        );
    }

    #[test]
    fn reachable_url_leaves_remote_host_untouched() {
        // An already-remote daemon (cloud / LAN peer) is reachable as-is.
        assert_eq!(
            Docker::container_reachable_url("https://ship.example.com:443"),
            "https://ship.example.com:443"
        );
    }

    #[test]
    fn host_gateway_added_when_networked_omitted_when_denied() {
        // DenyAll has no interface → no add-host (v1 Envelope plane instead).
        assert!(host_gateway_args(&NetworkPolicy::DenyAll).is_empty());
        // Open / Allowlist get the host-gateway alias so the daemon URL resolves.
        let open = host_gateway_args(&NetworkPolicy::Open);
        assert_eq!(open[0], "--add-host");
        assert_eq!(open[1], "host.docker.internal:host-gateway");
        assert_eq!(
            host_gateway_args(&NetworkPolicy::Allowlist { hosts: vec![] }),
            open
        );
    }

    #[test]
    fn allowlist_keeps_default_network_for_host_proxy() {
        let args = network_args(&NetworkPolicy::Allowlist {
            hosts: vec!["api.test".into()],
        });
        assert!(args.is_empty(), "container must reach host proxy on 127.0.0.1");
    }

    #[test]
    fn limits_translate_to_cli_units() {
        let args = limit_args(&policy());
        // memory bytes verbatim
        assert!(args.iter().any(|a| a == "2147483648"));
        // 200_000 us → 2.00 CPUs
        assert!(args.iter().any(|a| a == "2.00"));
        // pids limit
        assert!(args.iter().any(|a| a == "512"));
    }

    #[test]
    fn mount_string_uses_bind_with_target() {
        let args = mount_args(&policy());
        let m = &args[1];
        assert!(m.contains("type=bind"));
        assert!(m.contains("source=/tmp/work"));
        assert!(m.contains("target=/work"));
        assert!(!m.contains(",ro"));
    }

    #[test]
    fn read_only_mount_carries_ro_flag() {
        let mut p = policy();
        p.mounts[0].mode = MountMode::ReadOnly;
        let args = mount_args(&p);
        assert!(args[1].contains(",ro"));
    }

    #[tokio::test]
    async fn available_fails_loud_when_engine_missing() {
        let cfg = DockerConfig {
            engine_bin: "does-not-exist-xyzzy".into(),
            ..Default::default()
        };
        let d = Docker::new(cfg);
        let err = d.available().await.unwrap_err();
        assert!(matches!(err, IsolatorError::Unavailable(_)));
    }
}
