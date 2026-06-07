//! Apple Container Tier-3 `Isolator` — macOS Apple Silicon. Each container is
//! its own microVM (own kernel) → stronger inter-actor isolation than a
//! shared Lima VM.
//!
//! DRAFT pending Apple-Silicon verification. The structure mirrors the Docker
//! Isolator (run a container with the sidecar, reach it over the network),
//! but Apple's `container` CLI gives each container its own IP on a vmnet
//! bridge rather than localhost port-forwarding — so the endpoint is
//! `http://<container-ip>:<port>`. The IP-extraction from `container inspect`
//! (`ip_from_inspect`) is the #1 thing to verify/adjust on real hardware; the
//! `run`/`inspect`/`stop`/`delete` verbs are Docker-shaped and likely correct.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;

use ship_substrate::isolation::{Isolator, IsolatorError, SandboxHandle, SandboxId, SandboxPolicy};

/// Port the in-container sidecar binds. The container has its own IP, so this
/// is fixed (no host-port collision — one sidecar per microVM).
const SIDECAR_PORT: u16 = 2468;

#[derive(Debug, Clone)]
pub struct AppleContainerConfig {
    /// Apple's `container` CLI (default "container").
    pub container_bin: String,
    /// OCI image (arm64) with ship-sidecar baked in.
    pub image: String,
    /// Sidecar path inside the image.
    pub sidecar_path: String,
}

pub struct AppleContainer {
    cfg: AppleContainerConfig,
}

struct AcState {
    container_bin: String,
    name: String,
    endpoint: Option<String>,
}

impl AppleContainer {
    pub fn new(cfg: AppleContainerConfig) -> Self {
        Self { cfg }
    }

    /// Host endpoint for the in-container sidecar: `http://<container-ip>:<port>`.
    pub fn sidecar_endpoint(sandbox: &SandboxHandle) -> Option<String> {
        sandbox.state.downcast_ref::<AcState>()?.endpoint.clone()
    }
}

/// Pull the container's IP out of `container inspect` JSON. DRAFT: the exact
/// shape is unverified — adjust the field paths on real hardware. Tries a few
/// likely locations and strips any CIDR suffix.
fn ip_from_inspect(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let obj = v.as_array().and_then(|a| a.first()).unwrap_or(&v);
    let candidates: &[&[&str]] = &[
        &["networks", "0", "address"],
        &["networks", "0", "ipv4"],
        &["network", "address"],
        &["NetworkSettings", "IPAddress"],
    ];
    for path in candidates {
        let mut cur = obj;
        let mut ok = true;
        for key in *path {
            cur = match key.parse::<usize>() {
                Ok(i) => match cur.get(i) {
                    Some(x) => x,
                    None => {
                        ok = false;
                        break;
                    }
                },
                Err(_) => match cur.get(key) {
                    Some(x) => x,
                    None => {
                        ok = false;
                        break;
                    }
                },
            };
        }
        if ok {
            if let Some(s) = cur.as_str() {
                return Some(s.split('/').next().unwrap_or(s).to_string());
            }
        }
    }
    None
}

async fn wait_until_ready(host: &str, port: u16) -> Result<(), IsolatorError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect((host, port)).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(IsolatorError::Sidecar(format!(
        "Apple Container sidecar not reachable at {host}:{port} within 30s"
    )))
}

#[async_trait]
impl Isolator for AppleContainer {
    fn name(&self) -> &'static str {
        "apple_container"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        if !cfg!(target_os = "macos") {
            return Err(IsolatorError::Unavailable(
                "Apple Container is macOS (Apple Silicon) only".into(),
            ));
        }
        let out = Command::new(&self.cfg.container_bin)
            .arg("--version")
            .output()
            .await
            .map_err(|e| IsolatorError::Unavailable(format!("`container` CLI: {e}")))?;
        if !out.status.success() {
            return Err(IsolatorError::Unavailable("`container` CLI not usable".into()));
        }
        Ok(())
    }

    async fn prepare(&self, _policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let id = SandboxId::new();
        let name = format!("ship-{}", id.0);

        // Each container is its own microVM. Run detached with the sidecar as
        // the command, bound on all interfaces (the container IP is what the
        // host dials).
        let run = Command::new(&self.cfg.container_bin)
            .args([
                "run",
                "-d",
                "--name",
                &name,
                &self.cfg.image,
                &self.cfg.sidecar_path,
                "--addr",
                &format!("0.0.0.0:{SIDECAR_PORT}"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| IsolatorError::SpawnFailed(format!("container run: {e}")))?;
        if !run.status.success() {
            return Err(IsolatorError::SpawnFailed(format!(
                "container run: {}",
                String::from_utf8_lossy(&run.stderr).trim()
            )));
        }

        let inspect = Command::new(&self.cfg.container_bin)
            .args(["inspect", &name])
            .output()
            .await
            .map_err(|e| IsolatorError::SpawnFailed(format!("container inspect: {e}")))?;
        let ip = ip_from_inspect(&String::from_utf8_lossy(&inspect.stdout)).ok_or_else(|| {
            IsolatorError::SpawnFailed(
                "could not resolve container IP from `container inspect` \
                 (DRAFT: adjust ip_from_inspect for the real JSON shape)"
                    .into(),
            )
        })?;

        if let Err(e) = wait_until_ready(&ip, SIDECAR_PORT).await {
            let _ = Command::new(&self.cfg.container_bin).args(["stop", &name]).output().await;
            let _ = Command::new(&self.cfg.container_bin).args(["delete", &name]).output().await;
            return Err(e);
        }

        Ok(SandboxHandle {
            id,
            backend: "apple_container",
            root: PathBuf::from(format!("apple:{name}")),
            state: Arc::new(AcState {
                container_bin: self.cfg.container_bin.clone(),
                name,
                endpoint: Some(format!("http://{ip}:{SIDECAR_PORT}")),
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(st) = sandbox.state.downcast_ref::<AcState>() {
            let _ = Command::new(&st.container_bin).args(["stop", &st.name]).output().await;
            let _ = Command::new(&st.container_bin).args(["delete", &st.name]).output().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_from_inspect_parses_networks_address() {
        let j = r#"[{"networks":[{"address":"192.168.64.3/24"}]}]"#;
        assert_eq!(ip_from_inspect(j).as_deref(), Some("192.168.64.3"));
    }

    #[test]
    fn ip_from_inspect_handles_docker_shape() {
        let j = r#"[{"NetworkSettings":{"IPAddress":"10.0.0.5"}}]"#;
        assert_eq!(ip_from_inspect(j).as_deref(), Some("10.0.0.5"));
    }

    #[test]
    fn ip_from_inspect_none_on_unknown_shape() {
        assert!(ip_from_inspect(r#"[{"foo":1}]"#).is_none());
    }
}
