//! Daemon path discovery and env-var wiring.
//!
//! These are L0/L1 init steps the daemon's composition root used to inline:
//! discover the project + worktree roots, pick the per-worktree IPC dir,
//! set the `DECK_FIRST_PARTY_VIEWS` env var, surface a hint when a stale
//! daemon sits at the redirected main-root hash, clean up stale daemon
//! files, set the workspace env vars + deck-core's authoritative project
//! root, and derive the listening port.

use anyhow::Result;
use std::path::PathBuf;

use ship_substrate::home::ProjectRoot;

/// Resolved paths the daemon needs before it can build the runtime.
///
/// `daemon_dir` here is the per-worktree IPC dir when launched from inside
/// a git worktree, or the singleton `<root>/machine/daemon/` otherwise.
/// `ShipRuntime::new` computes the same value internally; the L2 daemon
/// reads `daemon_dir` off the runtime for any post-construction use
/// (identity files, port file) and uses this struct's copy for the
/// pre-construction steps (stale-file cleanup).
pub struct DaemonPaths {
    /// `~/.ship` — the global ship dir. Never workspace-scoped.
    pub global_dir: PathBuf,
    /// `--workspace` arg, when provided. Drives per-project artifact storage.
    pub workspace_dir: Option<PathBuf>,
    /// The discovered `ProjectRoot` — what `ProjectRoot::discover` would
    /// redirect to. May differ from `worktree_root` for linked worktrees.
    pub project_root: Option<ProjectRoot>,
    /// The worktree root the daemon is actually scoped to. Drives both
    /// the per-worktree IPC dir and the port derivation.
    pub worktree_root: Option<PathBuf>,
    /// `<root>/machine/daemons/<hash(worktree_root)>/` (worktree launch)
    /// or `<root>/machine/daemon/` (singleton).
    pub daemon_dir: PathBuf,
    /// `ShipHome` rooted at `global_dir` — convenience for the
    /// worktree-dir lookup. Stashed here so callers don't reconstruct it.
    pub home: ship_substrate::ShipHome,
}

/// Resolved binding config the runtime needs.
///
/// `host` is the L2 caller's CLI value with `--dev` already applied.
pub struct DaemonConfig {
    /// Host string (CLI arg, with dev-mode override already applied).
    pub host: String,
    /// Caller-supplied `--port` wins; otherwise derived from the worktree
    /// root via `worktree_port_for` (honors `worktrees.ports[<branch>]`
    /// pins, otherwise hashes the project root into `9315 + N % 100`).
    /// Outside any project, the default is 9315.
    pub desired_port: u16,
    /// `true` iff the operator passed `--port` explicitly. The bind-retry
    /// path treats a user-pinned port as a hard failure when busy (no
    /// increment-and-retry), since the user asked for a specific slot.
    pub user_pinned: bool,
}

/// Discover the daemon's paths: global dir, project + worktree roots, and
/// the per-worktree IPC dir. Mirrors the `daemon_dir` computation in
/// `ShipRuntime::new`; runs before the runtime is built so callers can
/// pass `daemon_dir` to `cleanup_stale_daemon_files` and similar.
pub fn resolve_daemon_paths(workspace_dir: &Option<PathBuf>) -> Result<DaemonPaths> {
    let global_dir = ship_substrate::project::get_global_dir()?;

    // Per devx-04, pick the daemon IPC dir based on workspace cwd. A daemon
    // launched from inside a git worktree writes to
    // `<root>/machine/daemons/<hash(worktree_root)>/`; bare/global startup
    // writes to the singleton `<root>/machine/daemon/`. Hashing the
    // worktree's own root (not the redirected main project root) is what
    // lets each linked worktree run its own daemon without trampling the
    // others' IPC handshakes.
    let project_probe = workspace_dir.clone().or_else(|| std::env::current_dir().ok());
    let project_root = project_probe
        .as_ref()
        .and_then(|p| ship_substrate::home::ProjectRoot::discover(p));
    let worktree_root = project_probe
        .as_ref()
        .and_then(|p| ship_substrate::home::worktree_root_for(p))
        .or_else(|| project_root.as_ref().map(|r| r.path().to_path_buf()));
    let home = ship_substrate::ShipHome::from_env();
    let daemon_dir = match worktree_root.as_ref() {
        Some(root) => home.worktree_daemon_dir(&ship_substrate::home::path_hash(root)),
        None => home.daemon_dir(),
    };

    Ok(DaemonPaths {
        global_dir,
        workspace_dir: workspace_dir.clone(),
        project_root,
        worktree_root,
        daemon_dir,
        home,
    })
}

/// Apply the env-var wiring the rest of the daemon depends on, and clean
/// stale daemon files left by a prior crashed instance.
///
/// Idempotent over env writes: never clobbers a pre-set
/// `Var::DevProjectDir` (callers that need a different SUI resolver root
/// than the daemon workspace can still override). The `DECK_FIRST_PARTY_VIEWS`
/// set is unconditional when the worktree ships an `apps/shell/src` dir.
pub fn apply_path_env_vars(paths: &DaemonPaths) -> Result<()> {
    // The ship host configures deck's first-party view root — deck itself knows
    // no app layout (see .artifacts/asset-architecture.md). Gate on the shell
    // source actually being present (a dev worktree of this repo), NOT on the
    // `--dev` flag, which dev launchers pass inconsistently. In a user's project
    // / prod install there is no `apps/shell/src`, so the env stays unset and
    // the shell bundle owns first-party views.
    if let Some(root) = paths.worktree_root.as_ref() {
        let fp_root = root.join("apps").join("shell").join("src");
        if fp_root.is_dir() {
            // SAFETY: set once at startup, before any request thread reads it.
            unsafe {
                std::env::set_var("DECK_FIRST_PARTY_VIEWS", &fp_root);
            }
            tracing::info!(path = %fp_root.display(), "shipd: DECK_FIRST_PARTY_VIEWS set");
        } else {
            tracing::warn!(path = %fp_root.display(), "shipd: DECK_FIRST_PARTY_VIEWS not set; first-party root missing");
        }
    } else {
        tracing::warn!("shipd: worktree_root is None; DECK_FIRST_PARTY_VIEWS not set");
    }

    // One-time hint for the worktree footgun: if `ProjectRoot::discover`
    // redirects to a different (main) checkout and there's a live daemon
    // sitting at the OLD (main-root) hash, surface it so the operator can
    // clean up at their leisure. Don't auto-kill — that's their call.
    if let (Some(pr), Some(wt)) = (paths.project_root.as_ref(), paths.worktree_root.as_ref())
        && pr.path() != wt.as_path()
    {
        let main_hash = ship_substrate::home::path_hash(pr.path());
        let main_dir = paths.home.worktree_daemon_dir(&main_hash);
        if let Some(handle) = ship_substrate::home::DaemonHandle::load(&main_dir)
            && handle.is_alive()
        {
            tracing::warn!(
                old_dir = %main_dir.display(),
                old_pid = handle.pid.unwrap_or(0),
                "found a daemon at the main-worktree hash for this repo; \
                 kill it if you no longer use shipd from {}",
                pr.path().display()
            );
        }
    }

    // Clean stale daemon files BEFORE setting workspace env vars — the
    // eprintln below surfaces the refusal-to-start error to the operator
    // before any subsequent log lines.
    crate::ship_runtime::init::identity_files::cleanup_stale_daemon_files(&paths.daemon_dir).map_err(|e| {
        eprintln!("\nshipd startup aborted: {e}\n");
        e
    })?;

    if let Some(w) = paths.workspace_dir.as_ref() {
        tracing::info!(workspace = %w.display(), daemon_dir = %paths.daemon_dir.display(), "shipd: project-scoped mode");
        // Make `--workspace` authoritative for the SUI resolver chain.
        // deck-core's resolver consults an in-process global set by
        // `deck::resolver::set_project_root` (the SAFE path) — see the
        // call below. The `Var::DevProjectDir` env var is kept for
        // non-deck readers (apps/shipd/src/views/discovery.rs, ship-agents
        // view_state.rs) that still key off it; deck-core no longer
        // reads it.
        //
        // Leave any pre-existing value alone — callers that need a
        // different SUI resolver root than the daemon workspace
        // (rare; mostly studio in-place editing) can still override.
        // Routed through the typed `Var` registry rather than a raw env
        // literal, so the override stays accountable to the env.rs
        // taxonomy (no raw `set_var` of a string name).
        // SAFETY: shipd's process is single-threaded at this point in
        // startup — set_var is unsound only when other threads can
        // observe the env mid-update.
        use ship_substrate::env::{self, Var};
        if env::read(Var::DevProjectDir).is_none() {
            // SAFETY: pre-async-runtime, no concurrent env access.
            unsafe { env::set(Var::DevProjectDir, w); }
        }
        // Authoritative project-root override for deck-core's resolver. Same
        // value the router reads at DeckConfig::load time (router.rs's
        // `rt.workspace_dir()`/cwd fallback) so the resolver and the
        // bundler agree on where `<root>/.deck` lives. Non-deck readers
        // (apps/shipd/src/views/discovery.rs, ship-agents view_state.rs)
        // still rely on the env var above.
        let repo_root = Some(w.clone()).or_else(|| std::env::current_dir().ok());
        if let Some(r) = repo_root {
            deck::resolver::set_project_root(r);
        }
    } else {
        tracing::info!(daemon_dir = %paths.daemon_dir.display(), "shipd: global-only mode (no --workspace given)");
    }
    Ok(())
}

/// Resolve the listening host + port. The host comes from the L2 caller
/// (CLI arg + `--dev` override). The port is caller-supplied `--port` if
/// present, otherwise derived from the worktree root via
/// `worktree_port_for` (which honors `~/.ship/machine/local.jsonc:
/// worktrees.ports[<branch>]` pins and otherwise hashes the project root
/// into `9315 + N % 100`). Using the worktree root rather than the
/// redirected main project root means each linked worktree gets its own
/// derived port slot. Outside any project, the historical default is 9315.
pub fn resolve_daemon_config(
    paths: &DaemonPaths,
    host: String,
    port: Option<u16>,
) -> Result<DaemonConfig> {
    let desired_port = port.unwrap_or_else(|| match paths.worktree_root.as_ref() {
        Some(root) => ship_substrate::home::worktree_port_for(root),
        None => 9315,
    });
    Ok(DaemonConfig {
        host,
        desired_port,
        user_pinned: port.is_some(),
    })
}
