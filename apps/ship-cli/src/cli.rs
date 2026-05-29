use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use crate::commands::{EventsCommands, HookCommands, McpCommands, SuiCommands, WorkspaceCommands};

const AFTER_HELP: &str = "\x1b[1mDaily Workflow:\x1b[0m
  ship init              Start here — scaffold .ship/
  ship use <agent-id>    Activate an agent (compiles immediately)
  ship status            Show active agent
  ship compile           Re-compile after editing config

\x1b[1mPackages:\x1b[0m
  ship add <package>     Add a dependency
  ship install           Resolve all dependencies
  ship audit             Scan for hidden Unicode (security)
  ship publish           Share your package on the registry

\x1b[1mConfiguration:\x1b[0m
  ship mcp add-stdio ... Register an MCP server
  ship config set k v    Set a user preference
  ship convert <source>  Convert harness configs to .ship/

\x1b[1mLearn More:\x1b[0m
  ship docs topics       List help topics
  ship docs <topic>      Detailed help for a topic
  https://getship.dev/docs";

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version = crate::build_version())]
#[command(about = "Agent configuration compiler — compose, compile, distribute")]
#[command(after_help = AFTER_HELP)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ── Setup ────────────────────────────────────────────────────────────────
    /// Scaffold .ship/ in the current project, or configure ~/.ship/ globally
    Init {
        /// Configure ~/.ship/ identity and defaults instead of current project
        #[arg(long)]
        global: bool,
        /// Default harness target (claude, codex, cursor, opencode)
        #[arg(long)]
        harness: Option<String>,
        /// Overwrite existing .ship/ configuration
        #[arg(long)]
        force: bool,
    },

    /// Authenticate with getship.dev
    Login,

    /// Sign out
    Logout,

    /// Show current identity
    Whoami,

    /// Daemon build/version, via the ConnectRPC op surface (ship.ops.v1)
    Meta,

    /// List or invoke the daemon op surface over ConnectRPC (the agent interface)
    Ops {
        #[command(subcommand)]
        action: OpsCommands,
    },

    /// Pair a browser session (Studio) with the local daemon
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
    },

    /// Read or write user preferences (~/.ship/config.toml)
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    // ── Daily Use ────────────────────────────────────────────────────────────
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

    /// Show the active agent and compilation status
    Status {
        #[arg(long)]
        path: Option<PathBuf>,
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

    /// Validate .ship/ config — checks TOML, skill refs, MCP fields, permissions
    Validate {
        /// Validate a single agent (omit to validate all)
        #[arg(long)]
        agent: Option<String>,
        /// Emit errors as JSON array instead of human-readable output
        #[arg(long)]
        json: bool,
        /// Path to project root (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Workspace utilities (trash, restore)
    Workspace {
        #[command(subcommand)]
        action: WorkspaceCommands,
    },

    /// Manage external MCP servers (add, list, remove)
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },

    // ── Publishing ───────────────────────────────────────────────────────────
    /// Install all dependencies declared in .ship/ship.jsonc
    Install {
        /// Fail if the lockfile would change rather than updating it
        #[arg(long)]
        frozen: bool,
        /// Skip install tracking (no network POST to registry)
        #[arg(long)]
        offline: bool,
    },

    /// Publish this package to the Ship registry
    Publish {
        /// Publish only this export (e.g. agents/skills/tdd, agents/profiles/red-green.toml)
        export_path: Option<String>,
        /// Preview what would be published without making any network requests
        #[arg(long)]
        dry_run: bool,
        /// Dist-tag for pre-release publishing (e.g. beta, next)
        #[arg(long)]
        tag: Option<String>,
    },

    /// Add a package dependency or import config from a Studio share link
    Add {
        /// Package path with optional version: github.com/owner/repo[@version]
        #[arg(required_unless_present = "from")]
        package: Option<String>,
        /// Import agent config from a Studio share link (MCP or JSON)
        #[arg(long, conflicts_with = "package")]
        from: Option<String>,
    },

    /// Scan files for hidden Unicode characters (prompt injection vectors)
    Audit {
        /// Path to scan (defaults to .ship/ in current directory)
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit findings as JSON array
        #[arg(long)]
        json: bool,
    },

    /// Convert harness config files (CLAUDE.md, .cursor/) into .ship/ format
    Convert {
        /// A getship.dev URL (e.g. https://getship.dev/p/<id>), local path, or harness config
        source: String,
    },

    // ── Inspection ───────────────────────────────────────────────────────────
    /// Query the project event log
    Events {
        #[command(subcommand)]
        action: EventsCommands,
    },

    /// Show detailed help for a topic (run `ship docs topics` to list)
    Docs {
        /// Topic name (e.g. agents, skills, mcp, compile, harnesses)
        topic: Option<String>,
    },

    /// Harness hook integration (before-tool, after-tool, session-end)
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        action: HookCommands,
    },

    /// Rebuild projection tables from the event log (dev tool — not part of the user-facing CLI)
    #[cfg(feature = "unstable")]
    #[command(hide = true)]
    RebuildProjections,

    /// Real-time dashboard — workspaces, sessions, events (htop for Ship)
    #[cfg(feature = "unstable")]
    Top,

    /// Run a named pipeline (e.g. ship run tdd "implement auth")
    Run {
        /// Pipeline name (loaded from .ship/pipelines/<name>.jsonc)
        pipeline: String,
        /// Inline spec text describing the work
        #[arg(trailing_var_arg = false)]
        spec_text: Option<String>,
        /// Path to a spec file (alternative to inline text)
        #[arg(long)]
        spec: Option<std::path::PathBuf>,
        /// Workspace to run in (auto-creates if needed)
        #[arg(long)]
        workspace: Option<String>,
        /// Pipeline variable (repeatable: --var key=value)
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        /// Show the resolved pipeline without executing
        #[arg(long)]
        dry_run: bool,
        /// Run steps as direct child processes (no tmux PTY window). Default
        /// is to launch each step in a tmux window so you can `tmux attach`
        /// and watch the agent live.
        #[arg(long)]
        headless: bool,
    },

    /// List process records (lifecycle view from the ProcessRecord projection)
    Procs {
        /// Filter to a single job slug
        #[arg(long)]
        slug: Option<String>,
        /// Filter by session kind. Valid values: agent, pipeline-step, shell.
        #[arg(long, value_name = "KIND")]
        kind: Option<String>,
        /// Cap the number of rows returned (default 100)
        #[arg(long)]
        limit: Option<usize>,
        /// Emit records as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Stream events for a slug in real time (1s polling)
    Tail {
        /// Job slug to follow
        slug: String,
        /// Filter to a single event namespace (e.g. "job", "process", "supervisor")
        #[arg(long)]
        namespace: Option<String>,
    },

    /// SUI `.sui` language intelligence — diagnostics, hover, dependencies
    Sui {
        #[command(subcommand)]
        action: SuiCommands,
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

    /// Manage the toolchain at ~/.ship/runtime/ (the sui-compiler binary; #163 retired bun)
    Runtime {
        #[command(subcommand)]
        action: RuntimeCommands,
    },

    /// Uninstall Ship from this machine (stops daemon, removes ~/.ship, scrubs PATH)
    Uninstall {
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// Show help — same as --help
    Help,

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
    /// Report how `platform.db`'s migration log relates to the migration
    /// set this binary embeds. Read-only — inspects `_sqlx_migrations` and
    /// changes nothing. The migration runner now tolerates a DB whose log
    /// is ahead of the binary on its own, so this is a pure diagnostic:
    /// use it to understand a startup migration error.
    #[command(name = "repair-migrations")]
    RepairMigrations {
        /// Backup and recreate `platform.db` when a checksum mismatch is
        /// detected. Pre-launch escape hatch: when a baseline migration is
        /// edited in place (per the migrations/README convention), every
        /// dev DB that already applied the old checksum fails to start.
        /// `--reset` backs up the affected DB to `<path>.bak-<timestamp>`
        /// and deletes the original (plus -wal and -shm), letting the
        /// next daemon start re-apply migrations from scratch. Without
        /// `--reset` the command remains a read-only diagnostic.
        #[arg(long)]
        reset: bool,
    },
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
pub enum AuthCommands {
    /// Confirm a pairing code displayed by Studio (or another browser client)
    Pair {
        /// The pairing code shown in Studio (e.g. "4F8-2K9")
        code: String,
    },
    /// List active paired sessions
    Status,
    /// Revoke paired sessions
    Revoke {
        /// Revoke only sessions whose token starts with this prefix
        #[arg(long, conflicts_with = "all")]
        token: Option<String>,
        /// Revoke all persisted sessions
        #[arg(long, conflicts_with = "token")]
        all: bool,
    },
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

#[derive(Subcommand, Debug)]
pub enum RuntimeCommands {
    /// Install or repair ~/.ship/runtime/ from local sources. Since the daemon
    /// bundles in-process via Rolldown, the only component is sui-compiler
    /// (in-repo target/{release,debug}/sui-compiler by default; override with
    /// --sui-compiler).
    Install {
        /// Path to the sui-compiler binary to copy in. Defaults to a walk-up
        /// search for target/{release,debug}/sui-compiler from cwd.
        #[arg(long = "sui-compiler", value_name = "PATH")]
        sui_compiler: Option<PathBuf>,
    },
    /// Wipe ~/.ship/runtime/ and reinstall fresh. Same flags as install.
    Update {
        #[arg(long = "sui-compiler", value_name = "PATH")]
        sui_compiler: Option<PathBuf>,
    },
    /// Verify the toolchain is healthy. Reports the first missing or
    /// non-executable component, and exits non-zero on any failure.
    Doctor,
}
