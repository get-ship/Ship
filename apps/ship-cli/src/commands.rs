use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum HookCommands {
    /// Emit session.started event for the current actor (Codex SessionStart)
    SessionStart,
    /// Read stdin hook payload and emit skill.started event
    BeforeTool,
    /// Read stdin hook payload and emit skill.completed or skill.failed event
    AfterTool,
    /// Emit session.ended event for the current actor
    SessionEnd,
}

#[derive(Subcommand, Debug)]
pub enum EventsCommands {
    /// List events from the project event log
    List {
        /// Show events since this timestamp (ISO 8601) or relative (e.g. "1h", "24h")
        #[arg(long)]
        since: Option<String>,
        /// Filter by actor
        #[arg(long)]
        actor: Option<String>,
        /// Filter by entity type (workspace, session, note, etc.)
        #[arg(long)]
        entity: Option<String>,
        /// Filter by action (create, update, delete, etc.)
        #[arg(long)]
        action: Option<String>,
        /// Maximum number of events to show (default: 50)
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Output as JSON array instead of table
        #[arg(long)]
        json: bool,
    },
}


#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    /// List trashed files for the current workspace
    Trash {
        /// Force purge all trash (ignores 24h retention)
        #[arg(long)]
        purge: bool,
    },
    /// Restore the most recent trashed version of a file
    Restore {
        /// File name to restore (as shown by `ship workspace trash`)
        filename: String,
        /// Restore to this path instead of the original location
        #[arg(long)]
        dest: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum IsolationCommands {
    /// Show detected host capabilities + which strategy ship would pick
    /// for an agent run today.
    Status {
        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Force a re-probe of host capabilities (clears the cache).
    Rescan {
        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Verify the current isolation configuration is sound (decision
    /// resolves, backend implemented, no silent downgrade).
    Verify {
        #[arg(long)]
        json: bool,
    },
    /// Per-tier availability + previews for the UI picker (also useful
    /// for scripting). Returns what would happen if each tier were
    /// pinned.
    Recommendations {
        #[arg(long)]
        json: bool,
    },
    /// Show what would happen if a hypothetical tier / strategy were
    /// chosen, without persisting anything.
    Preview {
        /// Tier id: none / process / container / vm / cloud / auto
        #[arg(long)]
        tier: Option<String>,
        /// Strategy id: noop / harness / docker / youki / wsl2 / lima / fly
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Apply isolation settings atomically. When `--enforce` is set,
    /// verify must pass before settings persist.
    Init {
        #[arg(long)]
        tier: Option<String>,
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        enforce: bool,
        #[arg(long)]
        json: bool,
    },
    /// List sandboxes the daemon is currently tracking (per
    /// project, including which actors are inside each).
    Sandboxes {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// Register an MCP server (HTTP/SSE transport)
    Add {
        /// Stable server ID
        id: String,
        /// Human-readable name (defaults to id)
        #[arg(long)]
        name: Option<String>,
        /// Server URL (required for HTTP/SSE transport)
        #[arg(long)]
        url: Option<String>,
        /// Register to ~/.ship/mcp/ instead of .ship/agents/mcp.toml
        #[arg(long)]
        global: bool,
    },
    /// Register a stdio MCP server
    AddStdio {
        id: String,
        command: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        global: bool,
    },
    /// List configured MCP servers
    List,
    /// Remove an MCP server
    Remove { id: String },
}

/// `ship sui` — SUI `.sui` language intelligence (diagnostics, hover,
/// dependencies) over `UiService` (Connect). The unified authoring service
/// surfaces the compiler analysis ops; the daemon answers in-process via
/// the sui_compiler facade.
#[derive(Subcommand, Debug)]
pub enum SuiCommands {
    /// All diagnostics (parse + semantic + lint) for a `.sui` file
    Diagnostics {
        /// `.sui` file — absolute or workspace-relative
        file: String,
    },
    /// Hover info for the symbol under a 0-based (line, col) cursor
    Hover {
        file: String,
        line: u32,
        col: u32,
    },
    /// Dependency surface: custom-element tags, handled events, controller path
    Dependencies {
        file: String,
    },
}
