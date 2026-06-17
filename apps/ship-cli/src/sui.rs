//! `ship sui <diagnostics|hover|dependencies> <file>` — SUI `.sui` language
//! intelligence over `UiService` (Connect). The unified authoring service
//! surfaces the compiler analysis ops; replaces the MCP `sui_*` tools and
//! the retired `SuiService` Connect service. The daemon answers in-process
//! via the sui_compiler facade.

use anyhow::{Context, Result, anyhow};
use connectrpc::client::{ClientConfig, HttpClient};
use ship_ops_rpc::{SuiFileReq, SuiHoverReq, UiServiceClient};

use crate::cli::SuiCommands;

pub fn run(action: SuiCommands) -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let uri: http::Uri =
        format!("http://127.0.0.1:{port}").parse().context("building daemon URI")?;
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
    let client = UiServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;

    let json = match action {
        SuiCommands::Diagnostics { file } => {
            let resp = rt
                .block_on(client.diagnostics(SuiFileReq { file, ..Default::default() }))
                .map_err(|e| anyhow!("sui diagnostics failed: {e}"))?;
            resp.view().json.to_string()
        }
        SuiCommands::Hover { file, line, col } => {
            let resp = rt
                .block_on(client.hover(SuiHoverReq { file, line, col, ..Default::default() }))
                .map_err(|e| anyhow!("sui hover failed: {e}"))?;
            resp.view().json.to_string()
        }
        SuiCommands::Dependencies { file } => {
            let resp = rt
                .block_on(client.dependencies(SuiFileReq { file, ..Default::default() }))
                .map_err(|e| anyhow!("sui dependencies failed: {e}"))?;
            resp.view().json.to_string()
        }
    };

    // The daemon returns compact JSON; pretty-print for human + agent reading.
    match serde_json::from_str::<serde_json::Value>(&json) {
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or(json)),
        Err(_) => println!("{json}"),
    }
    Ok(())
}
