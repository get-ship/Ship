use anyhow::Result;
use crate::daemon::commands::start_context;
use crate::daemon::supervision;

const HEALTH_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);
const AUTOSTART_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

/// Ensure the daemon is running, starting it if needed. On success the
/// host socket is bound and a follow-up `crate::uds_transport::connect_transport`
/// will resolve. Callers that need the typed wire surface (settings / ops /
/// agents) go through [`crate::uds_transport::connect_transport_or_autostart`],
/// which calls this on the no-socket path and then resolves the transport.
pub fn ensure_daemon_running() -> Result<()> {
    if daemon_info_if_healthy().is_some() {
        return Ok(());
    }

    eprintln!("shipd: starting daemon...");
    let ctx = start_context("0.0.0.0".to_string(), false)?;
    supervision::select().start(&ctx)?;

    let deadline = std::time::Instant::now() + AUTOSTART_WAIT;
    while std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if daemon_info_if_healthy().is_some() {
            return Ok(());
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
/// `SHIPD_PORT` constant. Bearer credentials are deliberately omitted — they
/// are not on the loopback probe path and would only leak into shell history.
pub(crate) fn announce_daemon_url() {
    let deadline = std::time::Instant::now() + AUTOSTART_WAIT;
    while std::time::Instant::now() < deadline {
        if let Some(port) = daemon_info_if_healthy() {
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
/// Returns the port on success; `None` on any failure (no handshake,
/// dead daemon, non-loopback bind that did not respond in time).
pub fn daemon_info_if_healthy() -> Option<u16> {
    let cwd = std::env::current_dir().ok()?;
    let handle = ship_substrate::home::discover_running_daemon(&cwd)?;
    let port = handle.port;

    let url = format!("http://127.0.0.1:{port}/api/health");
    let resp = crate::http::daemon_client()
        .get(&url)
        .timeout(HEALTH_PROBE_TIMEOUT)
        .send()
        .ok()?;

    crate::http::check_skew(resp.headers());

    if resp.status().is_success() {
        Some(port)
    } else {
        None
    }
}
