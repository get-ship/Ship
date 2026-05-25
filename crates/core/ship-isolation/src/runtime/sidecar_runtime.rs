//! `SidecarActorRuntime` — the spawn-side of the sidecar runtime
//! pair. Implements [`ship_substrate::supervisor::ActorRuntime`] by
//! routing `spawn(profile)` through `Processes.Create` against an
//! in-sandbox runtime's v1 endpoint.
//!
//! Spawn flow:
//!   1. Build a `ProcessCreateRequest` from the [`ActorProfile`]
//!      (bin, args, env, cwd, pty flag from `TransportKind`).
//!   2. Call `Processes.Create` to allocate the remote process.
//!   3. Build a [`SidecarChildBackend`] holding the client + remote
//!      process id + tokio handle.
//!   4. Spawn a background task that drives `Processes.AttachPty`
//!      (for PTY-backed actors) or `Events.Stream` (for the rest),
//!      feeding bytes into the broadcast channel that
//!      [`ChildHandle`] owns.
//!   5. Wrap everything in [`ChildHandle`] and an [`ActorHandle`].
//!
//! What this does NOT do (yet):
//!   - Use `SandboxRegistry` for endpoint discovery — caller passes
//!     the URL directly. Chunk C3 wires the registry-driven dispatch.
//!   - Emit kernel events (`actor.created` etc.) — substrate's
//!     in-process `spawn_actor` does that for its path. Sidecar-side
//!     event emission folds in when the orchestrator dispatches
//!     uniformly through `Arc<dyn ActorRuntime>`.

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use connectrpc::client::{ClientConfig, Http2Connection, SharedHttp2Connection};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

use ship_sidecar_proto::{
    EventsClient, ProcessAttachPtyRequest, ProcessCreateRequest, ProcessesClient, PtySize,
};
use ship_substrate::supervisor::{
    ActorHandle, ActorProfile, ActorRuntime, ChildHandle, TransportKind,
};

use super::sidecar_backend::SidecarChildBackend;

const OUTPUT_CAP: usize = 1024;

/// Spawn-side runtime for sidecar-backed actors. Holds the wire
/// configuration; opens connections per-actor (cheap — they're
/// pooled by the HTTP/2 connection layer internally).
pub struct SidecarActorRuntime {
    endpoint: String,
    label: String,
}

impl SidecarActorRuntime {
    /// Create a runtime that spawns actors against a remote
    /// in-sandbox runtime at `endpoint` (e.g. `"http://127.0.0.1:2468"`
    /// or `"http://host.docker.internal:2468"`).
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        let label = format!("sidecar:{endpoint}");
        Self { endpoint, label }
    }

    /// Convenience: wrap as a trait object for the
    /// SandboxRegistry-driven dispatch path (Chunk C3).
    pub fn shared(endpoint: impl Into<String>) -> Arc<dyn ActorRuntime> {
        Arc::new(Self::new(endpoint))
    }

    fn build_clients(
        &self,
    ) -> Result<(
        ProcessesClient<SharedHttp2Connection>,
        EventsClient<SharedHttp2Connection>,
    )> {
        // The control plane is HTTP/2 (h2c) over either a unix socket — the
        // hardened container/VM path, `unix:<sock>` — or plaintext TCP
        // (in-process / dev / cloud-TLS-terminated). connectrpc's unix
        // support lives on `Http2Connection`; `SharedHttp2Connection` wraps
        // either so the typed clients share one transport type.
        const CONN_BUFFER: usize = 1024;
        let (conn, uri) = if let Some(rest) = self.endpoint.strip_prefix("fcvsock:") {
            // Tier-3 Firecracker: the host reaches the in-guest sidecar's
            // vsock via firecracker's host-side uds — connect to the uds and
            // send `CONNECT <port>\n`, then speak Connect over the bridged
            // stream. Endpoint shape: `fcvsock:<port>:<uds_path>`.
            let (port_s, uds) = rest
                .split_once(':')
                .ok_or_else(|| anyhow!("fcvsock endpoint must be fcvsock:<port>:<uds>"))?;
            let port: u32 = port_s
                .parse()
                .map_err(|e| anyhow!("fcvsock port: {e}"))?;
            let uds = std::path::PathBuf::from(uds);
            let authority: ::http::Uri = "http://ship-sidecar.local"
                .parse()
                .map_err(|e: ::http::uri::InvalidUri| anyhow!("sidecar authority: {e}"))?;
            let connector = tower::service_fn(move |_uri: ::http::Uri| {
                let uds = uds.clone();
                async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut s = tokio::net::UnixStream::connect(&uds).await?;
                    s.write_all(format!("CONNECT {port}\n").as_bytes()).await?;
                    // firecracker acks with `OK <host_port>\n` on success.
                    let mut line = Vec::new();
                    let mut b = [0u8; 1];
                    loop {
                        s.read_exact(&mut b).await?;
                        if b[0] == b'\n' {
                            break;
                        }
                        line.push(b[0]);
                    }
                    if !line.starts_with(b"OK") {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::ConnectionRefused,
                            format!(
                                "firecracker vsock CONNECT refused: {}",
                                String::from_utf8_lossy(&line)
                            ),
                        ));
                    }
                    Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(s))
                }
            });
            (
                Http2Connection::lazy_with_connector(connector, authority.clone())
                    .shared(CONN_BUFFER),
                authority,
            )
        } else if let Some(path) = self.endpoint.strip_prefix("unix:") {
            // `:authority` is cosmetic over a unix socket; a stable
            // placeholder keeps request-path construction valid.
            let authority: ::http::Uri = "http://ship-sidecar.local"
                .parse()
                .map_err(|e: ::http::uri::InvalidUri| anyhow!("sidecar authority: {e}"))?;
            (
                Http2Connection::lazy_unix(path, authority.clone()).shared(CONN_BUFFER),
                authority,
            )
        } else {
            let uri: ::http::Uri = self
                .endpoint
                .parse()
                .map_err(|e: ::http::uri::InvalidUri| anyhow!("sidecar endpoint: {e}"))?;
            (
                Http2Connection::lazy_plaintext(uri.clone()).shared(CONN_BUFFER),
                uri,
            )
        };
        let config = ClientConfig::new(uri);
        Ok((
            ProcessesClient::new(conn.clone(), config.clone()),
            EventsClient::new(conn, config),
        ))
    }
}

#[async_trait]
impl ActorRuntime for SidecarActorRuntime {
    async fn spawn(&self, profile: ActorProfile) -> Result<ActorHandle> {
        let actor_id = profile.resolve_actor_id();
        let (procs, _events) = self.build_clients()?;
        let procs = Arc::new(procs);

        // Build the Create request from the profile.
        let pty = matches!(profile.transport, TransportKind::Pty);
        let pty_size = if pty {
            ::buffa::MessageField::some(PtySize { cols: 120, rows: 24, ..Default::default() })
        } else {
            ::buffa::MessageField::default()
        };
        let req = ProcessCreateRequest {
            bin: profile.bin.clone(),
            args: profile.args.clone(),
            env: profile.env.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            cwd: profile.work_dir.to_string_lossy().into_owned(),
            pty,
            pty_size,
            id_hint: actor_id.clone(),
            ..Default::default()
        };

        let info = procs
            .create(req)
            .await
            .map_err(|e| anyhow!("Processes.Create: {e}"))?;
        let process_id = info.into_view().id.to_string();

        // Log path. For Tier 2+ this should land in a host-visible
        // location (bind-mounted log dir) so existing log-attach UIs
        // keep working. Today we just stamp the supervisor's chosen
        // path; the in-sandbox runtime is responsible for actually
        // writing there via its own log tee (which we'll wire when
        // the in-sandbox runtime's logger lands).
        let log_dir = profile.log_dir.clone().unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&log_dir).ok();
        let log_path = log_dir.join(format!("{actor_id}.log"));

        let (output_tx, _) = broadcast::channel(OUTPUT_CAP);

        // PTY-backed actors: subscribe to AttachPty and push bytes
        // into the broadcast. Non-PTY actors will use Events.Stream
        // for ProcessOutput envelopes once we wire that path (the
        // proto already has it; consumer just isn't built here yet
        // because most production paths use PTY).
        if matches!(profile.transport, TransportKind::Pty) {
            let attach_req =
                ProcessAttachPtyRequest { id: process_id.clone(), ..Default::default() };
            let attach_tx = output_tx.clone();
            let procs_for_attach = procs.clone();
            tokio::spawn(async move {
                let mut stream = match procs_for_attach.attach_pty(attach_req).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("AttachPty start: {e}");
                        return;
                    }
                };
                loop {
                    match stream.message().await {
                        Ok(Some(chunk)) => {
                            let bytes: Vec<u8> = chunk.data.to_vec();
                            // Send returns Err only when there are
                            // zero subscribers. Keep draining the
                            // stream so the sidecar doesn't
                            // backpressure on us.
                            let _ = attach_tx.send(bytes);
                        }
                        Ok(None) => break,
                        Err(e) => {
                            tracing::debug!("AttachPty stream: {e}");
                            break;
                        }
                    }
                }
            });
        }

        let backend = SidecarChildBackend::new(
            procs,
            process_id,
            tokio::runtime::Handle::current(),
        );
        let child = ChildHandle::new(actor_id.clone(), Box::new(backend), output_tx, log_path);

        Ok(ActorHandle {
            actor_id,
            kind: profile.kind,
            child: Arc::new(child),
            sandbox: None,
        })
    }

    fn name(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_label_includes_endpoint() {
        let rt = SidecarActorRuntime::new("http://127.0.0.1:2468");
        assert_eq!(rt.name(), "sidecar:http://127.0.0.1:2468");
    }

    #[test]
    fn shared_returns_trait_object() {
        let _rt: Arc<dyn ActorRuntime> = SidecarActorRuntime::shared("http://localhost:9999");
    }
}

