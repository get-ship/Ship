//! `ship meta` — calls the daemon's `ship.ops.v1` OpsService over ConnectRPC and
//! prints its version/build. The first CLI command on the Connect op surface;
//! the rest of the registry follows this same generated-client pattern.

use anyhow::{anyhow, Context, Result};
use ship_ops_rpc::{MetaRequest, OpsServiceClient};

use crate::uds_transport::{client_config, connect_transport_or_autostart};

pub fn run() -> Result<()> {
    let transport = connect_transport_or_autostart()
        .map_err(|e| anyhow!("shipd not running: {e}\nTry: ship daemon start"))?;
    let client = OpsServiceClient::new(transport, client_config());

    // Commands dispatch synchronously; there is no ambient runtime here.
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
    let resp = rt
        .block_on(client.meta(MetaRequest::default()))
        .map_err(|e| anyhow::anyhow!("ops.meta call failed: {e}"))?;
    let v = resp.view();
    println!("version {}  build {}", v.version, v.build);
    Ok(())
}
