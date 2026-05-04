//! `ship doctor` — diagnostic snapshot for "something feels off."
//!
//! Reports the build matrix (CLI vs running daemon), home-layout state,
//! and common-issue checks. The canonical entry point for diagnosing
//! version skew, missing layout, and daemon connectivity.

use anyhow::Result;
use std::path::Path;
use std::time::Duration;

use crate::http::daemon_client;

/// Run `ship doctor` and print a multi-section report.
pub fn run() -> Result<()> {
    let cli = build_info::build_info!();
    let home = runtime::ShipHome::from_env();

    section("ship CLI");
    kv("version", env!("CARGO_PKG_VERSION"));
    kv("build", &cli.full());

    section("ship home");
    kv("root", &home.root().display().to_string());
    let library_ok = home.library_dir().is_dir();
    let machine_ok = home.machine_dir().is_dir();
    kv("library/", if library_ok { "ok" } else { "MISSING" });
    kv("machine/", if machine_ok { "ok" } else { "MISSING" });

    section("daemon");
    match probe_daemon() {
        Some(probe) => {
            kv("status", "running");
            kv("port", &probe.port.to_string());
            kv("build", &probe.build);
            kv("commit", &probe.commit);
            kv("ship_dir", &probe.ship_dir);
            if probe.commit_short() != cli.git_sha
                && probe.commit_short() != format!("{}+dirty", cli.git_sha)
            {
                kv(
                    "skew",
                    &format!(
                        "yes — daemon {} ≠ cli {}",
                        probe.commit_short(),
                        cli.short()
                    ),
                );
            } else {
                kv("skew", "none");
            }
        }
        None => {
            kv("status", "not running");
            kv(
                "hint",
                "start with `shipd` (or `ship daemon start` for the supervised setup)",
            );
        }
    }

    section("recommended next steps");
    if !library_ok || !machine_ok {
        println!(
            "  ! home layout incomplete — start `shipd` once to populate (ensure_layout will mkdir -p)"
        );
    }
    if let Some(probe) = probe_daemon()
        && probe.commit_short() != cli.git_sha
        && probe.commit_short() != format!("{}+dirty", cli.git_sha)
    {
        println!(
            "  ! daemon was built from a different commit than this CLI — restart the daemon to pick up CLI changes"
        );
    }
    Ok(())
}

#[derive(Debug)]
struct DaemonProbe {
    port: u16,
    build: String,
    commit: String,
    ship_dir: String,
}

impl DaemonProbe {
    fn commit_short(&self) -> String {
        // The /api/health body returns the dirty-suffixed short. Strip
        // for SHA-only comparison if needed.
        self.commit.clone()
    }
}

fn probe_daemon() -> Option<DaemonProbe> {
    let home = runtime::ShipHome::from_env();
    let port_path = home.daemon_file(runtime::DaemonFile::Port);
    let port: u16 = std::fs::read_to_string(&port_path)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let url = format!("http://127.0.0.1:{port}/api/health");
    let client = daemon_client();
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(2))
        .send()
        .ok()?;
    let headers = resp.headers().clone();
    crate::http::check_skew(&headers);
    let body: serde_json::Value = resp.json().ok()?;
    let commit = body
        .get("commit")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let build = body
        .get("build")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let ship_dir = body
        .get("ship_dir")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    Some(DaemonProbe {
        port,
        build,
        commit,
        ship_dir,
    })
}

fn section(title: &str) {
    println!();
    println!("── {title} ────────────────────────────────");
}

fn kv(key: &str, val: &str) {
    println!("  {key:14} {val}");
}

#[allow(dead_code)]
fn dir_size_summary(path: &Path) -> String {
    if !path.exists() {
        return "missing".to_string();
    }
    "present".to_string()
}
