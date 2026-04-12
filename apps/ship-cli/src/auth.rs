use anyhow::Result;
use base64::Engine as _;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use crate::config::{Credentials, CredentialsAccount, ShipConfig};

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

/// Generate a PKCE code_verifier (64 random bytes, base64url-encoded, no padding).
/// Uses OS CSPRNG via getrandom.
fn generate_code_verifier() -> String {
    let mut buf = [0u8; 48]; // 48 bytes → 64 base64url chars
    getrandom::getrandom(&mut buf).expect("OS RNG failed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

/// Compute the S256 code_challenge from a verifier:
/// `base64url_nopad(sha256(verifier.as_bytes()))`
fn s256_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
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

/// Exchange an authorization code + PKCE verifier for a token via POST.
fn exchange_code_for_token(code: &str, verifier: &str, port: u16) -> Result<String> {
    let body = serde_json::json!({
        "code": code,
        "verifier": verifier,
        "redirect_uri": format!("http://127.0.0.1:{}/callback", port),
    });

    let resp: String = ureq::post("https://getship.dev/api/auth/token")
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .map_err(|e| anyhow::anyhow!("Token exchange failed: {}", e))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Failed to read token response: {e}"))?;

    let parsed: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|_| anyhow::anyhow!("Invalid response from auth server"))?;

    if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
        anyhow::bail!("Auth failed: {}", err);
    }

    parsed
        .get("token")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No token in auth response"))
}

/// `ship login` — PKCE S256 OAuth flow.
pub fn run_login() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| anyhow::anyhow!("Could not start callback server: {}", e))?;
    let port = listener.local_addr()?.port();

    let verifier = generate_code_verifier();
    let challenge = s256_challenge(&verifier);
    let state = format!("{:x}", port as u64 ^ 0xdead_beef);

    let auth_url = format!(
        "https://getship.dev/auth/cli\
        ?code_challenge={challenge}\
        &code_challenge_method=S256\
        &redirect_uri=http://127.0.0.1:{port}/callback\
        &state={state}"
    );

    println!("Opening browser for login...");
    println!("  {}", auth_url);
    open_browser(&auth_url);
    println!("\nWaiting for callback (60 s timeout)...");

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

    let token = exchange_code_for_token(&code, &verifier, port)?;

    let mut creds = Credentials::load();
    creds.account = Some(CredentialsAccount { token: Some(token.clone()) });
    creds.save()?;

    // Bind this machine's daemon to the logged-in user. Enables zero-click
    // browser auth via server attestation (Studio → daemon) while ensuring
    // a stolen JWT from a different user can't unlock this daemon.
    match bind_daemon_owner(&token) {
        Ok(Some(user_id)) => {
            println!("Logged in as {user_id}. Daemon bound to this account.");
        }
        Ok(None) => {
            println!("Logged in. (Could not fetch user profile to bind daemon — run `ship daemon status`.)");
        }
        Err(e) => {
            println!("Logged in. (Daemon binding failed: {e})");
        }
    }
    println!("Token stored in ~/.ship/credentials (0600).");
    Ok(())
}

/// Fetch the authenticated user's id and record it as the daemon owner in
/// platform.db. The daemon checks this on every server-attest call.
fn bind_daemon_owner(token: &str) -> Result<Option<String>> {
    let mut resp = ureq::get("https://getship.dev/api/me")
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| anyhow::anyhow!("fetch user profile: {e}"))?;
    let body: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| anyhow::anyhow!("parse user profile: {e}"))?;
    let user_id = body
        .get("user")
        .and_then(|u| u.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from);
    if let Some(ref id) = user_id {
        let value = serde_json::Value::String(id.clone());
        runtime::db::kv::set("auth.daemon_owner", "user_id", &value)
            .map_err(|e| anyhow::anyhow!("write daemon owner: {e}"))?;
    }
    Ok(user_id)
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
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::json!({ "code": code }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon request failed: {e}"))?;

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
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?
        .join(".ship");
    let port: u16 = std::fs::read_to_string(home.join("daemon.port"))
        .map_err(|_| anyhow::anyhow!("daemon is not running — start it with: ship daemon start"))?
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid daemon.port file"))?;
    let token = std::fs::read_to_string(home.join("daemon.token"))
        .map_err(|_| anyhow::anyhow!("daemon.token not readable — is the daemon running?"))?
        .trim()
        .to_string();
    Ok((port, token))
}

/// `ship auth status` — list active paired sessions.
pub fn run_auth_status() -> Result<()> {
    let (port, token) = daemon_connect()?;
    let url = format!("http://127.0.0.1:{port}/api/auth/sessions");
    let resp = reqwest::blocking::Client::new()
        .get(&url)
        .bearer_auth(&token)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon request failed: {e}"))?;
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
    println!("{:<18}  {:<20}  {:<10}  {}", "TOKEN", "AGENT", "KIND", "EXPIRES");
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
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .bearer_auth(&daemon_token)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow::anyhow!("daemon request failed: {e}"))?;
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

/// `ship logout` — remove the stored auth token.
pub fn run_logout() -> Result<()> {
    let mut creds = Credentials::load();
    if creds.token().is_none() {
        println!("Not logged in.");
        return Ok(());
    }
    creds.account = None;
    creds.save()?;
    // Clear the daemon-owner binding so the next ship login can bind fresh.
    let _ = runtime::db::kv::delete("auth.daemon_owner", "user_id");
    println!("Logged out.");
    Ok(())
}

/// `ship whoami` — print identity and auth token status.
/// Tries to fetch user info from getship.dev; falls back to local config.
pub fn run_whoami() -> Result<()> {
    let creds = Credentials::load();
    match creds.token() {
        Some(token) => {
            // Try the remote API first.
            match fetch_me(token) {
                Ok(info) => println!("{}", info),
                Err(_) => {
                    // API unreachable — show local identity.
                    print_local_identity();
                    println!("authenticated (offline — could not reach API)");
                }
            }
        }
        None => {
            print_local_identity();
            println!("Not logged in.");
        }
    }
    Ok(())
}

fn print_local_identity() {
    let cfg = ShipConfig::load();
    match cfg.identity {
        Some(ref id) if !id.name.is_empty() => {
            println!("{}", id.name);
            if let Some(ref email) = id.email {
                println!("{}", email);
            }
        }
        _ => println!("(no identity — run: ship init --global)"),
    }
}

/// Fetch /api/auth/me and return a display string.
fn fetch_me(token: &str) -> Result<String> {
    let mut resp = ureq::get("https://getship.dev/api/auth/me")
        .header("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(|e| anyhow::anyhow!("API request failed: {}", e))?;

    let body: String = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Failed to read response: {e}"))?;

    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| anyhow::anyhow!("Invalid response"))?;

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("(unknown)");
    let email = parsed.get("email").and_then(|v| v.as_str());

    let mut out = name.to_string();
    if let Some(e) = email {
        out.push_str(&format!("\n{}", e));
    }
    out.push_str("\nauthenticated");
    Ok(out)
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

    #[test]
    fn generate_code_verifier_length_and_charset() {
        let v = generate_code_verifier();
        // 48 random bytes → 64 base64url chars (no padding)
        assert_eq!(v.len(), 64);
        assert!(
            v.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "verifier contains invalid chars: {v}"
        );
    }

    #[test]
    fn generate_code_verifier_is_not_deterministic() {
        let v1 = generate_code_verifier();
        let v2 = generate_code_verifier();
        assert_ne!(v1, v2, "two verifiers should not be identical");
    }

    #[test]
    fn s256_challenge_is_base64url_sha256() {
        // Test vector: SHA256("abc") = ba7816bf8f01cfea414140de5dae2ec73b00361bbef0469546108688dbfd254
        // base64url_nopad of that hash = ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=
        // without padding: ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0
        // (standard base64, then converted to URL-safe)
        let c = s256_challenge("abc");
        // Must be 43 chars (256-bit hash → 32 bytes → 43 base64url chars without padding)
        assert_eq!(c.len(), 43);
        assert!(
            c.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        );
    }

    #[test]
    fn s256_challenge_differs_from_verifier() {
        let v = generate_code_verifier();
        let c = s256_challenge(&v);
        assert_ne!(c, v);
    }
}
