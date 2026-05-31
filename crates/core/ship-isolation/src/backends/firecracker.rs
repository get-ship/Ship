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
/// vsock port the in-guest app-plane bridge dials on the host (guest→host).
/// Firecracker forwards a guest connection to `(host, this)` to the host
/// unix socket `<uds>_<this>`, where this Isolator listens and splices to
/// the daemon. Matches the sidecar's `--app-plane <local>:2469`.
const APP_PLANE_VSOCK_PORT: u32 = 2469;

#[derive(Debug, Clone)]
pub struct FirecrackerConfig {
    pub firecracker_bin: PathBuf,
    pub kernel: PathBuf,
    /// Pre-baked ext4 rootfs with `ship-sidecar` as init on vsock. Copied
    /// per-VM in `prepare` (a shared rw root would corrupt under concurrent
    /// VMs). Pinned/signed image baking is a follow-up.
    pub rootfs: PathBuf,
    /// Daemon `host:port` to splice the in-VM app plane to (so the agent can
    /// reach MCP / hooks / log_progress despite the microVM having no network
    /// device). `None` = no app-plane bridge. The supervisor sets this from
    /// the daemon origin.
    pub app_plane_daemon: Option<String>,
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
    /// Work-unit-copy workspace: `(data_image, host_workspace_dir)`. The
    /// image is seeded from the host dir at prepare and copied back at
    /// shutdown (Firecracker has no live shared fs — see tier3-build.md).
    workspace: Option<(PathBuf, PathBuf)>,
    /// App-plane bridge accept-loop (host side); aborted on shutdown.
    app_plane: Option<tokio::task::JoinHandle<()>>,
}

/// App-plane bridge (host side): firecracker forwards each guest→host vsock
/// connection on `APP_PLANE_VSOCK_PORT` to the unix socket `<uds>_<port>`.
/// Accept on an already-bound listener and splice every connection to the
/// daemon's `host:port`. The listener is bound in `prepare` (before it
/// returns) so it's up before the agent runs.
async fn accept_host_app_plane(listener: tokio::net::UnixListener, daemon: String) {
    loop {
        let Ok((mut ux, _)) = listener.accept().await else {
            break;
        };
        let daemon = daemon.clone();
        tokio::spawn(async move {
            match tokio::net::TcpStream::connect(&daemon).await {
                Ok(mut tcp) => {
                    let _ = tokio::io::copy_bidirectional(&mut ux, &mut tcp).await;
                }
                Err(e) => tracing::warn!("firecracker app-plane daemon dial {daemon}: {e}"),
            }
        });
    }
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

        let mut boot_args =
            "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw init=/init".to_string();
        let mut drives = vec![serde_json::json!({
            "drive_id": "rootfs",
            "path_on_host": rootfs,
            "is_root_device": true,
            "is_read_only": false
        })];

        // Work-unit-copy workspace: seed an ext4 from the policy's workspace
        // mount (rootless `mke2fs -d`), attach as /dev/vdb, and tell the
        // guest init where to mount it. Firecracker has no virtio-fs, so
        // this is snapshot-in / copy-out, not a live mount.
        let workspace = match policy.mounts.first() {
            Some(m) => {
                let data = vm_dir.join("workspace.ext4");
                let out = Command::new("mke2fs")
                    .args(["-q", "-t", "ext4", "-d"])
                    .arg(&m.host_path)
                    .arg(&data)
                    .arg("512M")
                    .output()
                    .await
                    .map_err(|e| IsolatorError::SpawnFailed(format!("mke2fs: {e}")))?;
                if !out.status.success() {
                    let _ = tokio::fs::remove_dir_all(&vm_dir).await;
                    return Err(IsolatorError::SpawnFailed(format!(
                        "mke2fs workspace: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    )));
                }
                drives.push(serde_json::json!({
                    "drive_id": "workspace",
                    "path_on_host": data,
                    "is_root_device": false,
                    "is_read_only": false
                }));
                boot_args.push_str(&format!(" ship.workspace={}", m.sandbox_path.display()));
                Some((data, m.host_path.clone()))
            }
            None => None,
        };

        let vm_json = serde_json::json!({
            "boot-source": {
                "kernel_image_path": self.cfg.kernel,
                "boot_args": boot_args
            },
            "drives": drives,
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

        // App-plane bridge: bind the firecracker guest→host vsock uds for
        // this port (synchronously, so it's up before the agent runs) and
        // splice each connection to the daemon — the in-VM agent reaches
        // MCP / hooks / log_progress over `http://127.0.0.1:<local>`.
        let app_plane = if let Some(daemon) = self.cfg.app_plane_daemon.clone() {
            let sock = PathBuf::from(format!("{}_{}", uds.display(), APP_PLANE_VSOCK_PORT));
            let _ = std::fs::remove_file(&sock);
            match tokio::net::UnixListener::bind(&sock) {
                Ok(listener) => Some(tokio::spawn(accept_host_app_plane(listener, daemon))),
                Err(e) => {
                    tracing::warn!("firecracker app-plane bind {} failed: {e}", sock.display());
                    None
                }
            }
        } else {
            None
        };

        Ok(SandboxHandle {
            id,
            backend: "firecracker",
            root: vm_dir.clone(),
            state: Arc::new(FcState {
                child: Mutex::new(Some(child)),
                vm_dir,
                uds,
                workspace,
                app_plane,
            }),
        })
    }

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        if let Some(st) = sandbox.state.downcast_ref::<FcState>() {
            if let Some(h) = &st.app_plane {
                h.abort();
            }
            // Firecracker buffers guest block writes until a virtio flush;
            // the guest runs a 1s periodic `sync`. Before the hard stop,
            // wait just over one interval so the actor's final workspace
            // edits flush to the data image, then copy back. (A cooperative
            // ACPI graceful-shutdown is the refinement — see tier3-build.md.)
            if st.workspace.is_some() {
                tokio::time::sleep(Duration::from_millis(1300)).await;
            }
            if let Some(mut child) = st.child.lock().await.take() {
                let _ = child.kill().await;
            }
            // Copy the workspace back out (the VM is stopped → the data
            // image is consistent). `debugfs rdump` extracts rootlessly.
            // Caveat: rdump writes the image's tree over the host dir; it
            // doesn't propagate deletions (documented v1 limitation).
            if let Some((data, host_ws)) = &st.workspace {
                let _ = Command::new("debugfs")
                    .arg("-R")
                    .arg(format!("rdump / {}", host_ws.display()))
                    .arg(data)
                    .output()
                    .await;
            }
            let _ = tokio::fs::remove_dir_all(&st.vm_dir).await;
        }
        Ok(())
    }
}
