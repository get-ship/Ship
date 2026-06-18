//! ConnectRPC handlers for ship.sidecar.v1.
#![allow(refining_impl_trait)]
// Wire-builder style: proto responses are built default-then-assign; the
// struct-literal form buys no safety on these non-exhaustive generated types.
#![allow(clippy::field_reassign_with_default)]
//!
//! Three services wired to the generated traits:
//!   - `Handshake.Hello` — capability advertisement
//!   - `Processes.*`     — Create/Run/List/Get/Stop/Kill/WriteInput/ResizeTerminal/AttachPty
//!   - `Events.Stream`   — bidi envelope channel
//!
//! Generated trait signatures accept `OwnedFooView` requests and return
//! `impl Encodable<Out> + Send`. The blanket `impl<M: Message + Serialize>
//! Encodable<M> for M` lets us return owned response structs directly.

use crate::session::Session;
use connectrpc::{ConnectError, ErrorCode, RequestContext, Response, ServiceResult, ServiceStream};
use futures_util::StreamExt;
use ship_isolation::sidecar::proto::*;
extern crate portable_pty;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct HandshakeImpl;

impl handshake_service::Handshake for HandshakeImpl {
    async fn hello(
        &self,
        _ctx: RequestContext,
        _req: OwnedHelloRequestView,
    ) -> ServiceResult<HelloResponse> {
        let mut resp = HelloResponse::default();
        resp.protocol_version = "1.0".into();
        resp.sidecar_version = VERSION.into();
        resp.capabilities = vec![
            "processes".into(),
            "pty".into(),
            "events.acp".into(),
        ];
        Response::ok(resp)
    }
}

pub struct ProcessesImpl {
    pub session: Arc<Session>,
}

impl processes_service::Processes for ProcessesImpl {
    async fn create(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessCreateRequestView,
    ) -> ServiceResult<ProcessInfo> {
        let bin = req.bin.to_string();
        let args = req.args.iter().map(|s| s.to_string()).collect();
        let env = view_env_to_map(&req.env);
        let cwd = req.cwd.to_string();
        let id_hint = req.id_hint.to_string();

        let info = if req.pty {
            let size = pty_size_from_view(&req.pty_size);
            self.session
                .create_pty(bin, args, env, cwd, id_hint, size)
                .await
                .map_err(io_to_connect)?
        } else {
            self.session
                .create(bin, args, env, cwd, id_hint)
                .await
                .map_err(io_to_connect)?
        };
        Response::ok(info)
    }

    async fn run(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessRunRequestView,
    ) -> ServiceResult<ProcessRunResponse> {
        let bin = req.bin.to_string();
        let args = req.args.iter().map(|s| s.to_string()).collect();
        let env = view_env_to_map(&req.env);
        let cwd = req.cwd.to_string();
        let stdin = req.stdin.to_vec();
        let timeout = req.timeout_ms;
        let resp = self
            .session
            .run(bin, args, env, cwd, stdin, timeout)
            .await
            .map_err(io_to_connect)?;
        Response::ok(resp)
    }

    async fn list(
        &self,
        _ctx: RequestContext,
        _req: OwnedProcessListRequestView,
    ) -> ServiceResult<ProcessListResponse> {
        let procs = self.session.list().await;
        let mut out = ProcessListResponse::default();
        for p in procs {
            out.processes.push(p.info().await);
        }
        Response::ok(out)
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessGetRequestView,
    ) -> ServiceResult<ProcessInfo> {
        let proc = self.session.get(req.id).await.ok_or_else(|| {
            ConnectError::new(ErrorCode::NotFound, format!("no such process: {}", req.id))
        })?;
        Response::ok(proc.info().await)
    }

    async fn stop(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessStopRequestView,
    ) -> ServiceResult<ProcessActionResponse> {
        let id = req.id.to_string();
        let grace = req.grace_ms;
        self.session.signal(&id, libc::SIGTERM).await.map_err(io_to_connect)?;
        if grace > 0 {
            let session = self.session.clone();
            let id_for_kill = id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(grace)).await;
                let _ = session.force_kill(&id_for_kill).await;
            });
        }
        let mut resp = ProcessActionResponse::default();
        resp.ok = true;
        Response::ok(resp)
    }

    async fn kill(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessKillRequestView,
    ) -> ServiceResult<ProcessActionResponse> {
        self.session
            .signal(req.id, libc::SIGKILL)
            .await
            .map_err(io_to_connect)?;
        let mut resp = ProcessActionResponse::default();
        resp.ok = true;
        Response::ok(resp)
    }

    async fn write_input(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessWriteInputRequestView,
    ) -> ServiceResult<ProcessActionResponse> {
        self.session
            .write_input(req.id, req.data)
            .await
            .map_err(io_to_connect)?;
        let mut resp = ProcessActionResponse::default();
        resp.ok = true;
        Response::ok(resp)
    }

    async fn resize_terminal(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessResizeTerminalRequestView,
    ) -> ServiceResult<ProcessActionResponse> {
        let id = req.id.to_string();
        let size = pty_size_from_view(&req.size);
        self.session.resize_pty(&id, size).await.map_err(io_to_connect)?;
        let mut resp = ProcessActionResponse::default();
        resp.ok = true;
        Response::ok(resp)
    }

    async fn attach_pty(
        &self,
        _ctx: RequestContext,
        req: OwnedProcessAttachPtyRequestView,
    ) -> ServiceResult<ServiceStream<PtyChunk>> {
        let id = req.id.to_string();
        let proc = self.session.get(&id).await.ok_or_else(|| {
            ConnectError::new(ErrorCode::NotFound, format!("no such process: {id}"))
        })?;
        let pty = proc.pty.as_ref().ok_or_else(|| {
            ConnectError::new(
                ErrorCode::FailedPrecondition,
                "process is not pty-backed; pass pty=true to Create",
            )
        })?;
        let rx = pty.output.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| async move { r.ok().map(Ok) });
        let stream: ServiceStream<PtyChunk> = Box::pin(stream);
        Response::ok(stream)
    }
}

fn pty_size_from_view(
    view: &::buffa::MessageFieldView<
        ship_isolation::sidecar::proto::__buffa::view::PtySizeView<'_>,
    >,
) -> portable_pty::PtySize {
    let mut size = portable_pty::PtySize::default();
    if let Some(v) = view.as_option() {
        if v.cols > 0 {
            size.cols = v.cols as u16;
        }
        if v.rows > 0 {
            size.rows = v.rows as u16;
        }
    }
    size
}

pub struct EventsImpl {
    pub session: Arc<Session>,
}

impl events_service::Events for EventsImpl {
    async fn stream(
        &self,
        _ctx: RequestContext,
        requests: ServiceStream<OwnedEnvelopeView>,
    ) -> ServiceResult<ServiceStream<Envelope>> {
        // Drain client-initiated envelopes in the background. v1 routes
        // them to handlers as the secret/setting/permission brokers come
        // online; for now we count and trace.
        let mut requests = requests;
        let drain = async move {
            while let Some(item) = requests.next().await {
                match item {
                    Ok(env) => {
                        tracing::trace!(
                            seq = env.sequence,
                            request_id = %env.request_id,
                            "events.stream: client envelope received (no handler in v1.0)"
                        );
                    }
                    Err(e) => {
                        tracing::debug!(err = %e, "events.stream: client side closed");
                        break;
                    }
                }
            }
        };
        tokio::spawn(drain);

        let rx = self.session.events.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| async move { r.ok().map(Ok) });
        let stream: ServiceStream<Envelope> = Box::pin(stream);
        Response::ok(stream)
    }
}

fn view_env_to_map(view: &::buffa::MapView<'_, &str, &str>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in view.iter() {
        map.insert(k.to_string(), v.to_string());
    }
    map
}

fn io_to_connect(e: std::io::Error) -> ConnectError {
    let code = match e.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::NotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        std::io::ErrorKind::BrokenPipe => ErrorCode::FailedPrecondition,
        _ => ErrorCode::Internal,
    };
    ConnectError::new(code, e.to_string())
}

// Aliases for the generated trait modules. The codegen exports them at
// the package root (no per-service module nesting). Re-exporting under
// short aliases keeps `impl handshake_service::Handshake for ...` etc.
pub mod handshake_service {
    pub use ship_isolation::sidecar::proto::Handshake;
}
pub mod processes_service {
    pub use ship_isolation::sidecar::proto::Processes;
}
pub mod events_service {
    pub use ship_isolation::sidecar::proto::Events;
}
