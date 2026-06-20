use clap::{Parser, Subcommand};
use std::path::PathBuf;

const AFTER_HELP: &str = "\x1b[1mAgent surface:\x1b[0m
  ship ops list             List every Connect op the daemon exposes
  ship ops call <name>      Invoke an op with a JSON body
  ship meta                 Daemon build/version fingerprint

\x1b[1mOperator:\x1b[0m
  ship daemon start         Start shipd in the background
  ship daemon stop          Stop the running daemon
  ship daemon restart       Restart (stop + start, or systemctl restart)
  ship daemon status        Show daemon supervision + liveness
  ship daemon dir           Print the daemon IPC dir for this cwd
  ship daemon cleanup       Remove orphaned worktrees for merged/failed jobs
  ship daemon install-service      Register shipd with the OS service manager
  ship daemon uninstall-service    Undo the service registration
  ship doctor               Diagnose ship setup — build, daemon, home

\x1b[1mDev-tooling (use, compile, config):\x1b[0m
  ship use <agent-id>       Activate an agent (compiles immediately)
  ship compile              Re-compile the active agent to harness-native files
  ship config get <key>     Read a config value
  ship config set <key> <v> Write a config value
  ship config list          List all set values
  ship config path          Show config file location

\x1b[1mLocal:\x1b[0m
  ship docs <topic>         Detailed help for a topic
  ship build                One-shot production build (`just build` wrapper)
  https://getship.dev/docs";

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version = crate::build_version())]
#[command(about = "Ship operator + agent op-client — invoke the ConnectRPC surface, run the daemon, and use the dev workflow (use / compile / config).")]
#[command(after_help = AFTER_HELP)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Daemon build/version, via the ConnectRPC op surface (ship.ops.v1)
    Meta,

    /// List or invoke the daemon op surface over ConnectRPC (the agent interface)
    Ops {
        #[command(subcommand)]
        action: OpsCommands,
    },

    /// Read or write user preferences (~/.ship/config.toml)
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Activate an agent for the current (or specified) directory
    ///
    /// Without launch flags this only compiles and activates the agent.
    /// Pass `--launch` (interactive) or `--auto` (headless) to dispatch a
    /// session immediately. `--auto`, `--prompt`, `--spec`, and `--resume`
    /// each imply `--launch`.
    Use {
        /// Agent ID or registry reference (e.g. rust-expert, @org/agent)
        agent_id: String,
        /// Bind to this path instead of the current directory
        #[arg(long)]
        path: Option<PathBuf>,
        /// Compile immediately after activating
        #[arg(long, default_value_t = true)]
        compile: bool,
        /// Compile for a daemon-spawned scoped launch (dispatch / command-bar /
        /// sandbox): emit the daemon-hosted authed-HTTP `ship` MCP entry
        /// (scope-enforced) instead of the trusted-local stdio shim. The
        /// dispatcher sets this; a manual `ship use` omits it.
        #[arg(long)]
        scoped: bool,
        /// Extra skill IDs to inject at compile time (repeatable)
        #[arg(long = "with", value_name = "SKILL_ID")]
        with_skills: Vec<String>,

        // ── launch ──
        /// Spawn an agent session after compiling.
        #[arg(long)]
        launch: bool,
        /// Re-enter the last session for this (agent, path) pair. Implies --launch.
        #[arg(long)]
        resume: bool,
        /// Inline prompt; becomes the agent's first user turn. Implies --launch.
        #[arg(long, conflicts_with = "spec")]
        prompt: Option<String>,
        /// Path to a job spec markdown; copied to .artifacts/job-spec.md. Implies --launch.
        #[arg(long, conflicts_with = "prompt")]
        spec: Option<PathBuf>,
        /// Per-launch harness override (claude, codex, cursor, opencode).
        #[arg(long)]
        harness: Option<String>,
        /// Per-launch model override (e.g. sonnet, opus).
        #[arg(long)]
        model: Option<String>,

        // ── dispatch axes (three orthogonal concerns, replaces the
        // old single `--auto` flag — see agent-primitive-inversion
        // plan in .artifacts/agents-arc/discussions/) ──
        //
        /// Background dispatch: don't open an interactive tab/PTY
        /// attach surface. The dispatch returns a slug + studio_url;
        /// observe / push via separate commands. Implies --launch.
        #[arg(short = 'd', long = "detach")]
        detach: bool,
        /// Harness invocation mode: pty (default; subscription-billed
        /// for claude) | print (one-shot, API-billed for claude
        /// post-2026-06) | stream (server-mode, codex/opencode only).
        #[arg(long, value_name = "MODE")]
        transport: Option<String>,
        /// Don't keep the harness alive after current work. By default
        /// foreground sessions keep alive (chat shape); detached
        /// sessions do not (batch shape). This flag overrides for the
        /// rare combinations.
        #[arg(long = "no-keepalive")]
        no_keepalive: bool,
        /// Keep the harness alive after current work (push more turns
        /// later via the daemon API). Default for foreground sessions.
        #[arg(short = 'i', long = "keepalive", conflicts_with = "no_keepalive")]
        keepalive: bool,
    },

    /// Compile the active agent to harness-native config files
    Compile {
        /// Compile for a specific harness only (claude, codex, cursor, opencode)
        #[arg(long)]
        harness: Option<String>,
        /// Preview output without writing any files
        #[arg(long)]
        dry_run: bool,
        /// Path to project root (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Show detailed help for a topic (run `ship docs topics` to list)
    Docs {
        /// Topic name (e.g. agents, skills, compile, harnesses)
        topic: Option<String>,
    },

    /// Manage the Ship daemon (shipd) background process
    Daemon {
        #[command(subcommand)]
        action: DaemonCommands,
    },

    /// Diagnose ship setup — build identity, daemon status, home layout
    Doctor,

    /// One-shot production build: regen types, rebuild ship-set + ui-components,
    /// then `cargo build --release` and verify all expected outputs exist.
    /// Equivalent to `just build`; provided so users without `just` can ship.
    Build {
        /// Print each stage's command without running it
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum OpsCommands {
    /// List the available ops (reads + commands) from the registry
    List,
    /// Invoke an op by name over ConnectRPC
    Call {
        /// Op name, e.g. `git.status` or `pipeline.update_status`
        name: String,
        /// JSON args object, e.g. `{"workspace_id":"x"}`
        #[arg(long)]
        json: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the daemon in the background
    Start {
        /// Bind address (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// DEV mode — auto-pair on first localhost request, force 127.0.0.1 bind.
        /// Not for production. Logs a loud warning on startup.
        #[arg(long)]
        dev: bool,
    },
    /// Stop the running daemon process
    Stop,
    /// Restart the daemon (stop + start, or `systemctl restart` under systemd)
    Restart {
        /// Bind address (default: 127.0.0.1). Ignored under systemd — the unit controls host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// DEV mode — auto-pair on first localhost request, force 127.0.0.1 bind.
        #[arg(long)]
        dev: bool,
    },
    /// Show daemon status
    Status,
    /// Print the daemon IPC directory for the current cwd. Per devx-04,
    /// resolves to `<SHIP_HOME>/machine/daemons/<hash>/` when inside a
    /// project root and `<SHIP_HOME>/machine/daemon/` otherwise. Used by
    /// just recipes and scripts/dev/* to avoid hashing paths in shell.
    Dir,
    /// Remove orphaned worktrees for merged and failed jobs
    Cleanup,
    /// Run the daemon in the foreground (internal — used by `start`, also useful for Docker/systemd)
    #[command(hide = true)]
    Run {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 9315)]
        port: u16,
        #[arg(long)]
        dev: bool,
    },
    /// Internal: supervisor loop that watches and restarts the daemon on crash
    #[command(hide = true)]
    Supervise {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long)]
        dev: bool,
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
    /// Get a config value (e.g. ship config get terminal.program)
    Get {
        /// Dot-path key (e.g. terminal.program, dispatch.confirm, worktrees.dir)
        key: String,
    },
    /// Set a config value (e.g. ship config set terminal.program wt)
    Set {
        /// Dot-path key
        key: String,
        /// Value to set
        value: String,
    },
    /// List all set config values
    List,
    /// Show config file path
    Path,
}
