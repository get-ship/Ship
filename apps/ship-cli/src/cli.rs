use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use crate::commands::{HookCommands, McpCommands, WorkspaceCommands};
#[cfg(feature = "unstable")]
pub use crate::commands::EventsCommands;

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
  ship convert <source>  Convert provider configs to .ship/

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
        /// Default provider target (claude, gemini, codex, cursor)
        #[arg(long)]
        provider: Option<String>,
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
    Use {
        /// Agent ID or registry reference (e.g. rust-expert, @org/agent)
        agent_id: String,
        /// Bind to this path instead of the current directory
        #[arg(long)]
        path: Option<PathBuf>,
        /// Compile immediately after activating
        #[arg(long, default_value_t = true)]
        compile: bool,
        /// Extra skill IDs to inject at compile time (repeatable)
        #[arg(long = "with", value_name = "SKILL_ID")]
        with_skills: Vec<String>,
    },

    /// Show the active agent and compilation status
    Status {
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Compile the active agent to provider-native config files
    Compile {
        /// Compile for a specific provider only (claude, gemini, codex, cursor)
        #[arg(long)]
        provider: Option<String>,
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

    /// Manage MCP servers (serve, add, list, remove)
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },

    // ── Publishing ───────────────────────────────────────────────────────────
    /// Install all dependencies declared in .ship/ship.toml
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

    /// Convert provider config files (CLAUDE.md, .cursor/) into .ship/ format
    Convert {
        /// A getship.dev URL (e.g. https://getship.dev/p/<id>), local path, or provider config
        source: String,
    },

    // ── Inspection ───────────────────────────────────────────────────────────
    /// Query the project event log (dev tool — not part of the user-facing CLI)
    #[cfg(feature = "unstable")]
    #[command(hide = true)]
    Events {
        #[command(subcommand)]
        action: EventsCommands,
    },

    /// Show detailed help for a topic (run `ship docs topics` to list)
    Docs {
        /// Topic name (e.g. agents, skills, mcp, compile, providers)
        topic: Option<String>,
    },

    /// Provider hook integration (before-tool, after-tool, session-end)
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        action: HookCommands,
    },

    /// Launch Ship Studio -- visual IDE for agent configuration
    Studio {
        /// HTTP port for the Studio MCP server
        #[arg(long, default_value_t = 51741)]
        port: u16,
        /// Open the Studio in your default browser
        #[arg(long)]
        open: bool,
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
        /// Pipeline name (loaded from .ship/pipelines/<name>.yaml)
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
    },

    /// Manage the Ship daemon (shipd) background process
    Daemon {
        #[command(subcommand)]
        action: DaemonCommands,
    },

    /// Show help — same as --help
    Help,
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the daemon in the background
    Start {
        /// Bind address (default: 0.0.0.0)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// DEV mode — auto-pair on first localhost request, force 127.0.0.1 bind.
        /// Not for production. Logs a loud warning on startup.
        #[arg(long)]
        dev: bool,
    },
    /// Stop the running daemon process
    Stop,
    /// Show daemon status
    Status,
    /// Remove orphaned worktrees for merged and failed jobs
    Cleanup,
    /// Run the daemon in the foreground (internal — used by `start`, also useful for Docker/systemd)
    #[command(hide = true)]
    Run {
        #[arg(long, default_value = "0.0.0.0")]
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
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Confirm a pairing code displayed by Studio (or another browser client)
    Pair {
        /// The pairing code shown in Studio (e.g. "4F8-2K9")
        code: String,
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
