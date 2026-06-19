use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use crate::config::ShipConfig;

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

/// `ship login` — PKCE S256 OAuth flow, daemon-mediated over Connect.
///
/// Flow:
///   1. CLI binds an ephemeral callback port on 127.0.0.1
///   2. CLI calls `AccountService.Login` (daemon bearer); daemon generates
///      PKCE state, returns the browser URL
///   3. CLI opens browser
///   4. Browser → gateway OAuth → redirects to CLI's callback server with `code`
///   5. CLI calls `AccountService.LoginComplete{state_id, code}`; daemon does
///      the PKCE exchange, stores the API key, binds the daemon owner. The
///      daemon owns the token — publish/convert read it from the daemon, so
///      there's no legacy credentials.jsonc mirror (#79 retired).
pub fn run_login() -> Result<()> {
    use connectrpc::client::{ClientConfig, HttpClient};
    use ship_ops_rpc::{AccountServiceClient, LoginCompleteRequest, LoginRequest};
    let (daemon_port, daemon_token) = daemon_connect()?;

    // AccountService.Login{,Complete} (Connect) replace POST /api/v1/account/login
    // {,/complete}. Called with the daemon bearer — user-presence enforced
    // server-side (CallerKind::Cli).
    let uri: http::Uri =
        format!("http://127.0.0.1:{daemon_port}").parse().context("building daemon URI")?;
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {daemon_token}"));
    let account = AccountServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;

    // Bind callback first so we know the redirect_uri to send the daemon.
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| anyhow::anyhow!("Could not start callback server: {}", e))?;
    let cb_port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", cb_port);

    // 1. Start the login via daemon — daemon generates PKCE + returns
    //    the browser URL pointing at the gateway with our redirect_uri.
    let (state_id, browser_url) = {
        let resp = rt
            .block_on(account.login(LoginRequest { redirect_uri, ..Default::default() }))
            .map_err(|e| anyhow::anyhow!("account login failed: {e}"))?;
        let started = resp.view();
        (started.state_id.to_string(), started.browser_url.to_string())
    };

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
    //    owner. The daemon holds the token — publish/convert read it from the
    //    daemon now, so there's no Credentials mirror (the #79 bridge is gone).
    let user_id = {
        let resp = rt
            .block_on(account.login_complete(LoginCompleteRequest {
                state_id,
                code,
                ..Default::default()
            }))
            .map_err(|e| anyhow::anyhow!("account login complete failed: {e}"))?;
        resp.view().user_id.to_string()
    };

    println!("Logged in as {user_id}. Daemon stored token + bound to this account.");
    Ok(())
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


/// `ship logout` — clear the daemon's stored token.
///
/// The daemon owns the SINGLE token doc (`machine/daemon/account.jsonc`), so
/// logout is just the daemon's `AccountService.Logout` (revoke gateway key +
/// clear local token). There is no CLI-side `credentials.jsonc` mirror to wipe
/// (retired in S5).
///
/// Idempotent: when there is no local token AND the daemon is unreachable,
/// there is nothing to revoke — print "Not logged in." and exit 0. A live
/// daemon is only required when there IS a token to clear.
pub fn run_logout() -> Result<()> {
    let home = ship_substrate::ShipHome::from_env();
    if !home.account_token_path().exists() && daemon_connect().is_err() {
        println!("Not logged in.");
        return Ok(());
    }
    let (port, daemon_token) = daemon_connect()?;

    use connectrpc::client::{ClientConfig, HttpClient};
    use ship_ops_rpc::{AccountServiceClient, LogoutRequest};
    // AccountService.Logout (Connect) revokes the gateway key + clears the
    // daemon's stored token. `had_token=false` means there was nothing to clear.
    let uri = format!("http://127.0.0.1:{port}")
        .parse::<http::Uri>()
        .context("building daemon URI")?;
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {daemon_token}"));
    let client = AccountServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
    let result = rt
        .block_on(client.logout(LogoutRequest::default()))
        .map_err(|e| anyhow::anyhow!("logout failed: {e}"))?;

    if result.view().had_token {
        println!("Logged out.");
    } else {
        println!("Not logged in.");
    }
    Ok(())
}

/// Coarse account snapshot the CLI renders for `ship whoami`. Mirrors the typed
/// `AccountState` wire status; carries the `user_id` only when logged in.
enum AccountSnapshot {
    LoggedOut,
    Pending,
    LoggedIn { user_id: String },
}

/// Fetch account state over the typed Connect `AccountService` (account.proto).
/// Mirrors the `ship meta` pattern.
fn fetch_account_state(port: u16, token: &str) -> Result<AccountSnapshot> {
    use connectrpc::client::{ClientConfig, HttpClient};
    use ship_ops_rpc::account_state::Status;
    use ship_ops_rpc::{AccountServiceClient, StateRequest};
    let uri: http::Uri =
        format!("http://127.0.0.1:{port}").parse().context("building daemon URI")?;
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
    let client = AccountServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
    let resp = rt
        .block_on(client.state(StateRequest::default()))
        .map_err(|e| anyhow::anyhow!("account.state call failed: {e}"))?;
    let view = resp.view();
    Ok(match view.status.as_known() {
        Some(Status::LOGGED_IN) => AccountSnapshot::LoggedIn { user_id: view.user_id.to_string() },
        Some(Status::PENDING) => AccountSnapshot::Pending,
        _ => AccountSnapshot::LoggedOut,
    })
}

/// `ship whoami` — query the daemon for current account state.
pub fn run_whoami() -> Result<()> {
    match daemon_connect() {
        Ok((port, daemon_token)) => match fetch_account_state(port, &daemon_token) {
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

fn print_account_state(state: &AccountSnapshot) {
    match state {
        AccountSnapshot::LoggedIn { user_id } => {
            print_local_identity();
            println!("user_id: {user_id}");
            println!("authenticated");
        }
        AccountSnapshot::Pending => {
            print_local_identity();
            println!("login in progress — return to the browser tab and complete it");
        }
        AccountSnapshot::LoggedOut => {
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
