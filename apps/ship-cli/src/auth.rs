use anyhow::Result;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use crate::config::{Credentials, CredentialsAccount, DiskBackedConfig, ShipConfig};

// PKCE state (verifier + challenge generation) moved to the daemon's
// AccountService in `crates/core/ship-platform/src/account/service.rs`.
// The CLI is the orchestrator + browser launcher; the daemon owns the
// crypto and the resulting token.

/// Open `url` in the system's default browser.
fn open_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn();
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    eprintln!("Cannot open browser automatically. Visit: {}", url);
}

/// Extract the `code` query parameter from an HTTP GET request line.
fn parse_code_from_request(request: &str) -> Option<String> {
    let line = request.lines().next()?;
    let path = line.split_whitespace().nth(1)?;
    let query = path.split_once('?')?.1;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next() == Some("code") {
            return kv.next().map(str::to_string);
        }
    }
    None
}

/// `ship login` — PKCE S256 OAuth flow, daemon-mediated.
///
/// Flow:
///   1. CLI binds an ephemeral callback port on 127.0.0.1
///   2. CLI POSTs the redirect_uri to daemon `/api/v1/account/login`;
///      daemon generates PKCE state, returns the browser URL
///   3. CLI opens browser
///   4. Browser → gateway OAuth → redirects to CLI's callback server with `code`
///   5. CLI POSTs `{state_id, code}` to daemon `/api/v1/account/login/complete`;
///      daemon does PKCE exchange + stores API key + binds daemon owner
///   6. CLI fetches the key via `/api/v1/account/key` and dual-writes to
///      legacy credentials.jsonc so publish/convert continue to work
///      (transitional bridge; remove when task #79 lands)
pub fn run_login() -> Result<()> {
    let (daemon_port, daemon_token) = daemon_connect()?;

    // Bind callback first so we know the redirect_uri to send the daemon.
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| anyhow::anyhow!("Could not start callback server: {}", e))?;
    let cb_port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", cb_port);

    // 1. Start the login via daemon — daemon generates PKCE + returns
    //    the browser URL pointing at the gateway with our redirect_uri.
    let start: serde_json::Value = daemon_post_json(
        daemon_port,
        &daemon_token,
        "/api/v1/account/login",
        serde_json::json!({ "redirect_uri": redirect_uri }),
    )?;
    let state_id = start.get("state_id").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("daemon /account/login: missing state_id"))?
        .to_string();
    let browser_url = start.get("browser_url").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("daemon /account/login: missing browser_url"))?
        .to_string();

    println!("Opening browser for login...");
    println!("  {}", browser_url);
    open_browser(&browser_url);
    println!("\nWaiting for callback (60 s timeout)...");

    // 2. Receive the auth code from the OAuth callback.
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let code = parse_code_from_request(&request).unwrap_or_default();
            let body =
                "<html><body><h1>Login complete</h1><p>You can close this tab.</p></body></html>";
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = tx.send(code);
        }
    });

    let code = match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => anyhow::bail!("No authorization code received. Login aborted."),
        Err(_) => anyhow::bail!("Login timed out (60 s). Run `ship login` to try again."),
    };

    // 3. Daemon exchanges the code, stores the API key, binds the daemon
    //    owner. Single round-trip; no Credentials write yet.
    let complete: serde_json::Value = daemon_post_json(
        daemon_port,
        &daemon_token,
        "/api/v1/account/login/complete",
        serde_json::json!({ "state_id": state_id, "code": code }),
    )?;
    let user_id = complete.get("user_id").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("daemon /account/login/complete: missing user_id"))?
        .to_string();

    // 4. Transitional bridge — fetch the API key back from the daemon
    //    and write it into legacy credentials.jsonc so publish/convert
    //    keep working until they too go via the daemon (task #79).
    let key: serde_json::Value = daemon_get_json(
        daemon_port,
        &daemon_token,
        "/api/v1/account/key",
    )?;
    let api_key = key.get("key").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("daemon /account/key: missing key"))?
        .to_string();
    let mut creds = Credentials::load();
    creds.account = Some(CredentialsAccount { token: Some(api_key) });
    creds.save()?;

    println!("Logged in as {user_id}. Daemon stored token + bound to this account.");
    println!("(transitional: credentials also mirrored to {} for publish/convert)", Credentials::path().display());
    Ok(())
}

/// POST a JSON body to the daemon, parse the JSON response.
fn daemon_post_json(
    port: u16,
    daemon_token: &str,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = crate::http::daemon_client()
        .post(&url)
        .bearer_auth(daemon_token)
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon {path} POST failed: {e}"))?;
    crate::http::check_skew(resp.headers());
    let status = resp.status();
    let body_text = resp.text().unwrap_or_default();
    if !status.is_success() {
        let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
        let err_msg = parsed.get("error").and_then(|v| v.as_str()).unwrap_or(&body_text);
        anyhow::bail!("daemon {path} returned {status}: {err_msg}");
    }
    serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("daemon {path}: parse response: {e}"))
}

/// GET a JSON response from the daemon.
fn daemon_get_json(port: u16, daemon_token: &str, path: &str) -> Result<serde_json::Value> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = crate::http::daemon_client()
        .get(&url)
        .bearer_auth(daemon_token)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon {path} GET failed: {e}"))?;
    crate::http::check_skew(resp.headers());
    let status = resp.status();
    let body_text = resp.text().unwrap_or_default();
    if !status.is_success() {
        let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
        let err_msg = parsed.get("error").and_then(|v| v.as_str()).unwrap_or(&body_text);
        anyhow::bail!("daemon {path} returned {status}: {err_msg}");
    }
    serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("daemon {path}: parse response: {e}"))
}

/// `ship auth pair <code>` — confirm a pairing code from Studio.
///
/// Reads the daemon token from disk (proving local filesystem access),
/// POSTs to `/api/auth/pair/confirm`, and prints success or the daemon's error.
pub fn run_auth_pair(code: &str) -> Result<()> {
    let code = code.trim().to_uppercase();
    if code.is_empty() {
        anyhow::bail!("empty pairing code");
    }
    let (port, token) = daemon_connect()?;
    let url = format!("http://127.0.0.1:{port}/api/auth/pair/confirm");
    let resp = crate::http::daemon_client()
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::json!({ "code": code }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon request failed: {e}"))?;

    crate::http::check_skew(resp.headers());
    let status = resp.status();
    let body_text = resp.text().unwrap_or_default();

    if status.is_success() {
        println!("Paired. Return to Studio — it should connect automatically.");
        return Ok(());
    }

    let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
    let err_msg = parsed
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or(&body_text);
    anyhow::bail!("pairing failed ({status}): {err_msg}");
}

/// Read daemon connection info (port + daemon.token) from ~/.ship/.
fn daemon_connect() -> Result<(u16, String)> {
    let home = ship_substrate::ShipHome::from_env();
    let port: u16 = std::fs::read_to_string(home.daemon_file(ship_substrate::home::DaemonFile::Port))
        .map_err(|_| anyhow::anyhow!("daemon is not running — start it with: ship daemon start"))?
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid daemon port file"))?;
    let token = std::fs::read_to_string(home.daemon_file(ship_substrate::home::DaemonFile::Token))
        .map_err(|_| anyhow::anyhow!("daemon token not readable — is the daemon running?"))?
        .trim()
        .to_string();
    Ok((port, token))
}

/// `ship auth status` — list active paired sessions.
pub fn run_auth_status() -> Result<()> {
    let (port, token) = daemon_connect()?;
    let url = format!("http://127.0.0.1:{port}/api/auth/sessions");
    let resp = crate::http::daemon_client()
        .get(&url)
        .bearer_auth(&token)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon request failed: {e}"))?;
    crate::http::check_skew(resp.headers());
    let status = resp.status();
    let body: serde_json::Value = resp.json()
        .map_err(|e| anyhow::anyhow!("invalid daemon response: {e}"))?;
    if !status.is_success() {
        anyhow::bail!("failed ({status}): {body}");
    }

    let sessions = body.get("sessions").and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("missing sessions field"))?;
    if sessions.is_empty() {
        println!("No active paired sessions.");
        return Ok(());
    }

    let now = chrono::Utc::now().timestamp();
    println!("{:<18}  {:<20}  {:<10}  EXPIRES", "TOKEN", "AGENT", "KIND");
    for s in sessions {
        let prefix = s.get("token_prefix").and_then(|v| v.as_str()).unwrap_or("");
        let agent = s.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
        let ephemeral = s.get("ephemeral").and_then(|v| v.as_bool()).unwrap_or(false);
        let kind = if ephemeral { "ephemeral" } else { "persisted" };
        let expires = match s.get("expires_at_unix").and_then(|v| v.as_i64()) {
            Some(ts) => {
                let remaining = ts - now;
                if remaining <= 0 {
                    "expired".to_string()
                } else {
                    let days = remaining / 86400;
                    let hours = (remaining % 86400) / 3600;
                    if days > 0 { format!("in {days}d {hours}h") } else { format!("in {hours}h") }
                }
            }
            None => "never".to_string(),
        };
        println!("{prefix:<18}  {agent:<20}  {kind:<10}  {expires}");
    }
    Ok(())
}

/// `ship auth revoke --token <prefix>` or `ship auth revoke --all` — revoke sessions.
pub fn run_auth_revoke(token: Option<&str>, all: bool) -> Result<()> {
    if token.is_none() && !all {
        anyhow::bail!("pass --token <prefix> or --all");
    }
    let (port, daemon_token) = daemon_connect()?;
    let url = format!("http://127.0.0.1:{port}/api/auth/sessions/revoke");
    let body = serde_json::json!({
        "token_prefix": token,
        "all": all,
    });
    let resp = crate::http::daemon_client()
        .post(&url)
        .bearer_auth(&daemon_token)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon request failed: {e}"))?;
    crate::http::check_skew(resp.headers());
    let status = resp.status();
    let body_text = resp.text().unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("revoke failed ({status}): {body_text}");
    }
    let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
    let revoked = parsed.get("revoked").and_then(|v| v.as_u64()).unwrap_or(0);
    println!("Revoked {revoked} session(s).");
    Ok(())
}

/// `ship logout` — clear the daemon's stored token + the legacy CLI mirror.
pub fn run_logout() -> Result<()> {
    // Best-effort daemon clear — if daemon isn't running we still want
    // to nuke the local mirror so the user isn't stuck in a half-state.
    let mut daemon_cleared = false;
    if let Ok((port, daemon_token)) = daemon_connect() {
        let url = format!("http://127.0.0.1:{port}/api/v1/account/logout");
        if let Ok(resp) = crate::http::daemon_client()
            .post(&url)
            .bearer_auth(&daemon_token)
            .timeout(std::time::Duration::from_secs(5))
            .send()
        {
            crate::http::check_skew(resp.headers());
            daemon_cleared = resp.status().is_success();
        }
    }

    let mut creds = Credentials::load();
    let had_legacy = creds.account_token().is_some();
    if had_legacy {
        creds.account = None;
        creds.save()?;
    }

    if !daemon_cleared && !had_legacy {
        println!("Not logged in.");
        return Ok(());
    }
    println!("Logged out.");
    Ok(())
}

/// `ship whoami` — query the daemon for current account state.
pub fn run_whoami() -> Result<()> {
    match daemon_connect() {
        Ok((port, daemon_token)) => match daemon_get_json(port, &daemon_token, "/api/v1/account/state") {
            Ok(state) => print_account_state(&state),
            Err(_) => {
                // Daemon unreachable — show local identity at least.
                print_local_identity();
                println!("(daemon unreachable — could not fetch account state)");
            }
        },
        Err(_) => {
            print_local_identity();
            println!("Not logged in. (daemon not running; run `ship daemon start` to enable account ops)");
        }
    }
    Ok(())
}

fn print_account_state(state: &serde_json::Value) {
    let kind = state.get("state").and_then(|v| v.as_str()).unwrap_or("logged_out");
    match kind {
        "logged_in" => {
            print_local_identity();
            if let Some(uid) = state.get("user_id").and_then(|v| v.as_str()) {
                println!("user_id: {uid}");
            }
            println!("authenticated");
        }
        "pending" => {
            print_local_identity();
            println!("login in progress — return to the browser tab and complete it");
        }
        _ => {
            print_local_identity();
            println!("Not logged in.");
        }
    }
}

fn print_local_identity() {
    let cfg = ShipConfig::load();
    match cfg.library.identity.as_ref() {
        Some(id) if id.name.as_deref().map(|n| !n.is_empty()).unwrap_or(false) => {
            if let Some(name) = id.name.as_deref() {
                println!("{name}");
            }
            if let Some(email) = id.email.as_deref() {
                println!("{email}");
            }
        }
        _ => println!("(no identity — run: ship init --global)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code_extracts_code_param() {
        let req = "GET /callback?code=abc123&state=xyz HTTP/1.1\r\nHost: localhost\r\n";
        assert_eq!(parse_code_from_request(req), Some("abc123".to_string()));
    }

    #[test]
    fn parse_code_returns_none_when_absent() {
        let req = "GET /callback?state=xyz HTTP/1.1\r\n";
        assert_eq!(parse_code_from_request(req), None);
    }

    // PKCE verifier/challenge tests moved with the implementation —
    // they live alongside AccountService in ship-platform now.
}
