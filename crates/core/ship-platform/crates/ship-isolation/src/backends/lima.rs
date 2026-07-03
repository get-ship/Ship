//! Lima Tier-3 `Isolator` — the macOS substrate (also runs on Linux).
//!
//! A Mac has no Linux kernel to run our Isolators in; Lima *provides* one —
//! it boots a Linux VM (Virtualization.framework / QEMU on mac, QEMU on
//! Linux) and the sidecar + tiers run inside, same as in a container or a
//! WSL distro. `prepare` launches `ship-sidecar` in a managed Lima instance
//! via `limactl shell`; the host drives the actor over `ship.sidecar.v1`
//! exactly as for Docker/Firecracker — only the substrate (a Lima VM) and
//! the launch mechanism differ.
//!
//! Transport: plain TCP over Lima's loopback port-forwarding. The sidecar
//! binds `0.0.0.0:<port>` in the guest — Lima forwards guest `0.0.0.0` ports
//! to the host's `127.0.0.1`, but treats `127.0.0.1` guest ports as private
//! and does NOT forward them. That maps onto the sidecar's TCP listener and
//! `SidecarRuntime`'s plaintext-http client — no vsock shim. Ship's
//! Lima substrate template MUST declare an explicit `portForwards` range
//! covering the sidecar ports: the event-driven auto-forward is environment
//! dependent (fine on native macOS, unreliable under nested virt — verified
//! on a WSL2 POC where Lima boots but auto-forward doesn't fire).
//!
//! The instance is *managed out of band* (`limactl start <template>` with the
//! Ship substrate image, started once and reused) — this Isolator does not
//! create or delete it, only runs the sidecar inside and reaps that process.
//!
//! Structurally identical to the Windows/WSL launcher (drive a managed Linux
//! substrate through a host CLI over forwarded loopback); if both ship, the
//! two collapse to one generic "CLI substrate" Isolator. Kept concrete until
//! the second consumer is real.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use ship_substrate::isolation::{Isolator, IsolatorError, SandboxHandle, SandboxId, SandboxPolicy};

#[derive(Debug, Clone)]
pub struct LimaConfig {
    /// Path to `limactl`. On mac it's a brew install; on Linux a GitHub
    /// release binary.
    pub limactl_bin: PathBuf,
    /// The managed Lima instance to run the sidecar in (started + reused out
    /// of band from the Ship substrate template).
    pub instance: String,
    /// Absolute path to the sidecar binary *inside the instance* (baked into
    /// the substrate image, or mounted).
    pub sidecar_path: String,
}

pub struct Lima {
    cfg: LimaConfig,
}

struct LimaState {
    /// The `limactl shell` child running the in-VM sidecar.
    child: Mutex<Option<Child>>,
    limactl_bin: PathBuf,
    instance: String,
    /// Loopback port the sidecar bound inside the VM (Lima forwards it to the
    /// host's `127.0.0.1`).
    port: u16,
    /// Temp dir holding the sidecar console log; removed on shutdown.
    log_dir: PathBuf,
}

impl Lima {
    pub fn new(cfg: LimaConfig) -> Self {
        Self { cfg }
    }

    /// Host endpoint for the in-VM sidecar: plaintext http over the forwarded
    /// loopback port. `None` on a downcast miss.
    pub fn sidecar_endpoint(sandbox: &SandboxHandle) -> Option<String> {
        let st = sandbox.state.downcast_ref::<LimaState>()?;
        Some(format!("http://127.0.0.1:{}", st.port))
    }

    /// Rewrite a daemon origin to a guest-reachable URL. Lima exposes the
    /// host to the guest as `host.lima.internal`, so the in-VM agent reaches
    /// the daemon there (mirrors Docker's `host.docker.internal`).
    pub fn guest_reachable_url(origin: &str) -> String {
        origin
            .replace("127.0.0.1", "host.lima.internal")
            .replace("localhost", "host.lima.internal")
    }
}

/// Reserve an ephemeral loopback port (bind :0, read it back, drop). The
/// in-VM sidecar binds the same port, which Lima forwards to the host.
fn reserve_port() -> Result<u16, IsolatorError> {
    let l = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(l.local_addr()?.port())
}

/// Poll until the in-VM sidecar accepts on the forwarded loopback, or the
/// deadline elapses. Lima's port-forward can lag VM boot, so the window is
/// generous.
async fn wait_until_ready(port: u16) -> Result<(), IsolatorError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(IsolatorError::Sidecar(format!(
        "Lima sidecar not reachable on 127.0.0.1:{port} within 45s"
    )))
}

#[async_trait]
impl Isolator for Lima {
    fn name(&self) -> &'static str {
        "lima"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        // Only file-check an absolute path; a bare `limactl` is resolved on
        // PATH by the subsequent `limactl list` (which fails → Unavailable if
        // it's not installed).
        if self.cfg.limactl_bin.is_absolute() && !self.cfg.limactl_bin.exists() {
            return Err(IsolatorError::Unavailable(format!(
                "limactl not found at {}",
                self.cfg.limactl_bin.display()
            )));
        }
        // The instance must already be Running — this Isolator runs the
        // sidecar inside it, it does not create the VM.
        let out = Command::new(&self.cfg.limactl_bin)
            .args(["list", &self.cfg.instance, "--format", "{{.Status}}"])
            .output()
            .await
            .map_err(|e| IsolatorError::Unavailable(format!("limactl list: {e}")))?;
        let status = String::from_utf8_lossy(&out.stdout);
        if !status.trim().eq_ignore_ascii_case("Running") {
            return Err(IsolatorError::Unavailable(format!(
                "Lima instance '{}' is not Running (status: {})",
                self.cfg.instance,
                status.trim()
            )));
        }
        Ok(())
    }

    async fn prepare(&self, _policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let id = SandboxId::new();
        let port = reserve_port()?;
        let log_dir = std::env::temp_dir().join(format!("ship-lima-{}", id.0));
        tokio::fs::create_dir_all(&log_dir).await?;
        let console = log_dir.join("sidecar.log");
        let log = std::fs::File::create(&console)?;

        // Launch the sidecar inside the instance, bound to the reserved
        // loopback port; Lima forwards it to the host's 127.0.0.1.
        let child = Command::new(&self.cfg.limactl_bin)
            .args([
                "shell",
                &self.cfg.instance,
                &self.cfg.sidecar_path,
                "--addr",
                // 0.0.0.0 (not 127.0.0.1): Lima only forwards 0.0.0.0 guest
                // ports to the host loopback. The guest is the isolated VM,
                // so binding all-interfaces there is not a host exposure.
                &format!("0.0.0.0:{port}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| IsolatorError::SpawnFailed(format!("limactl shell spawn: {e}")))?;

        if let Err(e) = wait_until_ready(port).await {
            let mut c = child;
            let _ = c.kill().await;
            let tail = tokio::fs::read_to_string(&console).await.unwrap_or_default();
            let _ = tokio::fs::remove_dir_all(&log_dir).await;
            return Err(IsolatorError::Sidecar(format!(
                "{e}; sidecar log tail: {}",
                tail.lines().rev().take(4).collect::<Vec<_>>().join(" | ")
            )));
        }

        Ok(SandboxHandle {
            id,
            backend: "lima",
            root: log_dir.clone(),
            state: Arc::new(LimaState {
                child: Mutex::new(Some(child)),
                limactl_bin: self.cfg.limactl_bin.clone(),
                instance: self.cfg.instance.clone(),
                port,
                log_dir,
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(st) = sandbox.state.downcast_ref::<LimaState>() {
            if let Some(mut child) = st.child.lock().await.take() {
                let _ = child.kill().await;
            }
            // Best-effort sweep of any lingering sidecar in the VM. We never
            // stop or delete the instance itself — it's reused.
            let _ = Command::new(&st.limactl_bin)
                .args(["shell", &st.instance, "pkill", "-f", "ship-sidecar"])
                .output()
                .await;
            let _ = tokio::fs::remove_dir_all(&st.log_dir).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_port_returns_nonzero() {
        assert!(reserve_port().unwrap() > 0);
    }

    #[test]
    fn endpoint_is_plaintext_http_loopback() {
        let st = LimaState {
            child: Mutex::new(None),
            limactl_bin: PathBuf::from("/usr/local/bin/limactl"),
            instance: "ship-substrate".into(),
            port: 52111,
            log_dir: PathBuf::from("/tmp/ship-lima-x"),
        };
        let handle = SandboxHandle {
            id: SandboxId::new(),
            backend: "lima",
            root: PathBuf::from("/tmp/ship-lima-x"),
            state: Arc::new(st),
        };
        assert_eq!(
            Lima::sidecar_endpoint(&handle).as_deref(),
            Some("http://127.0.0.1:52111")
        );
    }
}
