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
                if let Some(id) = &app_id {
                    let _ = window.set_title(&format!("Ship — {id}"));
                    // A space window loads the THIN scaffold the DAEMON serves at
                    // /space/<id> — same-origin, so /_sui + Connect resolve to the
                    // daemon (no proxy, no vite, dev == prod). The default window
                    // (no --app) keeps the bundled hub. Discover the daemon port.
                    if let Some(port) = crate::daemon::daemon_status().port {
                        if let Ok(url) = format!("http://127.0.0.1:{port}/space/{id}").parse() {
                            let _ = window.navigate(url);
                        }
                    }
                }
                // Plant the launched app id for the frontend to route on (the hub
                // case + a belt-and-suspenders for the space case).
                let id_js = app_id.clone().unwrap_or_default();
                let _ = window.eval(format!("window.__SHIP_APP_ID__ = {id_js:?};"));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Ship shell");
}
