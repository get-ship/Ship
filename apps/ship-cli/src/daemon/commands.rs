use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::daemon::ensure::announce_daemon_url;
use crate::daemon::paths::ship_dir;
use crate::daemon::pidfile;
use crate::daemon::service_registration;
use crate::daemon::status::daemon_status;
use crate::daemon::cleanup::daemon_cleanup;
use crate::daemon::supervise::daemon_supervise;
use crate::daemon::supervision::StartContext;

pub enum DaemonCommand {
    Start { host: String, unsafe_skip_auth: bool },
    Stop,
    Restart { host: String, unsafe_skip_auth: bool },
    Status,
    /// Print the daemon IPC directory for cwd. Powers `just status` /
    /// `scripts/dev/*` without those reaching into ShipHome internals.
    Dir,
    Cleanup,
    /// Rebuild every projection from the per-scope event logs at the
    /// XDG data root — the recovery path for boot detection of
    /// MISSING projection tables (claim c-2026-07-11T01-30-00).
    /// Refuses on fatal events corruption.
    Rebuild { dry_run: bool },
    /// Internal: supervisor loop that watches and restarts the daemon.
    Run { host: String, port: u16, unsafe_skip_auth: bool },
    Supervise { host: String, unsafe_skip_auth: bool },
    /// Internal: called by native installer post-install hooks to
    /// register shipd with the OS service manager. Not user-facing.
    InstallService { shipd_path: Option<PathBuf>, no_start: bool, no_autostart: bool },
    /// Internal: called by uninstall flows to undo `InstallService`.
    UninstallService,
}

pub fn run_daemon(cmd: DaemonCommand) -> Result<()> {
    match cmd {
        DaemonCommand::Start { host, unsafe_skip_auth } => {
            let ctx = start_context(host, unsafe_skip_auth)?;
            check_version_mismatch(&ctx.global_dir)?;
            crate::daemon::supervision::select().start(&ctx)?;
            announce_daemon_url();
            Ok(())
        }
        DaemonCommand::Stop => crate::daemon::supervision::select().stop(),
        DaemonCommand::Restart { host, unsafe_skip_auth } => {
            let ctx = start_context(host, unsafe_skip_auth)?;
            crate::daemon::supervision::select().restart(&ctx)?;
            announce_daemon_url();
            Ok(())
        }
        DaemonCommand::Status => daemon_status(),
        DaemonCommand::Cleanup => daemon_cleanup(),
        DaemonCommand::Rebuild { dry_run } => daemon_rebuild(dry_run),
        DaemonCommand::Dir => {
            println!("{}", ship_substrate::home::current_daemon_dir().display());
            Ok(())
        }
        DaemonCommand::Run { host, port, unsafe_skip_auth } => daemon_run(host, port, unsafe_skip_auth),

        DaemonCommand::Supervise { host, unsafe_skip_auth } => daemon_supervise(host, unsafe_skip_auth),
        DaemonCommand::InstallService { shipd_path, no_start, no_autostart } => {
            let report = service_registration::install_service(
                service_registration::InstallOptions {
                    start_now: !no_start,
                    enable_autostart: !no_autostart,
                    shipd_path,
                },
            )?;
            println!("{}", report.message);
            Ok(())
        }
        DaemonCommand::UninstallService => {
            let report = service_registration::uninstall_service()?;
            println!("{}", report.message);
            Ok(())
        }
    }
}

pub(super) fn start_context(host: String, unsafe_skip_auth: bool) -> Result<StartContext> {
    // Dev mode pins the bind to localhost regardless of --host so that
    // the relaxed auth surface is never exposed on a LAN. Applied here
    // so both adapters see the same final host.
    let host = if unsafe_skip_auth { "127.0.0.1".to_string() } else { host };
    Ok(StartContext {
        host,
        unsafe_skip_auth,
        global_dir: ship_dir()?,
    })
}

/// Refuse `ship daemon start` when a live daemon is running a different
/// version than the CLI we're invoking. This catches the "copy a fresh
/// shipd into ~/.local/bin and forget to restart" footgun, where the
/// stale process keeps serving old code while the operator thinks the
/// new one is live.
///
/// Only `start` checks. `restart` is the remediation, so version mismatch
/// during restart is expected — we *want* to replace the old process.
/// `stop` doesn't care: shutting down a stale daemon is never wrong.
pub(super) fn check_version_mismatch(global_dir: &Path) -> Result<()> {
    let Some((record, true)) = pidfile::read(global_dir)? else {
        return Ok(());
    };
    let installed = env!("CARGO_PKG_VERSION");
    if record.version != installed {
        anyhow::bail!(
            "shipd version mismatch: running {} (pid {}), installed {}.\n\
             Use `ship daemon restart` to replace the running daemon.",
            record.version,
            record.pid,
            installed
        );
    }
    Ok(())
}

pub(super) fn daemon_run(host: String, port: u16, unsafe_skip_auth: bool) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("cannot build tokio runtime")?;
    rt.block_on(shipd::run_network(host, Some(port), unsafe_skip_auth, None, shipd::DaemonMode::Development, None, None))
}

/// `ship daemon rebuild` — recovery path for boot detection of MISSING
/// projection tables (claim c-2026-07-11T01-30-00).
///
/// Resolves the XDG data root (or SHIP_HOME-collapsed `<base>/data`),
/// runs the per-scope integrity check, and on `ProjectionsCorrupt`
/// invokes `rebuild::rebuild_projections` against the same root. Refuses
/// to rebuild when the integrity check returns `EventsCorrupt` — that
/// is fatal and operator-actionable, not drift.
///
/// In `--dry-run` mode the check is run and the eventual decision
/// reported without rebuilding.
pub(super) fn daemon_rebuild(dry_run: bool) -> Result<()> {
    use ship_substrate::db::integrity::{IntegrityStatus, check_platform_integrity};
    use ship_substrate::db::rebuild;

    let ship_dir = ship_dir()?;
    println!("checking integrity at {}…", ship_dir.display());
    match check_platform_integrity(&ship_dir)? {
        IntegrityStatus::Ok => {
            println!("integrity OK; nothing to rebuild");
            Ok(())
        }
        IntegrityStatus::ProjectionsCorrupt { details } => {
            println!("projection drift detected: {details}");
            if dry_run {
                println!("dry-run: not rebuilding");
                return Ok(());
            }
            let report = rebuild::rebuild_projections(&ship_dir)
                .context("rebuild_projections failed")?;
            println!(
                "rebuilt: {} events replayed, {} projections refreshed ({} ms)",
                report.events_replayed,
                report.projections_rebuilt.len(),
                report.duration_ms,
            );
            for name in &report.projections_rebuilt {
                println!("  • {name}");
            }
            // Re-check so we report the post-rebuild state.
            match check_platform_integrity(&ship_dir)? {
                IntegrityStatus::Ok => println!("post-rebuild integrity: OK"),
                IntegrityStatus::ProjectionsCorrupt { details } => {
                    println!("post-rebuild integrity: STILL DRIFTING — {details}");
                }
                IntegrityStatus::EventsCorrupt { details } => {
                    println!("post-rebuild integrity: FATAL — {details}");
                }
            }
            Ok(())
        }
        IntegrityStatus::EventsCorrupt { details } => {
            anyhow::bail!("events corruption: {details}\nRebuild refused. Restore from backup.")
        }
    }
}

