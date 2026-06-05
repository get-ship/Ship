//! Ship native shell — Tauri core.
//!
//! Design: `docs/.../future/tauri-shell.md`. The webview runs the SUI shell as a
//! Connect client over loopback HTTP (the same client browser dev uses) — there
//! is deliberately **no IPC data plane and no proxy**. IPC is reserved for the
//! few genuinely-native concerns the browser can't do:
//!   - daemon lifecycle (start / status), see `daemon`,
//!   - native C3 escalation (the human approval gate), see `auth`,
//!   - the auto-updater (plugin).
//! The security posture win is the CSP-locked webview + native escalation, not a
//! bespoke transport.

mod auth;
mod daemon;
mod launcher;

use std::time::Duration;

use tauri::Manager;

/// The space/app this runtime instance was launched for, from `--app=<id>` (or
/// `--app <id>`). A per-app OS launcher (see `launcher`) passes this so the same
/// installed binary opens the right space and brands its window. `None` is the
/// hub ("Ship" itself — the config / marketplace / settings app).
fn parse_app_id() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(id) = arg.strip_prefix("--app=") {
            return Some(id.to_string());
        }
        if arg == "--app" {
            return args.next();
        }
    }
    None
}

/// Resolve the loopback port the webview should target, starting the daemon if
/// it isn't already up. `daemon_status` is read-only — on a cold launch its port
/// is `None`, so we must drive `daemon_start` (which spawns `ship daemon start`
/// and polls ~5s for the handshake). The whole discover→start→re-discover cycle
/// is retried a bounded number of times so a slow or racing launch self-heals;
/// `None` is returned only after every attempt is exhausted, letting the caller
/// surface the in-bootstrap error state instead of hanging.
fn resolve_daemon_port() -> Option<u16> {
    const ATTEMPTS: usize = 3;
    for _ in 0..ATTEMPTS {
        if let Some(port) = crate::daemon::daemon_status().port {
            return Some(port);
        }
        // Cold or down — start it (bounded internal poll) and read the port the
        // handshake reports. A start failure falls through to the next attempt.
        if let Ok(info) = crate::daemon::daemon_start() {
            if let Some(port) = info.port {
                return Some(port);
            }
        }
    }
    None
}

/// The path the webview lands on: a `--app=<id>` window opens its space; the hub
/// window opens `/` (or `SHIP_SHELL_TARGET` for dev — point straight at an
/// internal view while the hub's default route is still being wired).
fn target_path(app_id: &Option<String>) -> String {
    match app_id {
        Some(id) => format!("/space/{id}"),
        None => std::env::var("SHIP_SHELL_TARGET")
            .ok()
            .filter(|s| s.starts_with('/'))
            .unwrap_or_else(|| "/".to_string()),
    }
}

/// The loopback URL to navigate to once the daemon is reachable — actively
/// starting it if it's down — or `None` while it isn't yet.
fn connect_url(app_id: &Option<String>) -> Option<String> {
    resolve_daemon_port().map(|port| format!("http://127.0.0.1:{port}{}", target_path(app_id)))
}

/// Build and run the shell. Registers the revived plugin set (updater / dialog /
/// process / opener) and the shell's native command surface, then launches the
/// `main` window the SUI frontend is served into. When launched with `--app`,
/// brands the window for that space and plants the id for the frontend to route on.
pub fn run() {
    let app_id = parse_app_id();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            daemon::daemon_status,
            daemon::daemon_start,
            auth::request_escalation,
            launcher::install_app_launcher,
            launcher::open_space,
        ])
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                // Both hub + space windows load DAEMON-served scaffolds —
                // same-origin so `/_sui` + Connect resolve to the daemon with no
                // proxy, dev == prod. The bundled `frontendDist` (apps/shell/src-tauri/
                // bootstrap/) is a one-page "Connecting…" placeholder; the navigate
                // below replaces it once the daemon is reachable.
                if let Some(id) = &app_id {
                    let _ = window.set_title(&format!("Ship — {id}"));
                }
                // Plant the launched app id for the frontend to route on (spaces
                // also read it server-side from the scaffold; this is a belt-and-
                // suspenders for any pre-navigate code paths). Synchronous, before
                // any navigate.
                let id_js = app_id.clone().unwrap_or_default();
                let _ = window.eval(format!("window.DECK_APP_ID = {id_js:?};"));

                // Connect OFF the setup thread so the bootstrap paints
                // immediately, and KEEP watching so a slow- or late-starting
                // daemon self-heals — the window navigates the moment the daemon
                // is reachable instead of stranding the user on "Connecting…"
                // with a one-shot that already fired. Only after a generous
                // window with no daemon at all do we surface the in-bootstrap
                // `#error` state.
                let win = window.clone();
                let app_id = app_id.clone();
                std::thread::spawn(move || {
                    // Active: discover, or start it + poll (bounded in resolve).
                    if let Some(url) = connect_url(&app_id) {
                        if let Ok(u) = url.parse() {
                            let _ = win.navigate(u);
                        }
                        return;
                    }
                    // Passive self-heal: the active start didn't land a handshake;
                    // keep watching (~40s, read-only) for a daemon to appear — a
                    // slow handshake or one started out-of-band — before giving up.
                    for _ in 0..20 {
                        std::thread::sleep(Duration::from_secs(2));
                        if let Some(port) = crate::daemon::daemon_status().port {
                            let url =
                                format!("http://127.0.0.1:{port}{}", target_path(&app_id));
                            if let Ok(u) = url.parse() {
                                let _ = win.navigate(u);
                            }
                            return;
                        }
                    }
                    if let Ok(u) = "index.html#error".parse() {
                        let _ = win.navigate(u);
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Ship shell");
}
