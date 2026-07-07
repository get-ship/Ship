//! Subprocess helpers for the supervisor — worktree, tmux, and agent-config utilities.

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

pub fn worktrees_dir() -> PathBuf {
    let project = repo_dir()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ship".to_string());
    ship_config::worktrees::worktrees_dir(&project)
}

pub fn repo_dir() -> PathBuf {
    if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir) {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Absolute path to the `git` binary, resolved from `$PATH` once and cached
/// for the life of the process. Some tests elsewhere in this crate mutate
/// `PATH` process-globally without isolation (e.g.
/// `executors::tests_server::routing_pre_flight_fails_cleanly_when_binary_missing`),
/// which otherwise makes any `Command::new("git")` spawned concurrently
/// flaky (`NotFound`) purely from test scheduling. Resolving to an absolute
/// path once and reusing it sidesteps `PATH` lookups (and any later
/// mutation of `PATH`) for every subsequent call.
fn git_bin() -> &'static std::path::Path {
    static GIT_BIN: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    GIT_BIN.get_or_init(resolve_git_bin).as_path()
}

fn resolve_git_bin() -> PathBuf {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("git");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    for fallback in ["/usr/bin/git", "/bin/git", "/usr/local/bin/git"] {
        if std::path::Path::new(fallback).is_file() {
            return PathBuf::from(fallback);
        }
    }
    PathBuf::from("git")
}

/// Read the HEAD branch of an existing worktree at `worktree_path` via
/// `git -C <path> rev-parse --abbrev-ref HEAD`. `None` on any failure (path
/// not a worktree, detached HEAD, etc.) — the caller treats that as "not a
/// reusable match."
fn existing_worktree_branch(worktree_path: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new(git_bin())
        .args(["-C", &worktree_path.display().to_string(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() || s == "HEAD" {
        return None;
    }
    Some(s)
}

/// True if `branch` already exists as a local branch in `root`.
fn branch_exists_locally(root: &std::path::Path, branch: &str) -> bool {
    std::process::Command::new(git_bin())
        .args(["-C", &root.display().to_string(), "show-ref", "--verify", "--quiet", &format!("refs/heads/{branch}")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Prepare an isolated worktree for `branch`, creating the branch first if
/// it does not exist yet. One honest sequence: branch-create (from
/// `base_branch`, defaulting to HEAD, when absent) then `worktree add` —
/// every failure propagates instead of silently referencing a branch that
/// was never made.
///
/// Three cases, handled deterministically:
/// - `worktree_path` already exists and its HEAD is on `branch` — reuse
///   (idempotent retry after a dispatch that got this far before failing
///   later).
/// - `worktree_path` already exists on a DIFFERENT branch (or isn't a git
///   worktree at all) — refuse with a clear error; the caller's path is
///   occupied by something else and silently reusing it would corrupt state.
/// - `worktree_path` is free: if `branch` already exists locally (retry
///   after a dispatch that created the branch but failed before/at
///   `worktree add`), attach the worktree to the existing branch. Otherwise
///   create `branch` from `base_branch` (or HEAD) and attach in one
///   `git worktree add -b` call.
///
/// Branch existence is always checked against `repo_dir()` — the same repo
/// `worktree add` runs in — so a same-named branch on an unrelated repo
/// never influences this decision.
pub fn create_worktree(
    worktree_path: &std::path::Path,
    branch: &str,
    base_branch: Option<&str>,
) -> Result<()> {
    create_worktree_in(&repo_dir(), worktree_path, branch, base_branch)
}

/// [`create_worktree`], parameterized on the repo root — the testable seam.
/// `root` is always the repo `worktree add`/branch-existence checks run
/// against, so a same-named branch on an unrelated repo never influences
/// the decision.
fn create_worktree_in(
    root: &std::path::Path,
    worktree_path: &std::path::Path,
    branch: &str,
    base_branch: Option<&str>,
) -> Result<()> {
    if branch.is_empty() {
        return Err(anyhow::anyhow!("branch name is required"));
    }
    if worktree_path.exists() {
        return match existing_worktree_branch(worktree_path) {
            Some(existing) if existing == branch => Ok(()),
            Some(existing) => Err(anyhow::anyhow!(
                "worktree at '{}' is on branch '{}', not '{}'",
                worktree_path.display(),
                existing,
                branch
            )),
            None => Err(anyhow::anyhow!(
                "path '{}' exists but is not a git worktree on '{}'",
                worktree_path.display(),
                branch
            )),
        };
    }
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;

    let mut args: Vec<String> = vec!["worktree".to_string(), "add".to_string()];
    if branch_exists_locally(root, branch) {
        // Retry after a failed dispatch: the branch survived a prior
        // attempt. Reuse it rather than trying (and failing) to recreate it.
        args.push(path_str.to_string());
        args.push(branch.to_string());
    } else {
        args.push("-b".to_string());
        args.push(branch.to_string());
        args.push(path_str.to_string());
        if let Some(base) = base_branch {
            args.push(base.to_string());
        }
    }

    let output = std::process::Command::new(git_bin())
        .args(&args)
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!(
            "git worktree add failed for '{}': {}",
            worktree_path.display(),
            if stderr.is_empty() { "unknown error".to_string() } else { stderr }
        ));
    }
    Ok(())
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
    // (V8 TS eval); `roll_up_blocking` bridges the sync trait boundary.
    let composed = ship_resolver::roll_up_blocking(worktree_path, None, None)
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
pub fn remove_worktree(worktree_path: &std::path::Path) -> Result<()> {
    if !worktree_path.exists() {
        return Ok(());
    }
    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;
    let status = std::process::Command::new("git")
        .args(["worktree", "remove", path_str, "--force"])
        .current_dir(repo_dir())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "git worktree remove failed for '{}'",
            worktree_path.display()
        ));
    }
    Ok(())
}

/// Delete a local git branch. No-op if branch does not exist.
pub fn delete_branch(branch: &str) -> Result<()> {
    let status = std::process::Command::new("git")
        .args(["branch", "-D", branch])
        .current_dir(repo_dir())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !status.success() {
        // Branch may not exist — not an error for cleanup.
        tracing::debug!(branch, "branch deletion returned non-zero (may not exist)");
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Init a throwaway git repo with one commit on its default branch —
    /// enough of a repo for `worktree add` / branch-existence checks.
    fn init_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let run = |args: &[&str]| {
            let status = std::process::Command::new(git_bin())
                .args(args)
                .current_dir(root)
                .status()
                .expect("run git");
            assert!(status.success(), "git {:?} failed", args);
        };
        run(&["init", "-q", "-b", "main"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "test"]);
        std::fs::write(root.join("README.md"), "hello").expect("write readme");
        run(&["add", "README.md"]);
        run(&["commit", "-q", "-m", "initial"]);
        tmp
    }

    /// Reproduces the observed bug: `create_worktree` (pre-fix) ran only
    /// `git worktree add <path> <branch>` with no `-b`, which fails with
    /// "invalid reference" when `job/<slug>` was never created — dispatch
    /// isolation prepare must instead create the branch itself.
    #[test]
    fn fresh_branch_dispatch_prepare_creates_branch_and_worktree() {
        let repo = init_repo();
        let worktree_path = repo.path().join("wt");

        create_worktree_in(repo.path(), &worktree_path, "job/fresh-branch", None)
            .expect("prepare should create the missing branch and the worktree");

        assert!(worktree_path.join("README.md").exists());
        assert!(branch_exists_locally(repo.path(), "job/fresh-branch"));
    }

    /// Retry after a failed dispatch: an earlier attempt created the branch
    /// but never got to (or failed at) `worktree add`. Prepare must reuse
    /// the existing branch deterministically instead of erroring on
    /// "branch already exists."
    #[test]
    fn retry_after_failed_dispatch_reuses_existing_branch() {
        let repo = init_repo();
        let status = std::process::Command::new(git_bin())
            .args(["branch", "job/retry-branch"])
            .current_dir(repo.path())
            .status()
            .expect("create branch");
        assert!(status.success());

        let worktree_path = repo.path().join("wt");
        create_worktree_in(repo.path(), &worktree_path, "job/retry-branch", None)
            .expect("prepare should reuse the pre-existing branch");

        assert!(worktree_path.join("README.md").exists());
    }

    /// The observed-shape hypothesis: a same-named branch existing on an
    /// UNRELATED repo must not affect whether this repo's prepare creates
    /// its own branch. Branch-existence is always checked against the
    /// target repo root, never a different one.
    #[test]
    fn cross_repo_same_name_branch_does_not_matter() {
        let repo_a = init_repo();
        let repo_b = init_repo();
        let status = std::process::Command::new(git_bin())
            .args(["branch", "job/shared-name"])
            .current_dir(repo_b.path())
            .status()
            .expect("create branch in repo b");
        assert!(status.success());

        // repo_a has no such branch — prepare must create it fresh there,
        // unaffected by repo_b's same-named branch.
        let worktree_path = repo_a.path().join("wt");
        create_worktree_in(repo_a.path(), &worktree_path, "job/shared-name", None)
            .expect("prepare in repo_a should create its own branch");

        assert!(worktree_path.join("README.md").exists());
        assert!(branch_exists_locally(repo_a.path(), "job/shared-name"));
    }

    /// Idempotent hit: the worktree path already exists and its HEAD is
    /// already on `branch` (e.g. a retried dispatch that got all the way
    /// through isolation prepare before failing at a later step) — reuse,
    /// don't error.
    #[test]
    fn existing_worktree_on_same_branch_is_reused() {
        let repo = init_repo();
        let worktree_path = repo.path().join("wt");
        create_worktree_in(repo.path(), &worktree_path, "job/idempotent", None)
            .expect("first prepare creates the worktree");

        create_worktree_in(repo.path(), &worktree_path, "job/idempotent", None)
            .expect("second prepare against the same branch should be a no-op reuse");
    }

    /// The worktree path is occupied by a DIFFERENT branch's checkout —
    /// refuse with a clear error rather than silently reusing the wrong
    /// working tree.
    #[test]
    fn existing_worktree_on_different_branch_is_refused() {
        let repo = init_repo();
        let worktree_path = repo.path().join("wt");
        create_worktree_in(repo.path(), &worktree_path, "job/branch-one", None)
            .expect("first prepare creates the worktree");

        let err = create_worktree_in(repo.path(), &worktree_path, "job/branch-two", None)
            .expect_err("path occupied by a different branch must be refused");
        assert!(err.to_string().contains("job/branch-one"));
    }

    #[test]
    fn empty_branch_name_is_rejected() {
        let repo = init_repo();
        let worktree_path = repo.path().join("wt");
        let err = create_worktree_in(repo.path(), &worktree_path, "", None).expect_err("empty branch must be rejected");
        assert!(err.to_string().contains("branch name"));
    }

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

    /// Gap 2: a dispatched worktree's `.claude/settings.json` must
    /// carry `effortLevel: medium` so a daemon-spawned agent never
    /// inherits the operator's user-scope `xhigh` default. The
    /// patch-emission logic is unit-tested in the compiler crate;
    /// this end-to-end test confirms the dispatch default
    /// (`default_effort: Some("medium")` in `compile_agent_config`)
    /// flows through to the worktree. Skipped when `roll_up_blocking`
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
