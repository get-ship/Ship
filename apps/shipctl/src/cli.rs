use clap::{Parser, Subcommand};
use std::path::PathBuf;

const AFTER_HELP: &str = "\x1b[1mOperator:\x1b[0m
  shipctl daemon start         Start shipd in the background
  shipctl daemon stop          Stop the running daemon
  shipctl daemon restart       Restart (stop + start, or systemctl restart)
  shipctl daemon status        Show daemon supervision + liveness
  shipctl daemon dir           Print the daemon IPC dir for this cwd
  shipctl daemon cleanup       Remove orphaned worktrees for merged/failed jobs
  shipctl daemon rebuild       Replay every projection from the event logs (drift recovery)
  shipctl daemon uninstall-service    Undo the service registration
  shipctl doctor               Diagnose ship setup — build, daemon, home

\x1b[1mConfig:\x1b[0m
  shipctl config get <key>     Read a config value
  shipctl config set <key> <v> Write a config value
  shipctl config list          List all set values
  https://getship.dev/docs";

#[derive(Parser, Debug)]
#[command(name = "shipctl")]
#[command(version = crate::build_version())]
#[command(about = "shipctl — shipd operator/daemon management. The 11 daemon verbs, doctor, and config.")]
#[command(after_help = AFTER_HELP)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Read or write user preferences (via the daemon settings service)
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Manage the Ship daemon (shipd) background process
    Daemon {
        #[command(subcommand)]
        action: DaemonCommands,
    },

    /// Diagnose ship setup — build identity, daemon status, home layout
    Doctor,
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the daemon in the background
    Start {
        /// Bind address (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// DANGER: token-less loopback auth (contributor dev only; localhost-only; compiled out of release builds)
        #[arg(long = "unsafe-skip-auth")]
        unsafe_skip_auth: bool,
    },
    /// Stop the running daemon process
    Stop,
    /// Restart the daemon (stop + start, or `systemctl restart` under systemd)
    Restart {
        /// Bind address (default: 127.0.0.1). Ignored under systemd — the unit controls host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// DANGER: token-less loopback auth (contributor dev only; localhost-only; compiled out of release builds)
        #[arg(long = "unsafe-skip-auth")]
        unsafe_skip_auth: bool,
    },
    /// Show daemon status
    Status,
    /// Print the daemon IPC directory for the current cwd. Per devx-04,
    /// resolves to `<SHIP_HOME>/machine/daemons/<hash>/` when inside a
    /// source directory and `<SHIP_HOME>/machine/daemon/` otherwise. Used by
    /// just recipes and scripts/dev/* to avoid hashing paths in shell.
    Dir,
    /// Remove orphaned worktrees for merged and failed jobs
    Cleanup,
    /// Rebuild every projection from the per-scope event logs at the
    /// XDG data root — the recovery path for boot detection of
    /// MISSING projection tables (claim c-2026-07-11T01-30-00).
    /// Runs the integrity check first and refuses on fatal events
    /// corruption.
    Rebuild {
        /// Print the plan + integrity status without rebuilding
        #[arg(long)]
        dry_run: bool,
    },
    #[command(hide = true)]
    Run {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 9315)]
        port: u16,
        #[arg(long = "unsafe-skip-auth")]
        unsafe_skip_auth: bool,
    },
    /// Internal: supervisor loop that watches and restarts the daemon on crash
    #[command(hide = true)]
    Supervise {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long = "unsafe-skip-auth")]
        unsafe_skip_auth: bool,
    },
    /// Internal: register shipd with the OS service manager (called by native installers)
    #[command(hide = true, name = "install-service")]
    InstallService {
        /// Explicit path to the shipd binary. Auto-detected when omitted.
        #[arg(long, value_name = "PATH")]
        shipd_path: Option<PathBuf>,
        /// Register the service but do not start it yet.
        #[arg(long)]
        no_start: bool,
        /// Register the service but do not enable autostart on login.
        #[arg(long)]
        no_autostart: bool,
    },
    /// Internal: undo install-service (called by native uninstallers)
    #[command(hide = true, name = "uninstall-service")]
    UninstallService,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Get a config value (e.g. shipctl config get terminal.program)
    Get {
        /// Dot-path key (e.g. terminal.program, dispatch.confirm, worktrees.dir)
        key: String,
    },
    /// Set a config value (e.g. shipctl config set terminal.program wt)
    Set {
        /// Dot-path key
        key: String,
        /// Value to set
        value: String,
    },
    /// List all set config values
    List,
}
