//! WSL `Isolator` — the Windows substrate (the tiers-strategy entry for
//! Windows, symmetric with Lima on mac and Firecracker on Linux).
//!
//! Windows has no native OS-level sandbox primitive, so the Windows path is
//! a Linux substrate: Ship ships a dedicated WSL2 distro (a published rootfs
//! tarball, imported as `ship-substrate`) with `ship-sidecar` + bubblewrap
//! baked in. `shipd` runs natively on Windows and reaches the in-distro
//! sidecar through `wsl.exe`; the supervisor drives the actor over
//! `ship.sidecar.v1` exactly as for Docker/Firecracker/Lima — only the
//! substrate (a WSL distro) and the launch mechanism differ.
//!
//! Transport: plain TCP. The sidecar binds `0.0.0.0:<port>` in the distro;
//! WSL2 forwards distro ports to the Windows host's `127.0.0.1`, so the
//! supervisor (shipd, native on Windows) reaches it there. Maps onto the
//! sidecar's TCP listener and `SidecarRuntime`'s plaintext-http client —
//! no vsock shim.
//!
//! POC-verified on a WSL2 box: `wsl.exe -d <distro> -- ship-sidecar` runs the
//! sidecar in-distro and `Test-NetConnection 127.0.0.1 <port>` from the
//! Windows side returns True. Crucial finding: WSL2's localhost forwarding is
//! **host↔distro, not sibling-distro↔distro** — a supervisor in a *different*
//! WSL distro cannot reach the target distro's port. That's fine: production
//! shipd is on the Windows host. It does mean the Rust live cell
//! (`wsl_sidecar_live`) only goes green when run *on Windows*.
//!
//! **Never touches the user's own distro.** The substrate distro name is
//! explicit config; Ship imports its own and runs only there.
//!
//! Runs on Windows native (where `ship.exe` orchestrates) and from inside
//! WSL2 via interop (`/mnt/c/Windows/System32/wsl.exe`), which is how it's
//! live-tested against a throwaway distro.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use isolation_port::{Isolator, IsolatorError, SandboxHandle, SandboxId, SandboxPolicy};

#[derive(Debug, Clone)]
pub struct WslConfig {
    /// Path to `wsl.exe`. On Windows it's on PATH; from inside WSL2 via
    /// interop at `/mnt/c/Windows/System32/wsl.exe`.
    pub wsl_bin: PathBuf,
    /// The dedicated Ship substrate distro — Ship imports this from a
    /// published rootfs tarball. NEVER the user's own distro.
    pub distro: String,
    /// Absolute path to the sidecar binary *inside the distro* (baked into
    /// the substrate rootfs).
    pub sidecar_path: String,
}

pub struct Wsl {
    cfg: WslConfig,
}

struct WslState {
    /// The `wsl.exe` child running the in-distro sidecar.
    child: Mutex<Option<Child>>,
    wsl_bin: PathBuf,
    distro: String,
    /// Loopback port the sidecar bound inside the distro (reachable on the
    /// shared WSL2 network namespace / Windows localhost forwarding).
    port: u16,
    /// Temp dir holding the sidecar console log; removed on shutdown.
    log_dir: PathBuf,
}

impl Wsl {
    pub fn new(cfg: WslConfig) -> Self {
        Self { cfg }
    }

    /// Host endpoint for the in-distro sidecar: plaintext http over the
    /// shared loopback. `None` on a downcast miss.
    pub fn sidecar_endpoint(sandbox: &SandboxHandle) -> Option<String> {
        let st = sandbox.state.downcast_ref::<WslState>()?;
        Some(format!("http://127.0.0.1:{}", st.port))
    }
}

/// Reserve an ephemeral loopback port (bind :0, read it back, drop). The
/// in-distro sidecar binds the same port on the shared namespace.
fn reserve_port() -> Result<u16, IsolatorError> {
    let l = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(l.local_addr()?.port())
}

/// Poll until the in-distro sidecar accepts on the shared loopback, or the
/// deadline elapses.
async fn wait_until_ready(port: u16) -> Result<(), IsolatorError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err(IsolatorError::Sidecar(format!(
        "WSL sidecar not reachable on 127.0.0.1:{port} within 30s"
    )))
}

#[async_trait]
impl Isolator for Wsl {
    fn name(&self) -> &'static str {
        "wsl2"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        // Only file-check an absolute path; a bare `wsl.exe` resolves on PATH.
        if self.cfg.wsl_bin.is_absolute() && !self.cfg.wsl_bin.exists() {
            return Err(IsolatorError::Unavailable(format!(
                "wsl.exe not found at {}",
                self.cfg.wsl_bin.display()
            )));
        }
        // The substrate distro must be registered — Ship imports it from the
        // published rootfs; this Isolator does not auto-import.
        let out = Command::new(&self.cfg.wsl_bin)
            .args(["-d", &self.cfg.distro, "--", "/bin/busybox", "true"])
            .output()
            .await
            .map_err(|e| IsolatorError::Unavailable(format!("wsl exec: {e}")))?;
        if !out.status.success() {
            return Err(IsolatorError::Unavailable(format!(
                "WSL substrate distro '{}' not available",
                self.cfg.distro
            )));
        }
        Ok(())
    }

    async fn prepare(&self, _policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let id = SandboxId::new();
        let port = reserve_port()?;
        let log_dir = std::env::temp_dir().join(format!("ship-wsl-{}", id.0));
        tokio::fs::create_dir_all(&log_dir).await?;
        let console = log_dir.join("sidecar.log");
        let log = std::fs::File::create(&console)?;

        // Launch the sidecar inside the substrate distro, bound 0.0.0.0 so
        // it's reachable across the shared WSL2 namespace + Windows localhost
        // forwarding.
        let child = Command::new(&self.cfg.wsl_bin)
            .args([
                "-d",
                &self.cfg.distro,
                "--",
                &self.cfg.sidecar_path,
                "--addr",
                &format!("0.0.0.0:{port}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| IsolatorError::SpawnFailed(format!("wsl sidecar spawn: {e}")))?;

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
            backend: "wsl2",
            root: log_dir.clone(),
            state: Arc::new(WslState {
                child: Mutex::new(Some(child)),
                wsl_bin: self.cfg.wsl_bin.clone(),
                distro: self.cfg.distro.clone(),
                port,
                log_dir,
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(st) = sandbox.state.downcast_ref::<WslState>() {
            if let Some(mut child) = st.child.lock().await.take() {
                let _ = child.kill().await;
            }
            // Best-effort sweep of any lingering sidecar in the distro. We
            // NEVER `--terminate`/`--unregister` the distro: it's reused, and
            // it is never the user's own. (Precise per-process kill in a
            // shared distro is a refinement.)
            let _ = Command::new(&st.wsl_bin)
                .args(["-d", &st.distro, "--", "/bin/busybox", "pkill", "-f", "ship-sidecar"])
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
        let st = WslState {
            child: Mutex::new(None),
            wsl_bin: PathBuf::from("/mnt/c/Windows/System32/wsl.exe"),
            distro: "ship-substrate".into(),
            port: 51888,
            log_dir: PathBuf::from("/tmp/ship-wsl-x"),
        };
        let handle = SandboxHandle {
            id: SandboxId::new(),
            backend: "wsl2",
            root: PathBuf::from("/tmp/ship-wsl-x"),
            state: Arc::new(st),
        };
        assert_eq!(
            Wsl::sidecar_endpoint(&handle).as_deref(),
            Some("http://127.0.0.1:51888")
        );
    }
}
