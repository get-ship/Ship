//! `SidecarChildBackend` — services the [`ChildBackend`] surface for
//! an actor whose process actually lives behind a sandbox boundary.
//!
//! Each method maps to a `ship.sidecar.v1.Processes` RPC. The trait
//! is sync; RPC calls are async; we bridge with `block_in_place +
//! block_on`. That requires the multi-thread tokio runtime, which
//! the supervisor already uses.
//!
//! Output bytes (stdout / PTY) flow from the sidecar's
//! `Processes.AttachPty` server-stream into the broadcast::Sender
//! that the owning [`ChildHandle`] holds. Subscribers go through
//! `ChildHandle::subscribe_output` exactly as for local backends.

use anyhow::{Result, anyhow};
use connectrpc::client::SharedHttp2Connection;
use std::sync::Arc;

use ship_sidecar_proto::{
    ProcessGetRequest, ProcessResizeTerminalRequest, ProcessState,
    ProcessStopRequest, ProcessWriteInputRequest, ProcessesClient, PtySize,
};
use process_supervisor::ChildBackend;

/// Sidecar-backed [`ChildBackend`]. Holds:
///
///   - the `ProcessesClient` that owns the wire to the in-sandbox
///     runtime (shared with the [`super::SidecarRuntime`] that
///     spawned this actor)
///   - the remote process id (opaque to the host)
///   - the tokio Handle used to bridge sync trait → async RPC
///
/// `pid()` returns `None` — the host doesn't see the in-sandbox
/// process's PID. `is_alive()` overrides the default `/proc` probe
/// with a `Processes.Get` round-trip.
pub struct SidecarChildBackend {
    client: Arc<ProcessesClient<SharedHttp2Connection>>,
    process_id: String,
    handle: tokio::runtime::Handle,
}

impl SidecarChildBackend {
    pub fn new(
        client: Arc<ProcessesClient<SharedHttp2Connection>>,
        process_id: String,
        handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            client,
            process_id,
            handle,
        }
    }

    /// Run a future on the captured runtime handle from a sync
    /// context. `block_in_place` allows blocking in a multi-thread
    /// runtime worker; outside a runtime it falls through to
    /// `Handle::block_on` directly.
    fn block_on<F: std::future::Future>(&self, fut: F) -> F::Output {
        // Detect whether we're being called from a runtime worker.
        // From outside any runtime context, block_in_place panics —
        // use Handle::block_on instead.
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| self.handle.block_on(fut))
        } else {
            self.handle.block_on(fut)
        }
    }
}

impl ChildBackend for SidecarChildBackend {
    fn write_input(&self, bytes: &[u8]) -> Result<()> {
        let req = ProcessWriteInputRequest {
            id: self.process_id.clone(),
            data: bytes.to_vec(),
            ..Default::default()
        };
        let client = self.client.clone();
        self.block_on(async move {
            client
                .write_input(req)
                .await
                .map_err(|e| anyhow!("Processes.WriteInput: {e}"))?;
            Ok::<_, anyhow::Error>(())
        })
    }

    fn stop(&self) -> Result<()> {
        // Stop = graceful (Processes.Stop with sidecar's default grace).
        // ChildHandle::stop callers that want hard-kill should use a
        // future force-kill API; today's PtyChildBackend stop also
        // sends a kill, so symmetry holds.
        let req = ProcessStopRequest { id: self.process_id.clone(), ..Default::default() };
        let client = self.client.clone();
        self.block_on(async move {
            client
                .stop(req)
                .await
                .map_err(|e| anyhow!("Processes.Stop: {e}"))?;
            Ok::<_, anyhow::Error>(())
        })
    }

    fn pid(&self) -> Option<u32> {
        // Sidecar-backed actors don't expose a host PID. Live state
        // comes from `is_alive` / `try_exit` via Get.
        None
    }

    fn try_exit(&self) -> Option<i32> {
        let req = ProcessGetRequest { id: self.process_id.clone(), ..Default::default() };
        let client = self.client.clone();
        let result = self.block_on(async move {
            client
                .get(req)
                .await
                .map(|r| r.into_view())
                .map_err(|e| anyhow!("Processes.Get: {e}"))
        });
        match result {
            Ok(view) => match view.state.as_known() {
                Some(ProcessState::PROCESS_STATE_EXITED)
                | Some(ProcessState::PROCESS_STATE_KILLED) => Some(view.exit_code),
                _ => None,
            },
            Err(_) => None,
        }
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let size = PtySize { cols: cols as u32, rows: rows as u32, ..Default::default() };
        let req = ProcessResizeTerminalRequest {
            id: self.process_id.clone(),
            size: ::buffa::MessageField::some(size),
            ..Default::default()
        };
        let client = self.client.clone();
        self.block_on(async move {
            client
                .resize_terminal(req)
                .await
                .map_err(|e| anyhow!("Processes.ResizeTerminal: {e}"))?;
            Ok::<_, anyhow::Error>(())
        })
    }

    fn is_alive(&self) -> bool {
        let req = ProcessGetRequest { id: self.process_id.clone(), ..Default::default() };
        let client = self.client.clone();
        let result = self.block_on(async move {
            client
                .get(req)
                .await
                .map(|r| r.into_view())
                .map_err(|e| anyhow!("Processes.Get: {e}"))
        });
        match result {
            Ok(view) => matches!(view.state.as_known(), Some(ProcessState::PROCESS_STATE_RUNNING)),
            Err(_) => false,
        }
    }
}

