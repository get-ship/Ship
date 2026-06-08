use anyhow::Result;
use crate::daemon::commands::start_context;
use crate::daemon::supervision;

/// Ensure the daemon is running, starting it if needed.
///
/// 1. Check if daemon.port file exists and the daemon responds to health check.
/// 2. If not, start the daemon and wait for it to become healthy.
/// 3. Returns `(port, token)` on success.
///
/// Callers should prefer this over manual daemon_info() checks.
pub fn ensure_daemon() -> Result<(u16, String)> {
    // Fast path: daemon already running and responsive.
    if let Some((port, token)) = daemon_info_if_healthy() {
        return Ok((port, token));
    }

    // Start the daemon via the adapter so systemd-managed installs use
    // `systemctl --user start` instead of the legacy supervisor spawn.
    eprintln!("shipd: starting daemon...");
    let ctx = start_context("0.0.0.0".to_string(), false)?;
    supervision::select().start(&ctx)?;

    // Wait for the daemon to become healthy (up to 5 seconds).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some((port, token)) = daemon_info_if_healthy() {
            return Ok((port, token));
        }
    }

    anyhow::bail!("shipd: daemon started but did not become healthy within 5 seconds")
}

/// Print the daemon URL after a successful `ship daemon start`/`restart`.
///
/// The daemon binds to a per-worktree port written into
/// `~/.ship/machine/daemons/<hash>/port` *after* the supervisor has spawned
/// shipd and shipd has bound its socket. We poll the discover-daemon helper
/// for up to ~5s so the URL is the actual bound port, not the legacy
/// `SHIPD_PORT` constant. Token is deliberately omitted — pairing is the
/// canonical auth path; tokens on stdout would leak into shell history.
pub(crate) fn announce_daemon_url() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some((port, _token)) = daemon_info_if_healthy() {
            println!("→ http://localhost:{port}/");
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    eprintln!(
        "shipd: started but not yet healthy — run `ship daemon status` for the URL"
    );
}

/// Try to read daemon info and verify it responds to /api/v1/health.
pub fn daemon_info_if_healthy() -> Option<(u16, String)> {
    let cwd = std::env::current_dir().ok()?;
    let handle = ship_substrate::home::discover_running_daemon(&cwd)?;
    let port = handle.port;
    let token = handle.token.clone();

    // Quick health probe — 1 second timeout.
    let url = format!("http://127.0.0.1:{port}/api/health");
    let resp = crate::http::daemon_client()
        .get(&url)
        .timeout(std::time::Duration::from_secs(1))
        .send()
        .ok()?;

    crate::http::check_skew(resp.headers());

    if resp.status().is_success() {
        Some((port, token))
    } else {
        None
    }
}
