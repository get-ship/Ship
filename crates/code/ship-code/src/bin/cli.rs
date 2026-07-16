use clap::{Parser, Subcommand};
use std::path::PathBuf;

const AFTER_HELP: &str = "\x1b[1mAgent surface:\x1b[0m
  ship-code ops list             List every Connect op the daemon exposes
  ship-code ops call <name>      Invoke an op with a JSON body
  ship-code meta                 Daemon build/version fingerprint
  ship-code use <agent-id>       Activate an agent (compiles immediately)
  ship-code compile              Re-compile the active agent to harness-native files
  https://getship.dev/docs";

#[derive(Parser, Debug)]
#[command(name = "ship-code")]
#[command(version = build_version())]
#[command(about = "ship-code — agent-facing CLI. Invoke the ConnectRPC ops surface, dispatch sessions via `use`, and run the compile pipeline.")]
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
        /// dispatcher sets this; a manual `ship-code use` omits it.
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
        /// Path to source directory (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
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
        /// JSON args object, e.g. `{"status":"ok"}`
        #[arg(long)]
        json: Option<String>,
    },
}

/// Static version string for clap. Uses the canonical `BUILD_GIT_SHA` env
/// var emitted by `build.rs`; clap requires `&'static str`.
pub fn build_version() -> &'static str {
    if cfg!(feature = "cli") {
        concat!(env!("CARGO_PKG_VERSION"), "+cli.")
    } else {
        env!("CARGO_PKG_VERSION")
    }
}
