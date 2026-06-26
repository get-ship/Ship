//! Daemon IPC handshake files.
//!
//! The daemon writes all four files atomically on start into `machine/daemon/`
//! and removes them on clean stop. Stale files signal a crash. This module
//! names the files; path resolution lives in `ship_home.rs`.
//!
//! Per devx-04, multi-daemon mode also lays out per-worktree daemons under
//! `machine/daemons/<hash>/` — each worktree gets its own port/token/pid set.
//! `DaemonHandle::load` and `discover_running_daemon` read these layouts.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

// `DaemonFile` (the handshake-file enum) lives in the compiler-free `ship-paths`
// leaf so `ShipHome::daemon_file` can take it without the substrate frontier.
// Re-exported here so `ship_substrate::home::daemon::DaemonFile` and the
// `home::mod` / crate-root re-exports keep resolving unchanged.
pub use ship_paths::DaemonFile;

// ── Multi-daemon discovery (devx-04) ─────────────────────────────────────────

/// First 8 hex chars of `sha256(canonical_path)`. Stable across processes;
/// short enough to type. Used as the directory key under `machine/daemons/`.
pub fn path_hash(path: &Path) -> String {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(8);
    for byte in digest.iter().take(4) {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Walk ancestors of `from` looking for the nearest `.git` entry — file or
/// directory. Returns that ancestor canonicalized, mirroring the semantics
/// of `git rev-parse --show-toplevel`:
///
/// - main checkout: `.git` is a directory, ancestor is the repo root
/// - linked worktree: `.git` is a FILE pointing at `<main>/.git/worktrees/<name>`,
///   ancestor is the linked worktree's own root (NOT the main repo's root)
///
/// `None` outside any git tree. Used as the hash input for per-worktree
/// daemon dirs and port derivation so each worktree gets its own slot.
pub fn worktree_root_for(from: &Path) -> Option<PathBuf> {
    for ancestor in from.ancestors() {
        if ancestor.join(".git").exists() {
            return Some(std::fs::canonicalize(ancestor).unwrap_or_else(|_| ancestor.to_path_buf()));
        }
    }
    None
}

/// Loaded view of one daemon's IPC handshake — port, token, pid (and
/// optional workspace path / build SHA on multi-daemon installs).
#[derive(Debug, Clone)]
pub struct DaemonHandle {
    /// Directory the handle was loaded from (`machine/daemon/` or a
    /// `machine/daemons/<hash>/` sibling).
    pub dir: PathBuf,
    pub port: u16,
    pub token: String,
    pub pid: Option<u32>,
    pub workspace: Option<PathBuf>,
    pub build_sha: Option<String>,
}

impl DaemonHandle {
    /// Read `port`, `token`, `pid`, `workspace`, `sha` from `dir`. Returns
    /// `None` when the directory is missing or the handshake is incomplete
    /// (e.g. port file absent).
    pub fn load(dir: &Path) -> Option<Self> {
        let port = read_trim(dir.join("port")).and_then(|s| s.parse::<u16>().ok())?;
        let token = read_trim(dir.join("token"))?;
        let pid = read_trim(dir.join("pid")).and_then(|s| s.parse::<u32>().ok());
        let workspace = read_trim(dir.join("workspace")).map(PathBuf::from);
        let build_sha = read_trim(dir.join("sha"));
        Some(Self {
            dir: dir.to_path_buf(),
            port,
            token,
            pid,
            workspace,
            build_sha,
        })
    }

    /// True when `pid` is recorded AND the process is currently alive.
    /// Best-effort — a `false` result also covers "no pid file at all".
    pub fn is_alive(&self) -> bool {
        match self.pid {
            Some(pid) => pid_alive(pid),
            None => false,
        }
    }
}

/// Hash input for the daemon dir keyed off `from`. Prefers the git
/// worktree root (so each linked worktree gets its own slot); falls back
/// to the redirected project root for non-git trees with a bare `.ship/`.
fn daemon_hash_input(from: &Path) -> Option<PathBuf> {
    worktree_root_for(from)
        .or_else(|| crate::home::ProjectRoot::discover(from).map(|r| r.path().to_path_buf()))
}

/// Discover the running daemon for `from`. Order:
///
/// 1. If `from` lives in a git worktree (or under a `.ship/` project), look
///    for a per-worktree daemon at `machine/daemons/<hash(worktree_root)>/`.
/// 2. If `from` is a LINKED git worktree (`.git` is a file), the main
///    project's daemon serves it — look at `machine/daemons/<hash(main_repo)>/`.
///    Needed for `ship hook` invocations from dispatched worktrees: the
///    daemon hosting the dispatch lives at the main project's hash, not
///    the linked worktree's.
/// 3. Fall back to the global `machine/daemon/`.
///
/// Returns `None` when no location holds a complete handshake.
pub fn discover_running_daemon(from: &Path) -> Option<DaemonHandle> {
    let home = crate::ShipHome::from_env();
    if let Some(root) = daemon_hash_input(from) {
        let hash = path_hash(&root);
        let dir = home.worktree_daemon_dir(&hash);
        if let Some(handle) = DaemonHandle::load(&dir) {
            return Some(handle);
        }
        // Linked-worktree fallback: hooks fired from a dispatched worktree
        // need to reach the main project's daemon. Resolve `.git` to its
        // common dir; take the parent of `.git/` as the main repo root.
        if let Some(main_root) = main_repo_root_for(&root) {
            let main_hash = path_hash(&main_root);
            if main_hash != hash {
                let dir = home.worktree_daemon_dir(&main_hash);
                if let Some(handle) = DaemonHandle::load(&dir) {
                    return Some(handle);
                }
            }
        }
    }
    DaemonHandle::load(&home.daemon_dir())
}

/// For a path that is itself a linked git worktree, follow the `.git`
/// file's gitdir pointer back to the main repository root. Returns
/// `None` for non-worktree paths (where `.git` is a directory).
fn main_repo_root_for(worktree_root: &Path) -> Option<PathBuf> {
    let dot_git = worktree_root.join(".git");
    if !dot_git.is_file() {
        return None;
    }
    let contents = std::fs::read_to_string(&dot_git).ok()?;
    // `.git` file is `gitdir: <path>` where <path> is
    // `<main_repo>/.git/worktrees/<name>`. Take the gitdir, strip the
    // `/worktrees/<name>` suffix and then strip `/.git` to land on the
    // main repo root.
    let gitdir_line = contents
        .lines()
        .find_map(|l| l.strip_prefix("gitdir:"))?
        .trim();
    let gitdir = PathBuf::from(gitdir_line);
    // Walk up: <gitdir>/.. = .git/worktrees, .. = .git, .. = main repo root
    let main_root = gitdir.parent()?.parent()?.parent()?;
    std::fs::canonicalize(main_root).ok()
}

/// The directory the daemon serving `from` should write its IPC files to.
///
/// Resolution mirrors `discover_running_daemon`: if `from` lives inside a
/// git worktree, returns `machine/daemons/<hash(worktree_root)>/` — each
/// linked worktree gets its own slot, so they don't trample each other's
/// IPC handshakes. Outside any project, returns the singleton `machine/daemon/`.
pub fn daemon_dir_for(from: &Path) -> PathBuf {
    let home = crate::ShipHome::from_env();
    match daemon_hash_input(from) {
        Some(root) => home.worktree_daemon_dir(&path_hash(&root)),
        None => home.daemon_dir(),
    }
}

/// Same as `daemon_dir_for(cwd)` — convenience wrapper for callers that
/// don't already have a path in hand.
pub fn current_daemon_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    daemon_dir_for(&cwd)
}

/// Default-derived port for a per-worktree daemon: `9315 + N % 100`
/// where `N` is the first 16 bits of the path hash (interpreted as
/// big-endian). Gives a stable port per project root across restarts;
/// collisions in the mod-100 window get resolved by bind-retry.
pub fn derive_worktree_port(hash: &str) -> u16 {
    const BASE: u16 = 9315;
    let prefix = hash.get(..4).unwrap_or("0000");
    let n = u32::from_str_radix(prefix, 16).unwrap_or(0);
    BASE + ((n % 100) as u16)
}

/// Resolve the port shipd should bind for a given project root.
///
/// Lookup order:
/// 1. `~/.ship/machine/local.jsonc:worktrees.ports[<current_branch>]` —
///    user-pinned override; useful for stable browser-extension ports.
/// 2. `derive_worktree_port(path_hash(project_root))` — hash-derived default.
///
/// `current_branch` is the branch checked out at `project_root`; resolved
/// via `git symbolic-ref --short HEAD`. A detached HEAD or non-git tree
/// skips the pin and falls through to derivation.
/// Minimal local-settings shape for port-pin lookup only.
/// Full `LocalSettings` lives in `config::local` (L1); this avoids that dep.
#[derive(Default, Deserialize)]
struct PortPinSettings {
    worktrees: Option<PortPinWorktrees>,
}

#[derive(Default, Deserialize)]
struct PortPinWorktrees {
    #[serde(default)]
    ports: std::collections::HashMap<String, u16>,
}

pub fn worktree_port_for(project_root: &Path) -> u16 {
    if let Some(branch) = current_branch(project_root) {
        let home = crate::ShipHome::from_env();
        if let Ok(raw) = std::fs::read_to_string(home.local_path()) {
            let stripped = crate::fs_util::strip_jsonc_comments(&raw);
            if let Ok(s) = serde_json::from_str::<PortPinSettings>(&stripped)
                && let Some(wt) = s.worktrees
                    && let Some(&port) = wt.ports.get(&branch)
                {
                    return port;
                }
        }
    }
    derive_worktree_port(&path_hash(project_root))
}

fn current_branch(project_root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn read_trim<P: AsRef<Path>>(path: P) -> Option<String> {
    std::fs::read_to_string(path.as_ref()).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) is the standard liveness probe; signal 0 doesn't
    // deliver, just checks whether the process exists and is signalable.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(windows)]
fn pid_alive(pid: u32) -> bool {
    use std::process::Command;
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}")])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

#[cfg(not(any(unix, windows)))]
fn pid_alive(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn file_names_are_stable() {
        assert_eq!(DaemonFile::Port.as_str(), "port");
        assert_eq!(DaemonFile::Token.as_str(), "token");
        assert_eq!(DaemonFile::Pid.as_str(), "pid");
        assert_eq!(DaemonFile::Socket.as_str(), "socket");
    }

    #[test]
    fn path_hash_is_stable() {
        let tmp = TempDir::new().unwrap();
        let a = path_hash(tmp.path());
        let b = path_hash(tmp.path());
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn path_hash_differs_across_paths() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        assert_ne!(path_hash(tmp1.path()), path_hash(tmp2.path()));
    }

    #[test]
    fn worktree_root_distinguishes_main_from_linked() {
        // Simulate the on-disk layout `git worktree add` produces:
        //   <tmp>/main/.git/worktrees/feat/   ← linked worktree's gitdir
        //   <tmp>/wt/.git                     ← pointer file
        // worktree_root_for must return the linked worktree's own root,
        // not the main repo's root.
        let tmp = TempDir::new().unwrap();
        let main = tmp.path().join("main");
        let common_git = main.join(".git");
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(common_git.join("worktrees").join("feat")).unwrap();
        std::fs::create_dir_all(wt.join("src")).unwrap();
        std::fs::write(
            wt.join(".git"),
            format!("gitdir: {}\n", common_git.join("worktrees/feat").display()),
        )
        .unwrap();

        let main_root = worktree_root_for(&main.join("src")).unwrap();
        let wt_root = worktree_root_for(&wt.join("src")).unwrap();

        // Without the fix, both ancestor walks landed on `main` for linked
        // worktrees (via ProjectRoot::discover's redirect); after the fix
        // each worktree returns its own root.
        assert_ne!(main_root, wt_root);
        assert_eq!(path_hash(&main_root).len(), 8);
        assert_ne!(
            path_hash(&main_root),
            path_hash(&wt_root),
            "daemon dir hash must differ between main and linked worktrees"
        );
    }

    #[test]
    fn worktree_root_returns_none_outside_git() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("not-a-repo");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(worktree_root_for(&dir).is_none());
    }

    #[test]
    fn handle_load_reads_full_handshake() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("port"), "9315\n").unwrap();
        std::fs::write(tmp.path().join("token"), "tok-abc\n").unwrap();
        std::fs::write(tmp.path().join("pid"), "12345\n").unwrap();
        std::fs::write(tmp.path().join("workspace"), "/proj\n").unwrap();
        std::fs::write(tmp.path().join("sha"), "abc1234\n").unwrap();

        let handle = DaemonHandle::load(tmp.path()).unwrap();
        assert_eq!(handle.port, 9315);
        assert_eq!(handle.token, "tok-abc");
        assert_eq!(handle.pid, Some(12345));
        assert_eq!(handle.workspace.unwrap(), PathBuf::from("/proj"));
        assert_eq!(handle.build_sha.unwrap(), "abc1234");
    }

    #[test]
    fn handle_load_returns_none_when_port_missing() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("token"), "x").unwrap();
        assert!(DaemonHandle::load(tmp.path()).is_none());
    }

    #[test]
    fn derive_worktree_port_is_in_window() {
        for hash in &["00000000", "abcd1234", "ffffffff", "deadbeef"] {
            let p = derive_worktree_port(hash);
            assert!((9315..=9414).contains(&p), "{hash} → {p} outside window");
        }
    }

    #[test]
    fn derive_worktree_port_is_stable() {
        let h = "ab12cd34";
        assert_eq!(derive_worktree_port(h), derive_worktree_port(h));
    }

    #[test]
    fn handle_is_alive_reflects_pid_state() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("port"), "9315").unwrap();
        std::fs::write(tmp.path().join("token"), "x").unwrap();

        // Stale handle (no pid file) → not alive.
        let handle = DaemonHandle::load(tmp.path()).unwrap();
        assert!(!handle.is_alive());

        // Use our own pid — it must be alive (we're running this code).
        let my_pid = std::process::id();
        std::fs::write(tmp.path().join("pid"), my_pid.to_string()).unwrap();
        let handle = DaemonHandle::load(tmp.path()).unwrap();
        assert!(handle.is_alive());
    }
}
