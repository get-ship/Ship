//! `ship meta` — calls the daemon's `ship.ops.v1` OpsService over ConnectRPC and
//! prints its version/build. The first CLI command on the Connect op surface;
//! the rest of the registry follows this same generated-client pattern.

use anyhow::{Context, Result};
use connectrpc::client::{ClientConfig, HttpClient};
use ship_ops_rpc::{MetaRequest, OpsServiceClient};

pub fn run() -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let uri: http::Uri = format!("http://127.0.0.1:{port}")
        .parse()
        .context("building daemon URI")?;
    // The op surface is mounted behind auth_middleware; present the daemon token.
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
    let client = OpsServiceClient::new(HttpClient::plaintext(), config);

    // Commands dispatch synchronously; there is no ambient runtime here.
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
    let resp = rt
        .block_on(client.meta(MetaRequest::default()))
        .map_err(|e| anyhow::anyhow!("ops.meta call failed: {e}"))?;
    let v = resp.view();
    println!("version {}  build {}", v.version, v.build);
    Ok(())
}
