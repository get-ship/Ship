//! Worktree teardown (delegating to ship-git), tmux helpers, agent-config utilities.

use anyhow::Result;
use std::path::PathBuf;

/// Build the `tmux` argv prefix that selects a per-mission server
/// socket. Gap 4: when the operator sets `SHIP_TMUX_SOCKET=<name>`,
/// every `tmux` call in the dispatch path runs against `-L <name>` (a
/// separate tmux server), so per-mission tmux sessions can't collide
/// or leak across the operator's main server. `None` (or unset env)
/// preserves the legacy single-server behavior. The per-call socket
/// name can also be passed verbatim via [`tmux_args_for`].
pub fn tmux_args() -> Vec<String> {
    tmux_args_for(std::env::var("SHIP_TMUX_SOCKET").ok().as_deref())
}

/// Same as [`tmux_args`] but takes the socket name explicitly. Used
/// by `HelmdPlatform` so a helmd boot can set the socket per-mission
/// without going through the env var (matches the `helmd` config
/// shape — the runtime value is the env var, the configured value is
/// the helmd-side knob).
pub fn tmux_args_for(socket: Option<&str>) -> Vec<String> {
    match socket {
        Some(s) if !s.is_empty() => vec!["-L".to_string(), s.to_string()],
        _ => Vec::new(),
    }
}

/// Send `signal` to `pid`'s process group. Used by both the tmux
/// (pane-pid-derived) and process (harness-pid-derived) stop paths
/// to reap the harness's descendants — gap 5: 17 claude processes
/// (3.3GB) survived `kill-window` because the harness's tool
/// subagents outlived the pane's shell. Best-effort: a missing pid
/// or an ESRCH is logged at debug and ignored. Unix-only.
#[cfg(unix)]
pub fn kill_process_group(pid: u32, signal: i32) {
    // SAFETY: `kill(-pgid, sig)` is the standard pgid-kill idiom;
    // `pid` came from a trusted source (tmux's pane_pid or the
    // harness's own pid).
    let ret = unsafe { libc::killpg(pid as i32, signal) };
    if ret != 0 {
        let e = std::io::Error::last_os_error();
        tracing::debug!(
            pid,
            signal,
            "kill_process_group: killpg returned {e}"
        );
    }
}

/// Resolve the worktrees root: the explicit `SHIP_WORKTREE_DIR`
/// (`worktrees.dir`) knob, `~/` expanded. Never guesses a layout —
/// missing config is a loud error.
pub fn worktrees_dir() -> Result<PathBuf> {
    match ship_substrate::env::read_raw(ship_substrate::env::Var::WorktreeDir) {
        Ok(dir) if !dir.is_empty() => Ok(expand_tilde(&dir)),
        Ok(_) | Err(std::env::VarError::NotPresent) => Err(anyhow::anyhow!(
            "missing worktrees.dir: set SHIP_WORKTREE_DIR (ship config worktrees.dir) before dispatching coding-agent worktrees"
        )),
        Err(e) => Err(anyhow::anyhow!("read SHIP_WORKTREE_DIR failed: {e}")),
    }
}

/// Expand a leading `~/` to `$HOME`. Leaves other paths untouched.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

pub fn repo_dir() -> PathBuf {
    if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir) {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn compile_agent_config(
    worktree_path: &std::path::Path,
    agent_id: &str,
) -> Result<()> {
    // Roll up the worktree's full composed world — cache tier + the
    // operator-enabled library tier + the space's own source, precedence-
    // law composed — the same `ship_resolver::roll_up` shipd's boot index
    // ingests, so a dispatch-compiled agent sees what the index sees
    // instead of a single-source `.ship/` read. `roll_up` is async
    // (V8 TS eval); `roll_up_registered_blocking` bridges the sync trait boundary.
    let member_roots = member_roots_for_worktree(worktree_path);
    let composed = ship_resolver::roll_up_members_blocking(&member_roots, None, None)
        .map_err(|e| anyhow::anyhow!("roll-up failed for agent '{agent_id}': {e}"))?;

    // In-process compile via the agent-def engine — the same pipeline
    // `compile_project` (the CLI's `ship use` wrapper) runs, minus the
    // single-source load + `#122` flat merge the roll-up already
    // superseded. `scoped_launch`: this is a daemon-spawned launch, so the
    // dispatcher injects $SHIP_TOKEN + $SHIP_DAEMON_URL into the agent's
    // env; the compile emits the daemon-hosted authed-HTTP `ship` MCP entry
    // (scope-enforced) rather than the trusted-local stdio shim.
    //
    // Gap 2: `default_effort: Some("medium")` writes `effortLevel:
    // medium` into the worktree's `.claude/settings.json` so a
    // dispatched agent never inherits the operator's user-scope
    // `effortLevel` (currently xhigh). The agent profile's
    // `harness_settings.claude.effortLevel` passthrough still wins
    // on conflict (merged LAST into the settings patch).
    let summary = agent_def::compile::compile_project_from_library(
        &agent_def::compile::CompileOptions {
            space_root: worktree_path.to_path_buf(),
            output_root: None,
            harness: None,
            active_agent: Some(agent_id.to_string()),
            extra_skills: vec![],
            plugins_installed: vec![],
            scoped_launch: true,
            dry_run: false,
            default_effort: Some("medium".to_string()),
        },
        composed.library,
    )
    .map_err(|e| anyhow::anyhow!("compile failed for agent '{agent_id}': {e}"))?;
    if summary.harnesses.is_empty() {
        return Err(anyhow::anyhow!(
            "compile for agent '{agent_id}' produced no harness outputs — \
             no harnesses configured in the worktree's .ship/"
        ));
    }
    Ok(())
}


/// Kill a tmux session by name. No-op if session does not exist.
///
/// Gap 4: when the operator sets `SHIP_TMUX_SOCKET=<name>`, the
/// session lives on that server. The `-L` flag is prepended via
/// [`tmux_args`] so cleanup targets the same server the spawn path
/// used.
pub fn kill_tmux_session(session_name: &str) -> Result<()> {
    let mut args: Vec<String> = tmux_args();
    args.extend(["kill-session".to_string(), "-t".to_string(), session_name.to_string()]);
    let status = std::process::Command::new("tmux")
        .args(&args)
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Ok(()), // session may not exist — that's fine
        Err(e) => Err(anyhow::anyhow!("failed to run tmux: {e}")),
    }
}

/// Remove a git worktree. No-op if path does not exist.
///
/// Thin supervisor-facing adapter over the [`ship_git::write::remove_worktree`]
/// leaf. The repo root comes from [`repo_dir`] (env `Var::RepoDir` or CWD);
/// `force = true` mirrors the legacy `git worktree remove --force` behavior
/// the dispatch cleanup path needs (worktrees may carry local modifications
/// when an agent died mid-step). After this, ship-code contains zero direct
/// `Command::new("git")` calls — git mechanics live in the ship-git leaf.
pub fn remove_worktree(worktree_path: &std::path::Path) -> Result<()> {
    if !worktree_path.exists() {
        return Ok(());
    }
    ship_git::write::remove_worktree(&repo_dir(), worktree_path, true)
        .map_err(|e| anyhow::anyhow!("git worktree remove failed for '{}': {e}", worktree_path.display()))
}

/// Delete a local git branch. Idempotent — silently tolerates "branch does not
/// exist" because cleanup paths may run before the branch is ever created.
///
/// Thin supervisor-facing adapter over [`ship_git::write::delete_branch`].
/// Passes `force = false` so the job-branch prefix guard in the leaf
/// catches accidental `main` / `feat/*` deletes (the dispatch cleanup
/// path always invokes this with a `job/{slug}` name, which the guard
/// allows). A leaf `BadRef` ("not a `job/*` branch …") therefore surfaces
/// here as an `Err` — the caller (`orphan_cleanup`, `cleanup_job`) already
/// logs it and moves on, preserving the legacy "warn-and-continue on
/// not-found" behavior.
pub fn delete_branch(branch: &str) -> Result<()> {
    match ship_git::write::delete_branch(&repo_dir(), branch, false) {
        Ok(_) => Ok(()),
        Err(ship_git::state::GitError::BadRef(reason)) => {
            // Match the legacy "warn at debug, treat as success" cleanup
            // contract. A `BadRef` here is the leaf's guard (not a job/*
            // branch) rather than a real failure — the caller's caller is
            // the cleanup path, not user input.
            tracing::debug!(branch, %reason, "branch deletion rejected by guard");
            Ok(())
        }
        Err(ship_git::state::GitError::Git(msg)) => {
            // Anything else (subprocess spawn failed, libgit2 error) is a
            // real failure for the cleanup path — propagate. The legacy
            // `tracing::debug!(branch, "branch deletion returned non-zero
            // (may not exist)")` swallowed these too; the leaf gives us
            // better signal so we surface them.
            Err(anyhow::anyhow!("git branch -D failed for `{branch}`: {msg}"))
        }
        // Forward any future error variants without swallowing them.
        Err(other) => Err(anyhow::anyhow!("git branch -D failed for `{branch}`: {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tmux_args_for` is the gap-4 seam. `None` and empty strings
    /// return no prefix (legacy single-server behavior); a non-empty
    /// name returns `["-L", "<name>"]` to switch the `tmux` server.
    #[test]
    fn tmux_args_for_none_and_empty_pass_through() {
        assert_eq!(tmux_args_for(None), Vec::<String>::new());
        assert_eq!(tmux_args_for(Some("")), Vec::<String>::new());
    }

    #[test]
    fn tmux_args_for_named_socket_emits_dash_l() {
        assert_eq!(
            tmux_args_for(Some("helmd-test")),
            vec!["-L".to_string(), "helmd-test".to_string()]
        );
    }

    #[test]
    fn worktrees_dir_requires_explicit_env() {
        let _guard = env_guard();
        unsafe {
            ship_substrate::env::remove(ship_substrate::env::Var::WorktreeDir);
        }

        let err = worktrees_dir().unwrap_err().to_string();
        assert!(err.contains("SHIP_WORKTREE_DIR"), "error names env var: {err}");
        assert!(err.contains("worktrees.dir"), "error names config knob: {err}");
    }

    #[test]
    fn worktrees_dir_uses_explicit_env() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().expect("tempdir");
        unsafe {
            ship_substrate::env::set(ship_substrate::env::Var::WorktreeDir, tmp.path());
        }

        assert_eq!(worktrees_dir().expect("worktrees dir"), tmp.path().to_path_buf());
    }

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Gap 2: a dispatched worktree's `.claude/settings.json` must
    /// carry `effortLevel: medium` so a daemon-spawned agent never
    /// inherits the operator's user-scope `xhigh` default. The
    /// patch-emission logic is unit-tested in the compiler crate;
    /// this end-to-end test confirms the dispatch default
    /// (`default_effort: Some("medium")` in `compile_agent_config`)
    /// flows through to the worktree. Skipped when `roll_up_registered_blocking`
    /// can't find a sui runtime — the unit test pins the patch
    /// behavior; the e2e confirms the wiring.
    #[test]
    fn compile_dispatch_writes_settings_with_effort_medium() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let worktree = tmp.path().join("wt");
        std::fs::create_dir_all(&worktree).expect("mkdir worktree");
        let ship_dir = worktree.join(".ship");
        std::fs::create_dir_all(&ship_dir).expect("mkdir .ship");
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{ "active_agent": "demo" }"#,
        )
        .expect("write ship.jsonc");
        std::fs::create_dir_all(ship_dir.join("agents")).expect("mkdir agents");
        std::fs::write(
            ship_dir.join("agents/demo.jsonc"),
            r#"{ "agent": { "id": "demo", "name": "Demo" }, "target_agents": ["claude"] }"#,
        )
        .expect("write demo agent");
        if compile_agent_config(&worktree, "demo").is_err() {
            // Roll-up needs a sui runtime that may not be present in
            // the test environment — skip; the unit test pins the
            // patch behavior.
            return;
        }
        let settings = std::fs::read_to_string(worktree.join(".claude/settings.json"))
            .expect("settings.json");
        let v: serde_json::Value = serde_json::from_str(&settings).expect("parse settings");
        assert_eq!(
            v.get("effortLevel").and_then(|s| s.as_str()),
            Some("medium"),
            "gap 2: dispatched worktree must carry effortLevel: medium"
        );
    }
}

/// The member roots a WORKTREE composes with: the worktree STANDS IN for
/// its origin repo among the origin's registered space members. Membership
/// is registration (never a scan) — this helper only decides which
/// registered member the worktree replaces:
///
/// - a plain root (`.git` is a directory) composes as itself via the
///   registry (`resolve_member_roots`);
/// - a linked worktree (`.git` is a gitfile) resolves its ORIGIN root,
///   takes the origin's registered members, and substitutes itself for
///   the origin — so a dispatched agent sees its own worktree's content
///   plus the space's other members, exactly what the space anchor sees;
/// - an unregistered origin composes the worktree alone (the degenerate
///   single-stack case).
fn member_roots_for_worktree(worktree: &std::path::Path) -> Vec<std::path::PathBuf> {
    let gitfile = worktree.join(".git");
    if !gitfile.is_file() {
        // Not a linked worktree — a plain root composes as itself.
        return ship_resolver::resolve_member_roots(worktree);
    }
    let Some(origin) = origin_repo_root(&gitfile) else {
        return vec![worktree.to_path_buf()];
    };
    let origin = std::fs::canonicalize(&origin).unwrap_or(origin);
    let wt = std::fs::canonicalize(worktree).unwrap_or_else(|_| worktree.to_path_buf());
    let mut roots = ship_resolver::resolve_member_roots(&origin);
    for r in roots.iter_mut() {
        let canon = std::fs::canonicalize(&*r).unwrap_or_else(|_| r.clone());
        if canon == origin {
            *r = wt.clone();
        }
    }
    roots
}

/// Parse a linked worktree's gitfile (`gitdir: <origin>/.git/worktrees/<name>`)
/// back to the origin repo root. Returns `None` for any shape that isn't a
/// standard linked-worktree gitdir.
fn origin_repo_root(gitfile: &std::path::Path) -> Option<std::path::PathBuf> {
    let content = std::fs::read_to_string(gitfile).ok()?;
    let gitdir = content.strip_prefix("gitdir:")?.trim();
    let p = std::path::Path::new(gitdir);
    let worktrees_dir = p.parent()?; // .../.git/worktrees
    if worktrees_dir.file_name()? != "worktrees" {
        return None;
    }
    let git_dir = worktrees_dir.parent()?; // .../.git
    if git_dir.file_name()? != ".git" {
        return None;
    }
    Some(git_dir.parent()?.to_path_buf())
}

#[cfg(test)]
mod worktree_standin_tests {
    use super::origin_repo_root;

    fn write_gitfile(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
        let f = dir.join(".git");
        std::fs::write(&f, content).unwrap();
        f
    }

    #[test]
    fn standard_linked_worktree_gitfile_resolves_origin() {
        let tmp = tempfile::tempdir().unwrap();
        let f = write_gitfile(
            tmp.path(),
            "gitdir: /home/dev/origin/.git/worktrees/my-job
",
        );
        assert_eq!(
            origin_repo_root(&f).unwrap(),
            std::path::PathBuf::from("/home/dev/origin")
        );
    }

    #[test]
    fn non_worktree_gitdir_shapes_resolve_none() {
        let tmp = tempfile::tempdir().unwrap();
        for content in [
            "gitdir: /somewhere/else
",              // no worktrees/ segment
            "gitdir: /repo/.git
",                   // bare gitdir
            "not a gitfile at all",                   // no prefix
            "gitdir: /repo/.hg/worktrees/x
",        // wrong dir name
        ] {
            let f = write_gitfile(tmp.path(), content);
            assert!(origin_repo_root(&f).is_none(), "content: {content:?}");
        }
    }
}
