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

/// Pure parser — given a slice of argv, return the `--app=<id>` or `--app <id>`
/// value if present. `parse_app_id` is the thin shim that calls this with
/// `std::env::args().skip(1)`; the split exists so the parsing logic is
/// unit-testable without spawning a process.
fn parse_app_id_from(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if let Some(id) = arg.strip_prefix("--app=") {
            return Some(id.to_string());
        }
        if arg == "--app" {
            return iter.next().cloned();
        }
    }
    None
}

/// The space/app this runtime instance was launched for, from `--app=<id>` (or
/// `--app <id>`). A per-app OS launcher (see `launcher`) passes this so the same
/// installed binary opens the right space and brands its window. `None` is the
/// hub ("Ship" itself — the config / marketplace / settings app).
fn parse_app_id() -> Option<String> {
    parse_app_id_from(&std::env::args().skip(1).collect::<Vec<_>>())
}

/// Resolve the loopback port the webview should target, starting the daemon if
/// it isn't already up. `daemon_status` is read-only — on a cold launch its port
/// is `None`, so we must drive `daemon_start` (which spawns `shipctl daemon start`
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
/// internal view while the hub's default route is still being wired). The
/// env-override is taken as an argument (not read directly from the
/// environment) so the function is unit-testable without env mutation.
fn target_path_with(app_id: &Option<String>, shell_target: Option<&str>) -> String {
    match app_id {
        Some(id) => format!("/space/{id}"),
        None => shell_target
            .filter(|s| s.starts_with('/'))
            .map(str::to_string)
            .unwrap_or_else(|| "/".to_string()),
    }
}

/// Read the dev-override env, then route. The thin shim that closes over
/// `std::env::var("SHIP_SHELL_TARGET")`; the routing logic lives in
/// `target_path_with` so tests can pass the override directly.
fn target_path(app_id: &Option<String>) -> String {
    target_path_with(app_id, std::env::var("SHIP_SHELL_TARGET").ok().as_deref())
}

/// The loopback URL to navigate to once the daemon is reachable — actively
/// starting it if it's down — or `None` while it isn't yet.
fn connect_url(app_id: &Option<String>) -> Option<String> {
    resolve_daemon_port().map(|port| format!("http://127.0.0.1:{port}{}", target_path(app_id)))
}

/// The in-bootstrap error URL: the given page URL with its fragment set to
/// `error`, which the bootstrap page's `:target` rule turns into the visible
/// error block. Built from the webview's *actual* URL because `navigate`
/// requires an absolute `tauri::Url` (a relative string never parses) and the
/// bundled-page origin is platform-dependent — `tauri://localhost` on
/// macOS/Linux, `http://tauri.localhost` on Windows — so no scheme/host may be
/// hardcoded here.
fn error_url(page: &tauri::Url) -> tauri::Url {
    let mut url = page.clone();
    url.set_fragment(Some("error"));
    url
}

/// One tick of the passive daemon-watcher policy: how long to sleep before the
/// next read-only `daemon_status` probe, and whether the in-bootstrap `#error`
/// state should be surfaced *now*. Pure so the schedule is unit-testable; the
/// loop in `run`'s setup thread just executes it.
struct WatchTick {
    delay: Duration,
    surface_error: bool,
}

/// Poll fast (2s) through the generous startup window where a slow handshake
/// or an out-of-band start is likely to land, surface `#error` exactly once
/// when that window closes, then keep watching forever at a slower cadence —
/// the probe is a cheap handshake-file read, and a daemon appearing at any
/// later time (e.g. after the user's Retry starts one) must still be picked
/// up and navigated to.
fn watch_tick(tick: u32) -> WatchTick {
    const FAST_TICKS: u32 = 20;
    const FAST: Duration = Duration::from_secs(2);
    const SLOW: Duration = Duration::from_secs(5);
    WatchTick {
        delay: if tick < FAST_TICKS { FAST } else { SLOW },
        surface_error: tick == FAST_TICKS,
    }
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
                // same-origin so `/_deck` + Connect resolve to the daemon with no
                // proxy, dev == prod. The bundled `frontendDist` (apps/host/
                // bootstrap/) is a one-page "Connecting…" placeholder; the
                // navigate below replaces it once the daemon is reachable.
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
                // with a one-shot that already fired. The watcher never exits
                // without a daemon: after a generous window it surfaces the
                // in-bootstrap `#error` state (where Retry drives the native
                // daemon commands) but keeps probing, so a daemon appearing at
                // ANY later time is still picked up and navigated to.
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
                    // Passive self-heal: read-only probes on the `watch_tick`
                    // schedule, forever. `win.url()` (not a hardcoded origin)
                    // anchors the `#error` navigation because the bundled-page
                    // origin differs per platform; if the window can't report a
                    // URL it's being torn down and navigation is moot.
                    let mut tick: u32 = 0;
                    loop {
                        let step = watch_tick(tick);
                        if step.surface_error {
                            if let Ok(page) = win.url() {
                                let _ = win.navigate(error_url(&page));
                            }
                        }
                        std::thread::sleep(step.delay);
                        if let Some(port) = crate::daemon::daemon_status().port {
                            let url =
                                format!("http://127.0.0.1:{port}{}", target_path(&app_id));
                            if let Ok(u) = url.parse() {
                                let _ = win.navigate(u);
                            }
                            return;
                        }
                        tick = tick.saturating_add(1);
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Ship shell");
}

// Unit tests for the arg-parser / target-path / connect-url surface.
// Split into a separate file so the lib's own test names don't shadow
// `lib.rs::tests::*` and so this crate can grow more lib-side tests without
// bloating lib.rs.
#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
