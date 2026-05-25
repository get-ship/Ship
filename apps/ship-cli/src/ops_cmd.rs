//! `ship ops` — list and invoke the daemon op surface over ConnectRPC.
//!
//! This is the agent interface: a capability-skill shells out to
//! `ship ops call <name> --json '{…}'`, and the CLI resolves the op to its
//! Connect path (`/ship.ops.v1.<Service>/<Method>`) and POSTs Connect-JSON.
//! The catalog comes from the daemon (`/api/v1/ship/schema` = the same
//! `schema::OPS` + `COMMANDS` the surface is generated from), so the CLI never
//! re-expresses the registry — it projects it. Replaces the internal MCP tool
//! surface as the way agents call ops.
//!
//! ## Transport
//!
//! Two ways to reach the daemon, one indirection (`transport()`):
//! - **Host unix socket** (`<daemon_dir>/ship.sock`) — preferred when present.
//!   The agent path: identity comes from the connection's kernel peer
//!   credentials (`SO_PEERCRED`), so **no token is presented**. An agent only
//!   ever has this; it cannot mint or read the operator token.
//! - **TCP + operator token** — fallback for the operator's own shell and for
//!   platforms without the socket. Goes through `ensure_daemon()`.
//!
//! `ship ops` is the seam that absorbs the transport choice; the sdk-skills
//! teach only `ship ops call …` and never mention sockets, ports, or tokens.

use anyhow::{Context, Result};
use serde_json::Value;

/// Resolve a dotted op name → its proto `(service, method)` via the embedded
/// FileDescriptorSet — the SSOT, the SAME source gen-skills documents. Routes to
/// the per-domain service (`RulesService/Create`), not the retiring generated
/// umbrella; so every documented op (incl. proto-only ones like `analytics.*`
/// the Rust registry never carried) is callable. Streaming + umbrella excluded.
fn resolve(name: &str) -> Option<(String, String)> {
    ship_ops_rpc::policy::read_policies()
        .into_iter()
        .find(|p| {
            !matches!(
                p.service.as_str(),
                "ReadOpsService" | "CommandOpsService" | "StreamService"
            ) && ship_ops_rpc::policy::op_name(&p.service, &p.method) == name
        })
        .map(|p| (p.service, p.method))
}

/// How to reach the daemon's op surface.
enum OpTransport {
    /// Host unix socket — agent path, identity via peer-cred, no token.
    #[cfg(unix)]
    Unix(std::path::PathBuf),
    /// TCP + operator token — operator path / platforms without the socket.
    Tcp { port: u16, token: String },
}

/// Choose the transport: the host socket if it exists (agent, tokenless),
/// otherwise TCP+token via `ensure_daemon()` (operator).
fn transport() -> Result<OpTransport> {
    #[cfg(unix)]
    if let Some(sock) = discover_socket() {
        return Ok(OpTransport::Unix(sock));
    }
    let (port, token) = crate::daemon::ensure_daemon()?;
    Ok(OpTransport::Tcp { port, token })
}

/// The host socket for the daemon serving this cwd, if it's bound. Never starts
/// a daemon — the socket exists only when one is already running.
#[cfg(unix)]
fn discover_socket() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let dir = ship_substrate::home::discover_running_daemon(&cwd)?.dir;
    let sock = dir.join("ship.sock");
    sock.exists().then_some(sock)
}

/// POST Connect-JSON to `path`, returning `(status, body)`.
fn op_post(path: &str, body: &str) -> Result<(u16, String)> {
    match transport()? {
        #[cfg(unix)]
        OpTransport::Unix(sock) => {
            let (s, b) = unix_http(&sock, "POST", path, Some(body))?;
            Ok((s, String::from_utf8_lossy(&b).into_owned()))
        }
        OpTransport::Tcp { port, token } => {
            let url = format!("http://127.0.0.1:{port}{path}");
            let resp = reqwest::blocking::Client::new()
                .post(&url)
                .bearer_auth(token)
                .header("content-type", "application/json")
                .header("connect-protocol-version", "1")
                .body(body.to_string())
                .send()
                .context("calling op")?;
            let status = resp.status().as_u16();
            Ok((status, resp.text().unwrap_or_default()))
        }
    }
}

/// GET `path`, returning `(status, body)`.
fn op_get(path: &str) -> Result<(u16, String)> {
    match transport()? {
        #[cfg(unix)]
        OpTransport::Unix(sock) => {
            let (s, b) = unix_http(&sock, "GET", path, None)?;
            Ok((s, String::from_utf8_lossy(&b).into_owned()))
        }
        OpTransport::Tcp { port, token } => {
            let url = format!("http://127.0.0.1:{port}{path}");
            let resp = reqwest::blocking::Client::new()
                .get(&url)
                .bearer_auth(token)
                .send()
                .context("fetching op schema")?;
            let status = resp.status().as_u16();
            Ok((status, resp.text().unwrap_or_default()))
        }
    }
}

/// Minimal blocking HTTP/1.1 over a unix socket. We control both ends (axum)
/// and the bodies are small Connect-JSON, so `Connection: close` + read-to-EOF
/// is sufficient; chunked is de-chunked defensively. No token — the daemon
/// authenticates this connection from its kernel peer credentials.
#[cfg(unix)]
fn unix_http(
    sock: &std::path::Path,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<(u16, Vec<u8>)> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(sock)
        .with_context(|| format!("connecting host socket {}", sock.display()))?;
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: ship\r\nConnection: close\r\n");
    if let Some(b) = body {
        req.push_str("content-type: application/json\r\n");
        req.push_str("connect-protocol-version: 1\r\n");
        req.push_str(&format!("content-length: {}\r\n", b.len()));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes())?;
    if let Some(b) = body {
        stream.write_all(b.as_bytes())?;
    }
    stream.flush()?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    parse_http_response(&raw)
}

/// Split a raw HTTP/1.1 response into `(status, body)`. Byte-based (no str
/// slicing on non-char boundaries). Handles Content-Length (read-to-EOF) and
/// `Transfer-Encoding: chunked`.
#[cfg(unix)]
fn parse_http_response(raw: &[u8]) -> Result<(u16, Vec<u8>)> {
    let pos = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .context("malformed HTTP response (no header terminator)")?;
    let head = String::from_utf8_lossy(&raw[..pos]);
    let body = &raw[pos + 4..];
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .context("malformed HTTP status line")?;
    let chunked = head.lines().any(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("transfer-encoding:") && l.contains("chunked")
    });
    let body = if chunked { dechunk(body) } else { body.to_vec() };
    Ok((status, body))
}

/// De-chunk an HTTP/1.1 chunked body. Stops at the terminating `0` chunk or any
/// malformed size line. Tolerant by design — the server is our own axum.
#[cfg(unix)]
fn dechunk(mut body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    while let Some(nl) = body.windows(2).position(|w| w == b"\r\n") {
        let size = std::str::from_utf8(&body[..nl])
            .ok()
            .and_then(|s| usize::from_str_radix(s.trim(), 16).ok())
            .unwrap_or(0);
        if size == 0 {
            break;
        }
        let start = nl + 2;
        let end = (start + size).min(body.len());
        out.extend_from_slice(&body[start..end]);
        body = body.get(end + 2..).unwrap_or(&[]);
    }
    out
}

/// Fetch the op catalog (reads + commands) from the daemon.
fn catalog() -> Result<(Vec<Value>, Vec<Value>)> {
    let (status, text) = op_get("/api/v1/ship/schema")?;
    if !(200..300).contains(&status) {
        anyhow::bail!("fetching op schema failed ({status}): {text}");
    }
    let resp: Value = serde_json::from_str(&text).context("parsing op schema")?;
    let ops = resp["data"]["ops"].as_array().cloned().unwrap_or_default();
    let cmds = resp["data"]["commands"].as_array().cloned().unwrap_or_default();
    Ok((ops, cmds))
}

/// `ship ops` / `ship ops list` — the scoped catalog (#70).
pub fn list() -> Result<()> {
    let (ops, cmds) = catalog()?;
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
    let (service, method) =
        resolve(name).with_context(|| format!("unknown op `{name}` — run `ship ops list`"))?;
    let body: Value = match json_args {
        Some(s) => serde_json::from_str(&s).context("parsing --json args")?,
        None => serde_json::json!({}),
    };
    let path = format!("/ship.ops.v1.{service}/{method}");
    let (status, text) = op_post(&path, &body.to_string())?;
    if !(200..300).contains(&status) {
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn content_length_body_parses() {
        let raw = b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 13\r\n\r\n{\"json\":\"ok\"}";
        let (status, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"{\"json\":\"ok\"}");
    }

    #[test]
    fn non_2xx_status_is_surfaced() {
        let raw = b"HTTP/1.1 403 Forbidden\r\ncontent-length: 6\r\n\r\ndenied";
        let (status, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 403);
        assert_eq!(body, b"denied");
    }

    #[test]
    fn chunked_body_is_dechunked() {
        // Two chunks ("Wiki" + "pedia") then the 0 terminator.
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        let (status, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"Wikipedia");
    }

    #[test]
    fn missing_terminator_errors() {
        assert!(parse_http_response(b"HTTP/1.1 200 OK\r\nno-end").is_err());
    }
}
