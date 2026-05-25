//! `ship ops` — list and invoke the daemon op surface over ConnectRPC.
//!
//! This is the agent interface: a capability-skill shells out to
//! `ship ops call <name> --json '{…}'`, and the CLI resolves the op to its
//! Connect path (`/ship.ops.v1.<Service>/<Method>`) and POSTs Connect-JSON.
//! The catalog comes from the daemon (`/api/v1/ship/schema` = the same
//! `schema::OPS` + `COMMANDS` the surface is generated from), so the CLI never
//! re-expresses the registry — it projects it. Replaces the internal MCP tool
//! surface as the way agents call ops.

use anyhow::{Context, Result};
use serde_json::Value;

/// PascalCase a dotted/underscored op name → the generated Connect method
/// (`pipeline.update_status` → `PipelineUpdateStatus`). Mirrors `xtask::pascal`.
fn pascal(op: &str) -> String {
    op.split(['.', '_'])
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

/// Fetch the op catalog (reads + commands) from the daemon.
fn catalog(port: u16, token: &str) -> Result<(Vec<Value>, Vec<Value>)> {
    let url = format!("http://127.0.0.1:{port}/api/v1/ship/schema");
    let resp: Value = reqwest::blocking::Client::new()
        .get(&url)
        .bearer_auth(token)
        .send()
        .context("fetching op schema")?
        .json()
        .context("parsing op schema")?;
    let ops = resp["data"]["ops"].as_array().cloned().unwrap_or_default();
    let cmds = resp["data"]["commands"].as_array().cloned().unwrap_or_default();
    Ok((ops, cmds))
}

/// `ship ops` / `ship ops list` — the scoped catalog (#70).
pub fn list() -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let (ops, cmds) = catalog(port, &token)?;
    println!("reads ({}):", ops.len());
    for o in &ops {
        println!(
            "  {:<30} {}",
            o["name"].as_str().unwrap_or(""),
            o["description"].as_str().unwrap_or("")
        );
    }
    println!("\ncommands ({}):", cmds.len());
    for c in &cmds {
        println!(
            "  {:<30} [{}] {}",
            c["name"].as_str().unwrap_or(""),
            c["blast_radius"].as_str().unwrap_or(""),
            c["description"].as_str().unwrap_or("")
        );
    }
    Ok(())
}

/// `ship ops call <name> [--json '{…}']` — generic Connect invoke.
pub fn call(name: &str, json_args: Option<String>) -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let (ops, cmds) = catalog(port, &token)?;
    let service = if ops.iter().any(|o| o["name"].as_str() == Some(name)) {
        "ReadOpsService"
    } else if cmds.iter().any(|c| c["name"].as_str() == Some(name)) {
        "CommandOpsService"
    } else {
        anyhow::bail!("unknown op `{name}` — run `ship ops list`");
    };
    let body: Value = match json_args {
        Some(s) => serde_json::from_str(&s).context("parsing --json args")?,
        None => serde_json::json!({}),
    };
    let url = format!("http://127.0.0.1:{port}/ship.ops.v1.{service}/{}", pascal(name));
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .bearer_auth(token)
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1")
        .body(body.to_string())
        .send()
        .context("calling op")?;
    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("op `{name}` failed ({status}): {text}");
    }
    // Reads + commands return JsonResponse { json: "<inner>" }; unwrap for readability.
    if let Ok(v) = serde_json::from_str::<Value>(&text) {
        if let Some(inner) = v.get("json").and_then(|j| j.as_str()) {
            println!("{inner}");
        } else {
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or(text));
        }
    } else {
        println!("{text}");
    }
    Ok(())
}
