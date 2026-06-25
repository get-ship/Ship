//! Typed `SHIP_*` environment-variable surface.
//!
//! Per ADR `home-layout-and-scope-convention`, every `SHIP_*` env var the
//! runtime reads is a variant on `Var`, and `read(Var)` is the single
//! accessor. A CI grep guard at `scripts/lint/check-raw-env-reads.sh` flags any
//! `env::var("SHIP_*")` outside this module (warn-only until Phase 2).

use serde::{Deserialize, Serialize};

/// Category taxonomy for env vars — matches the ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// User-facing configuration knobs (`SHIP_HOME`, `SHIP_API_URL`, …).
    Configuration,
    /// Runtime identity tokens propagated to child processes
    /// (`SHIP_AGENT_ID`, `SHIP_WORKSPACE_ID`, …).
    RuntimeIdentity,
    /// Development and debugging toggles (`SHIP_DEV`, `SHIP_TMUX_TESTS`,
    /// `SHIP_DEV_PROJECT_DIR`, …).
    DevDebug,
}

/// Every `SHIP_*` environment variable the runtime reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Var {
    // ── Configuration ─────────────────────────────────────────────
    Home,
    ApiUrl,
    RegistryUrl,
    /// `SHIP_DAEMON_URL` — the host daemon's HTTP base URL, injected into
    /// sandboxed actors so the in-container agent can reach the daemon's
    /// app plane (MCP / hooks / log_progress) with its scoped `SHIP_TOKEN`.
    /// For container tiers the host is rewritten to a container-reachable
    /// alias (`host.docker.internal`); Tier 0/1 actors share the host
    /// network and get the loopback origin verbatim.
    DaemonUrl,
    /// `SHIP_DAEMON_MODE=development|production` — daemon operating mode,
    /// a fallback for shipd's `--mode` flag. `production` enables immutable
    /// cache headers, disables HMR routes, enables the SSR cache, and
    /// redacts error details; `development` (the default) does none of
    /// that. Unrecognized values fall back to `development`.
    DaemonMode,
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
    TmuxTests,
    DaemonPath,
    ManagedHooks,
    HooksBin,
    TelegramToken,
    TelegramChatId,
    /// `SHIP_RESPECT_WORKTREE=1` — when set, project-root resolution honors
    /// the worktree's local `.ship/` instead of redirecting to the main
    /// checkout. Used by dev tooling iterating per-worktree views/themes.
    RespectWorktree,
    /// `SHIP_DEV_PROJECT_DIR` — project-dir override for tests + dev tooling.
    /// When set, `ship_substrate::project::get_project_dir(None)` returns this path
    /// instead of walking up from cwd. Tests use it for parallel-safe
    /// project-root isolation; production callers should never set it.
    DevProjectDir,
    /// `SHIP_DEV_DB_DIR` — directory override for the machine SQLite stores.
    /// When set, the canonical DB-path resolver
    /// (`ship_substrate::db::resolve_db_path`) returns `<dir>/<name>` instead
    /// of `<ship_home>/machine/db/<name>`. This is **workflow-set, never
    /// production-set** — the dev workflow points an iterating worktree's
    /// daemon at an isolated scratch DB so it can build a different version
    /// of the migration set without colliding with the shared canonical
    /// `platform.db`. Unset (the production and default-dev case) yields
    /// exactly the canonical path. Same disposition as `DevProjectDir`.
    DevDbDir,
    /// `SHIP_APPS_BASE_DIR` — base directory the shipd `AppSource` resolver
    /// searches for `<base>/node_modules/<package>`. Default: `<worktree>/`
    /// (the ship repo root, where the project-root `node_modules/` is
    /// expected to live once apps install via the registry). In dev
    /// typically pointed at the ship-apps workspace (e.g.
    /// `/home/madvib/dev/ship-apps`) so the resolver finds that repo's
    /// `node_modules/@ship/hub` etc. without dragging the ship monorepo
    /// into the package graph. Set by the dev start script; unset in
    /// production (the host ships a `node_modules/` next to `apps.jsonc`).
    AppsBaseDir,
}

impl Var {
    /// Canonical environment-variable name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Var::Home => "SHIP_HOME",
            Var::ApiUrl => "SHIP_API_URL",
            Var::RegistryUrl => "SHIP_REGISTRY_URL",
            Var::DaemonUrl => "SHIP_DAEMON_URL",
            Var::DaemonMode => "SHIP_DAEMON_MODE",
            Var::ActorId => "SHIP_ACTOR_ID",
            Var::WorkspaceId => "SHIP_WORKSPACE_ID",
            Var::SessionId => "SHIP_SESSION_ID",
            Var::AgentId => "SHIP_AGENT_ID",
            Var::WorktreeDir => "SHIP_WORKTREE_DIR",
            Var::RepoDir => "SHIP_REPO_DIR",
            Var::MeshId => "SHIP_MESH_ID",
            Var::Dev => "SHIP_DEV",
            Var::DefaultTerminal => "SHIP_DEFAULT_TERMINAL",
            Var::TmuxTests => "SHIP_TMUX_TESTS",
            Var::DaemonPath => "SHIPD_PATH",
            Var::ManagedHooks => "SHIP_MANAGED_HOOKS",
            Var::HooksBin => "SHIP_HOOKS_BIN",
            Var::TelegramToken => "SHIP_TELEGRAM_TOKEN",
            Var::TelegramChatId => "SHIP_TELEGRAM_CHAT_ID",
            Var::RespectWorktree => "SHIP_RESPECT_WORKTREE",
            Var::DevProjectDir => "SHIP_DEV_PROJECT_DIR",
            Var::DevDbDir => "SHIP_DEV_DB_DIR",
            Var::AppsBaseDir => "SHIP_APPS_BASE_DIR",
        }
    }

    /// Taxonomy category for this var.
    pub fn category(&self) -> Category {
        match self {
            Var::Home | Var::ApiUrl | Var::RegistryUrl | Var::DaemonUrl | Var::DaemonMode => {
                Category::Configuration
            }
            Var::ActorId
            | Var::WorkspaceId
            | Var::SessionId
            | Var::AgentId
            | Var::WorktreeDir
            | Var::RepoDir
            | Var::MeshId => Category::RuntimeIdentity,
            Var::Dev
            | Var::DefaultTerminal
            | Var::TmuxTests
            | Var::DaemonPath
            | Var::ManagedHooks
            | Var::HooksBin
            | Var::TelegramToken
            | Var::TelegramChatId
            | Var::RespectWorktree
            | Var::DevProjectDir
            | Var::DevDbDir
            | Var::AppsBaseDir => Category::DevDebug,
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
        assert_eq!(Var::DevProjectDir.as_str(), "SHIP_DEV_PROJECT_DIR");
        assert_eq!(Var::DevDbDir.as_str(), "SHIP_DEV_DB_DIR");
        assert_eq!(Var::AppsBaseDir.as_str(), "SHIP_APPS_BASE_DIR");
    }

    #[test]
    fn categories_cover_every_var() {
        // Exhaustiveness: every variant yields a category; compilation
        // enforces this via the match in `category()`.
        let cats = [
            Var::Home.category(),
            Var::AgentId.category(),
            Var::Dev.category(),
            Var::DevProjectDir.category(),
        ];
        assert_eq!(cats[0], Category::Configuration);
        assert_eq!(cats[1], Category::RuntimeIdentity);
        assert_eq!(cats[2], Category::DevDebug);
        assert_eq!(cats[3], Category::DevDebug);
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
            Var::DaemonUrl,
            Var::DaemonMode,
            Var::ActorId,
            Var::WorkspaceId,
            Var::SessionId,
            Var::AgentId,
            Var::WorktreeDir,
            Var::RepoDir,
            Var::MeshId,
            Var::Dev,
            Var::DefaultTerminal,
            Var::TmuxTests,
            Var::DaemonPath,
            Var::ManagedHooks,
            Var::HooksBin,
            Var::TelegramToken,
            Var::TelegramChatId,
            Var::RespectWorktree,
            Var::DevProjectDir,
            Var::DevDbDir,
            Var::AppsBaseDir,
        ];
        let mut seen = std::collections::HashSet::new();
        for v in names {
            assert!(seen.insert(v.as_str()), "duplicate env name: {}", v.as_str());
        }
    }
}
