//! ConnectRPC handlers for ship.sidecar.v1.
#![allow(refining_impl_trait)]
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
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct HandshakeImpl;

impl handshake_service::Handshake for HandshakeImpl {
    async fn hello<'a>(
        &'a self,
        _ctx: RequestContext,
        _req: OwnedHelloRequestView,
    ) -> ServiceResult<HelloResponse> {
        let mut resp = HelloResponse::default();
        resp.protocol_version = "1.0".into();
        resp.sidecar_version = VERSION.into();
        resp.capabilities = vec!["processes".into(), "events.acp".into()];
        Response::ok(resp)
    }
}

pub struct ProcessesImpl {
    pub session: Arc<Session>,
}

impl processes_service::Processes for ProcessesImpl {
    async fn create<'a>(
        &'a self,
        _ctx: RequestContext,
        req: OwnedProcessCreateRequestView,
    ) -> ServiceResult<ProcessInfo> {
        if req.pty {
            return Err(ConnectError::new(
                ErrorCode::Unimplemented,
                "pty processes are not implemented in v1.0",
            ));
        }
        let bin = req.bin.to_string();
        let args = req.args.iter().map(|s| s.to_string()).collect();
        let env = view_env_to_map(&req.env);
        let cwd = req.cwd.to_string();
        let id_hint = req.id_hint.to_string();

        let info = self
            .session
            .create(bin, args, env, cwd, id_hint)
            .await
            .map_err(io_to_connect)?;
        Response::ok(info)
    }

    async fn run<'a>(
        &'a self,
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

    async fn list<'a>(
        &'a self,
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

    async fn get<'a>(
        &'a self,
        _ctx: RequestContext,
        req: OwnedProcessGetRequestView,
    ) -> ServiceResult<ProcessInfo> {
        let proc = self.session.get(req.id).await.ok_or_else(|| {
            ConnectError::new(ErrorCode::NotFound, format!("no such process: {}", req.id))
        })?;
        Response::ok(proc.info().await)
    }

    async fn stop<'a>(
        &'a self,
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

    async fn kill<'a>(
        &'a self,
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

    async fn write_input<'a>(
        &'a self,
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

    async fn resize_terminal<'a>(
        &'a self,
        _ctx: RequestContext,
        _req: OwnedProcessResizeTerminalRequestView,
    ) -> ServiceResult<ProcessActionResponse> {
        Err(ConnectError::new(
            ErrorCode::Unimplemented,
            "pty resize is not supported in v1.0 (no pty-backed processes)",
        ))
    }

    async fn attach_pty(
        &self,
        _ctx: RequestContext,
        _req: OwnedProcessAttachPtyRequestView,
    ) -> ServiceResult<ServiceStream<PtyChunk>> {
        Err(ConnectError::new(
            ErrorCode::Unimplemented,
            "pty attach is not supported in v1.0 (no pty-backed processes)",
        ))
    }
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
