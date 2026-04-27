//! Typed `SHIP_*` environment-variable surface.
//!
//! Per ADR `home-layout-and-scope-convention`, every `SHIP_*` env var the
//! runtime reads is a variant on `Var`, and `read(Var)` is the single
//! accessor. A CI grep guard at `scripts/lint/check-raw-env-reads.sh` flags any
//! `env::var("SHIP_*")` outside this module (warn-only until Phase 2).

use serde::{Deserialize, Serialize};

/// Category taxonomy for env vars — matches the ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// User-facing configuration knobs (`SHIP_HOME`, `SHIP_API_URL`, …).
    Configuration,
    /// Runtime identity tokens propagated to child processes
    /// (`SHIP_AGENT_ID`, `SHIP_WORKSPACE_ID`, …).
    RuntimeIdentity,
    /// Development and debugging toggles (`SHIP_DEV`, `SHIP_TMUX_TESTS`, …).
    DevDebug,
    /// Legacy vars kept for backward compatibility; subject to removal.
    Legacy,
}

/// Every `SHIP_*` environment variable the runtime reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum Var {
    // ── Configuration ────────────────────────────────────────────────────
    Home,
    ApiUrl,
    RegistryUrl,
    // ── Runtime identity ─────────────────────────────────────────────────
    ActorId,
    WorkspaceId,
    SessionId,
    AgentId,
    WorktreeDir,
    RepoDir,
    MeshId,
    // ── Dev / debug ──────────────────────────────────────────────────────
    Dev,
    DefaultTerminal,
    Executor,
    TmuxTests,
    ShellDevUrl,
    DaemonPath,
    ManagedHooks,
    HooksBin,
    TelegramToken,
    TelegramChatId,
    /// `SHIP_RESPECT_WORKTREE=1` — when set, project-root resolution honors
    /// the worktree's local `.ship/` instead of redirecting to the main
    /// checkout. Used by dev tooling iterating per-worktree views/themes.
    RespectWorktree,
    // ── Legacy ───────────────────────────────────────────────────────────
    /// Legacy `SHIP_DIR` — superseded by `SHIP_HOME` / `ShipHome`.
    Dir,
}

impl Var {
    /// Canonical environment-variable name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Var::Home => "SHIP_HOME",
            Var::ApiUrl => "SHIP_API_URL",
            Var::RegistryUrl => "SHIP_REGISTRY_URL",
            Var::ActorId => "SHIP_ACTOR_ID",
            Var::WorkspaceId => "SHIP_WORKSPACE_ID",
            Var::SessionId => "SHIP_SESSION_ID",
            Var::AgentId => "SHIP_AGENT_ID",
            Var::WorktreeDir => "SHIP_WORKTREE_DIR",
            Var::RepoDir => "SHIP_REPO_DIR",
            Var::MeshId => "SHIP_MESH_ID",
            Var::Dev => "SHIP_DEV",
            Var::DefaultTerminal => "SHIP_DEFAULT_TERMINAL",
            Var::Executor => "SHIP_EXECUTOR",
            Var::TmuxTests => "SHIP_TMUX_TESTS",
            Var::ShellDevUrl => "SHIP_SHELL_DEV_URL",
            Var::DaemonPath => "SHIPD_PATH",
            Var::ManagedHooks => "SHIP_MANAGED_HOOKS",
            Var::HooksBin => "SHIP_HOOKS_BIN",
            Var::TelegramToken => "SHIP_TELEGRAM_TOKEN",
            Var::TelegramChatId => "SHIP_TELEGRAM_CHAT_ID",
            Var::RespectWorktree => "SHIP_RESPECT_WORKTREE",
            Var::Dir => "SHIP_DIR",
        }
    }

    /// Taxonomy category for this var.
    pub fn category(&self) -> Category {
        match self {
            Var::Home | Var::ApiUrl | Var::RegistryUrl => Category::Configuration,
            Var::ActorId
            | Var::WorkspaceId
            | Var::SessionId
            | Var::AgentId
            | Var::WorktreeDir
            | Var::RepoDir
            | Var::MeshId => Category::RuntimeIdentity,
            Var::Dev
            | Var::DefaultTerminal
            | Var::Executor
            | Var::TmuxTests
            | Var::ShellDevUrl
            | Var::DaemonPath
            | Var::ManagedHooks
            | Var::HooksBin
            | Var::TelegramToken
            | Var::TelegramChatId
            | Var::RespectWorktree => Category::DevDebug,
            Var::Dir => Category::Legacy,
        }
    }
}

/// Read an env var. `None` if unset or empty-string.
pub fn read(v: Var) -> Option<String> {
    match std::env::var(v.as_str()) {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// Read an env var as a non-empty `Result`-style raw value, identical to
/// `std::env::var(v.as_str())`. Used by the few call sites that branch on
/// the `VarError` variant or care about empty-string-as-set semantics.
pub fn read_raw(v: Var) -> Result<String, std::env::VarError> {
    std::env::var(v.as_str())
}

/// Set an env var by typed `Var`. Convenience for tests; production code
/// should prefer not to mutate process env.
///
/// # Safety
/// Wraps `std::env::set_var`, which is `unsafe` on edition 2024.
pub unsafe fn set<S: AsRef<std::ffi::OsStr>>(v: Var, value: S) {
    unsafe { std::env::set_var(v.as_str(), value) }
}

/// Remove an env var by typed `Var`.
///
/// # Safety
/// Wraps `std::env::remove_var`, which is `unsafe` on edition 2024.
pub unsafe fn remove(v: Var) {
    unsafe { std::env::remove_var(v.as_str()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_names_match_canonical_strings() {
        assert_eq!(Var::Home.as_str(), "SHIP_HOME");
        assert_eq!(Var::AgentId.as_str(), "SHIP_AGENT_ID");
        assert_eq!(Var::DaemonPath.as_str(), "SHIPD_PATH");
        assert_eq!(Var::Dir.as_str(), "SHIP_DIR");
    }

    #[test]
    fn categories_cover_every_var() {
        // Exhaustiveness: every variant yields a category; compilation
        // enforces this via the match in `category()`.
        let cats = [
            Var::Home.category(),
            Var::AgentId.category(),
            Var::Dev.category(),
            Var::Dir.category(),
        ];
        assert_eq!(cats[0], Category::Configuration);
        assert_eq!(cats[1], Category::RuntimeIdentity);
        assert_eq!(cats[2], Category::DevDebug);
        assert_eq!(cats[3], Category::Legacy);
    }

    #[test]
    fn read_returns_some_when_set_none_otherwise() {
        // Use a var unlikely to collide with the host environment.
        let v = Var::TelegramToken;
        let prev = read_raw(v).ok();
        // SAFETY: tests run single-threaded relative to this specific var;
        // restored before return.
        unsafe {
            remove(v);
        }
        assert_eq!(read(v), None);
        unsafe {
            set(v, "abc123");
        }
        assert_eq!(read(v).as_deref(), Some("abc123"));
        // Empty string is treated as unset.
        unsafe {
            set(v, "");
        }
        assert_eq!(read(v), None);
        unsafe {
            match prev {
                Some(val) => set(v, val),
                None => remove(v),
            }
        }
    }

    #[test]
    fn var_names_are_unique() {
        let names = [
            Var::Home,
            Var::ApiUrl,
            Var::RegistryUrl,
            Var::ActorId,
            Var::WorkspaceId,
            Var::SessionId,
            Var::AgentId,
            Var::WorktreeDir,
            Var::RepoDir,
            Var::MeshId,
            Var::Dev,
            Var::DefaultTerminal,
            Var::Executor,
            Var::TmuxTests,
            Var::ShellDevUrl,
            Var::DaemonPath,
            Var::ManagedHooks,
            Var::HooksBin,
            Var::TelegramToken,
            Var::TelegramChatId,
            Var::Dir,
        ];
        let mut seen = std::collections::HashSet::new();
        for v in names {
            assert!(seen.insert(v.as_str()), "duplicate env name: {}", v.as_str());
        }
    }
}
