//! Shared unix-socket transport for connectrpc CLI clients.
//!
//! The CLI is a peer-cred client: the host daemon (`shipd`) binds a unix
//! socket at `<daemon_dir>/ship.sock`, and the CLI connects to it without
//! a bearer token — the kernel-attested peer identity is the credential.
//! This module is the ONE place that knows how to bridge the connectrpc
//! generated clients (typed over `T: ClientTransport`) onto that socket.
//!
//! All CLI surfaces — `ship config` (SettingsService), `ship meta`
//! (OpsService), `ship use --dispatch` (AgentsService) — go through the
//! same [`connect_transport_or_autostart`] helper. The
//! [`ops_cmd`] "ops" surface uses the same socket with a hand-rolled
//! HTTP client (the ConnectRPC wire is plain Connect-JSON there).
//!
//! ## Construction
//!
//! `hyper_util::client::legacy::Client` is parameterized over a
//! `tower::Service<Uri>` connector. [`UdsConnector`] dials a fixed unix
//! path on every call (the `Uri` argument is required by the connector
//! trait but ignored). The client is then wrapped in
//! `connectrpc::client::ServiceTransport`, which adapts it to the
//! `ClientTransport` trait the generated clients consume.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use connectrpc::client::{ClientConfig, ServiceTransport};
use http::Uri;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::Connection as HyperConnection;
use hyper_util::client::legacy::connect::Connected;
use hyper_util::rt::TokioIo;
use tower::Service;

/// The request body type the connectrpc generated clients send.
pub type ConnectBody = BoxBody<Bytes, connectrpc::ConnectError>;

/// The hyper-util legacy client (HTTP/1.1) over a unix-socket connector.
pub type UdsClient = Client<UdsConnector, ConnectBody>;

/// The transport the connectrpc generated clients consume.
pub type UdsTransport = ServiceTransport<UdsClient>;

/// Resolve the host unix socket for the daemon serving `cwd`. Mirrors
/// `ops_cmd::discover_socket`: same discovery, same `None` semantics.
pub fn discover_socket() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let dir = ship_substrate::home::discover_running_daemon(&cwd)?.dir;
    let sock = dir.join("ship.sock");
    sock.exists().then_some(sock)
}

/// Build a `ClientConfig` pointed at the host socket with no bearer
/// (peer-cred is the credential on this transport). The `base_uri` is
/// cosmetic — the connector ignores the URI and always dials the same
/// socket path — but `ClientConfig::new` requires one. The placeholder
/// matches what `ship-isolation` uses for its unix connections.
pub fn client_config() -> ClientConfig {
    ClientConfig::new(client_uri())
}

/// Cosmetic `base_uri` for `ClientConfig`. See [`client_config`].
pub fn client_uri() -> Uri {
    "http://ship.local".parse().expect("valid URI")
}

/// Connect a connectrpc client to the host daemon's unix socket. Returns
/// `None` when no daemon is bound — the caller surfaces a "shipd not
/// running" error.
pub fn connect_transport() -> Option<UdsTransport> {
    let path = discover_socket()?;
    Some(ServiceTransport::new(build_client(path)))
}

/// Connect a connectrpc client to the host daemon's unix socket, or error
/// if no daemon is bound. The agent-facing CLI never autostarts the
/// daemon — that's an operator action (shipctl daemon start). Auto-start
/// was a CLI historical default the agent path drops: an agent
/// shouldn't quietly fork a daemon just to make a read.
pub fn connect_transport_or_autostart() -> anyhow::Result<UdsTransport> {
    connect_transport().ok_or_else(|| {
        anyhow::anyhow!("shipd not running (no host socket).\nTry: shipctl daemon start")
    })
}

/// Build the hyper-util legacy client for the given socket path. Exposed
/// for tests that wire a temp socket; production callers use
/// [`connect_transport`].
pub fn build_client(path: PathBuf) -> UdsClient {
    Client::builder(hyper_util::rt::TokioExecutor::new()).build(UdsConnector::new(path))
}

/// A `tower::Service<Uri>` connector that ignores the URI and dials a
/// fixed unix socket path on every call. The path is set at
/// construction; the `Uri` argument is required by hyper-util's
/// `Connect` blanket impl but unused.
#[derive(Clone)]
pub struct UdsConnector {
    path: Arc<PathBuf>,
}

impl UdsConnector {
    pub fn new(path: PathBuf) -> Self {
        Self { path: Arc::new(path) }
    }
}

impl Service<Uri> for UdsConnector {
    type Response = UdsStream;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _uri: Uri) -> Self::Future {
        let path = self.path.clone();
        Box::pin(async move {
            let stream = tokio::net::UnixStream::connect(path.as_os_str()).await?;
            Ok(UdsStream(TokioIo::new(stream)))
        })
    }
}

/// Local newtype around `TokioIo<UnixStream>` so we can implement
/// `hyper_util::client::legacy::connect::Connection` (the orphan rule
/// forbids implementing foreign traits for foreign types).
pub struct UdsStream(pub TokioIo<tokio::net::UnixStream>);

impl hyper::rt::Read for UdsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl hyper::rt::Write for UdsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl HyperConnection for UdsStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}
