//! Firecracker Tier-3 microVM `Isolator`.
//!
//! `prepare` boots a Firecracker microVM running `ship-sidecar` as init,
//! reachable over vsock; `shutdown` SIGKILLs it. The sidecar is constant —
//! the host drives the actor over `ship.sidecar.v1` exactly as for Docker,
//! only the substrate (own-kernel microVM) and transport (vsock via
//! firecracker's host uds) differ. Linux/KVM only.
//!
//! Transcribes the recipe proven in `docs/tier3-build.md`: a static-musl
//! sidecar baked into a rootfs as init on `vsock:<PORT>`; firecracker
//! booted with kernel + rootfs + a vsock device; the host dials the
//! firecracker uds with a `CONNECT <port>` preamble (see
//! `SidecarActorRuntime`'s `fcvsock:` transport).
//!
//! The rootfs baking + a pinned/signed image are a follow-up — today the
//! kernel + pre-baked rootfs paths are supplied via `FirecrackerConfig`.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use ship_substrate::isolation::{Isolator, IsolatorError, SandboxHandle, SandboxId, SandboxPolicy};

/// vsock port the in-guest sidecar listens on — matches the baked rootfs
/// init (`--addr vsock:2468`).
const SIDECAR_VSOCK_PORT: u32 = 2468;
/// Guest context id assigned to the microVM.
const GUEST_CID: u32 = 3;

#[derive(Debug, Clone)]
pub struct FirecrackerConfig {
    pub firecracker_bin: PathBuf,
    pub kernel: PathBuf,
    /// Pre-baked ext4 rootfs with `ship-sidecar` as init on vsock. Copied
    /// per-VM in `prepare` (a shared rw root would corrupt under concurrent
    /// VMs). Pinned/signed image baking is a follow-up.
    pub rootfs: PathBuf,
}

pub struct Firecracker {
    cfg: FirecrackerConfig,
}

struct FcState {
    child: Mutex<Option<Child>>,
    /// Temp dir holding this VM's rootfs copy, config, console log, and the
    /// vsock uds. Removed on shutdown.
    vm_dir: PathBuf,
    uds: PathBuf,
}

impl Firecracker {
    pub fn new(cfg: FirecrackerConfig) -> Self {
        Self { cfg }
    }

    /// Host endpoint for the in-guest sidecar: `fcvsock:<port>:<uds>`. The
    /// supervisor's `SidecarActorRuntime` dials it via the uds + CONNECT
    /// preamble. `None` on a downcast miss.
    pub fn sidecar_endpoint(sandbox: &SandboxHandle) -> Option<String> {
        let st = sandbox.state.downcast_ref::<FcState>()?;
        Some(format!("fcvsock:{SIDECAR_VSOCK_PORT}:{}", st.uds.display()))
    }
}

/// Poll until the in-guest sidecar accepts over the firecracker vsock uds
/// (connect + `CONNECT <port>` → `OK`), or the deadline elapses.
async fn wait_until_ready(uds: &Path) -> Result<(), IsolatorError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    while tokio::time::Instant::now() < deadline {
        if let Ok(mut s) = tokio::net::UnixStream::connect(uds).await {
            let _ = s
                .write_all(format!("CONNECT {SIDECAR_VSOCK_PORT}\n").as_bytes())
                .await;
            let mut b = [0u8; 2];
            if s.read_exact(&mut b).await.is_ok() && &b == b"OK" {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err(IsolatorError::Sidecar(format!(
        "microVM sidecar not reachable over vsock ({}) within 20s",
        uds.display()
    )))
}

#[async_trait]
impl Isolator for Firecracker {
    fn name(&self) -> &'static str {
        "firecracker"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        if !Path::new("/dev/kvm").exists() {
            return Err(IsolatorError::Unavailable("/dev/kvm not present".into()));
        }
        for (p, what) in [
            (&self.cfg.firecracker_bin, "firecracker binary"),
            (&self.cfg.kernel, "guest kernel"),
            (&self.cfg.rootfs, "rootfs"),
        ] {
            if !p.exists() {
                return Err(IsolatorError::Unavailable(format!(
                    "{what} not found at {}",
                    p.display()
                )));
            }
        }
        Ok(())
    }

    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let id = SandboxId::new();
        let vm_dir = std::env::temp_dir().join(format!("ship-fc-{}", id.0));
        tokio::fs::create_dir_all(&vm_dir).await?;
        let uds = vm_dir.join("vsock.uds");
        let console = vm_dir.join("console.log");
        // Per-VM rootfs copy: a shared rw root would corrupt under
        // concurrent microVMs.
        let rootfs = vm_dir.join("rootfs.ext4");
        tokio::fs::copy(&self.cfg.rootfs, &rootfs).await?;

        // Resource caps from policy (threat-model §7.3). Firecracker takes
        // whole vcpus + MiB; default 1 vcpu / 256 MiB.
        let mem_mib = policy
            .limits
            .memory_bytes
            .map(|b| (b / (1024 * 1024)).max(128))
            .unwrap_or(256);
        let vcpus = policy
            .limits
            .cpu_quota_micros
            .map(|q| (q / 100_000).max(1))
            .unwrap_or(1);

        let vm_json = serde_json::json!({
            "boot-source": {
                "kernel_image_path": self.cfg.kernel,
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw init=/init"
            },
            "drives": [{
                "drive_id": "rootfs",
                "path_on_host": rootfs,
                "is_root_device": true,
                "is_read_only": false
            }],
            "vsock": { "guest_cid": GUEST_CID, "uds_path": uds },
            "machine-config": { "vcpu_count": vcpus, "mem_size_mib": mem_mib }
        });
        let cfg_path = vm_dir.join("vm.json");
        let vm_bytes = serde_json::to_vec_pretty(&vm_json)
            .map_err(|e| IsolatorError::Other(e.into()))?;
        tokio::fs::write(&cfg_path, vm_bytes).await?;

        let log = std::fs::File::create(&console).map_err(IsolatorError::from)?;
        let child = Command::new(&self.cfg.firecracker_bin)
            .arg("--no-api")
            .arg("--config-file")
            .arg(&cfg_path)
            .stdout(Stdio::from(log.try_clone().map_err(IsolatorError::from)?))
            .stderr(Stdio::from(log))
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| IsolatorError::SpawnFailed(format!("firecracker spawn: {e}")))?;

        if let Err(e) = wait_until_ready(&uds).await {
            let mut c = child;
            let _ = c.kill().await;
            let tail = tokio::fs::read_to_string(&console).await.unwrap_or_default();
            let _ = tokio::fs::remove_dir_all(&vm_dir).await;
            return Err(IsolatorError::Sidecar(format!(
                "{e}; console tail: {}",
                tail.lines().rev().take(4).collect::<Vec<_>>().join(" | ")
            )));
        }

        Ok(SandboxHandle {
            id,
            backend: "firecracker",
            root: vm_dir.clone(),
            state: Arc::new(FcState {
                child: Mutex::new(Some(child)),
                vm_dir,
                uds,
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(st) = sandbox.state.downcast_ref::<FcState>() {
            if let Some(mut child) = st.child.lock().await.take() {
                let _ = child.kill().await;
            }
            let _ = tokio::fs::remove_dir_all(&st.vm_dir).await;
        }
        Ok(())
    }
}
