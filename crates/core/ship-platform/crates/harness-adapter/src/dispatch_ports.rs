//! Minimal dispatch-port value types kept in L0 ship-substrate.
//!
//! Only `LaunchCommand` (+ its `to_shell_string` impl) and `LaunchProfile`
//! remain here. They must stay in L0 because `ship-runner::harness_adapter`
//! (which must not depend on the L1 `ship-code` crate) imports them.
//!
//! All port traits (`SessionExecutor`, `IsolationStrategy`, etc.), `SessionContext`,
//! `ExitSignal`, `ExecutorHandle`, and the runner-ports types have moved to
//! `ship_code::dispatch_ports`.

use std::collections::HashMap;


/// CLI command built by a `HarnessLauncher`.
#[derive(Debug, Clone)]
pub struct LaunchCommand {
    /// Binary name (e.g. "claude", "codex").
    pub bin: String,
    /// CLI arguments.
    pub args: Vec<String>,
    /// Extra environment variables to set.
    pub env: HashMap<String, String>,
}

// `LaunchCommand::to_shell_string` + the `shell_quote` helper live in the
// `shell_quote` submodule (size-cap split). The type stays here as a port type.
mod shell_quote;

/// Everything about *how* to run the harness process.
#[derive(Clone, Debug, Default)]
pub struct LaunchProfile {
    pub harness: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub resume_session: Option<String>,
    pub session_name: Option<String>,
    pub max_budget_usd: Option<f64>,
    pub permission_mode: Option<String>,
    pub persist_session: bool,
    pub structured_output: bool,
    pub autostart_prompt: Option<String>,
    pub append_system_prompt: Option<String>,
}
