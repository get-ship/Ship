//! GitService write operations — stage / unstage / commit / push / fetch /
//! pull (fast-forward-only) / checkout.
//!
//! libgit2 direct; no subprocess. Push/fetch/pull require network I/O;
//! authentication is delegated to git2's default credential helpers so the
//! same credentials the user's shell git uses also work here (ssh agent,
//! credential manager, etc.).

use git2::{
    AnnotatedCommit, AutotagOption, BranchType, Cred, FetchOptions, ObjectType,
    PushOptions, RemoteCallbacks, Repository, Signature,
};
use std::path::Path;

use super::state::{current_branch_name, open_repo, GitError};

// ── staging ──────────────────────────────────────────────────────────────

/// Stage `paths` relative to the repo root. Matches `git add`: for deleted
/// files, `index.add_path` fails; we fall through to `index.remove_path`.
pub fn stage_paths(root: &Path, paths: &[String]) -> Result<usize, GitError> {
    let repo = open_repo(root)?;
    let mut index = repo.index()?;
    let workdir = repo.workdir().ok_or_else(|| GitError::Git("bare repo".into()))?;
    let mut staged = 0usize;
    for p in paths {
        let rel = std::path::Path::new(p);
        let full = workdir.join(rel);
        if full.exists() {
            index.add_path(rel)?;
        } else {
            // File has been deleted — reflect that in the index.
            index.remove_path(rel)?;
        }
        staged += 1;
    }
    index.write()?;
    Ok(staged)
}

/// Unstage `paths` — reset the index entry for each path back to HEAD.
pub fn unstage_paths(root: &Path, paths: &[String]) -> Result<usize, GitError> {
    let repo = open_repo(root)?;
    let head_obj = match repo.head() {
        Ok(h) => Some(h.peel(ObjectType::Commit)?),
        Err(_) => None, // no HEAD yet — just clear the entries
    };

    if let Some(obj) = head_obj {
        let pathspecs: Vec<&str> = paths.iter().map(String::as_str).collect();
        repo.reset_default(Some(&obj), pathspecs.iter())?;
    } else {
        let mut index = repo.index()?;
        for p in paths {
            let _ = index.remove_path(std::path::Path::new(p));
        }
        index.write()?;
    }
    Ok(paths.len())
}

// ── commit ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CommitResult {
    pub sha: String,
    pub branch: Option<String>,
}

/// Commit staged changes. `author` is `"Name <email>"`; when `None` we pull
/// from git config. Refuses commit when nothing is staged.
pub fn commit_staged(
    root: &Path,
    message: &str,
    author: Option<&str>,
) -> Result<CommitResult, GitError> {
    let repo = open_repo(root)?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let parent = match repo.head() {
        Ok(h) => Some(h.peel_to_commit()?),
        Err(_) => None,
    };

    // Refuse if nothing staged vs the parent tree.
    if let Some(ref p) = parent {
        if p.tree_id() == tree_oid {
            return Err(GitError::NothingStaged);
        }
    } else if tree.is_empty() {
        return Err(GitError::NothingStaged);
    }

    let sig = build_signature(&repo, author)?;
    let parents: Vec<&git2::Commit> = parent.iter().collect();
    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(CommitResult {
        sha: oid.to_string(),
        branch: current_branch_name(&repo),
    })
}

fn build_signature<'a>(repo: &'a Repository, author: Option<&str>) -> Result<Signature<'a>, GitError> {
    if let Some(a) = author {
        let (name, email) = parse_author(a)
            .ok_or_else(|| GitError::Git(format!("bad author `{a}` — expected `Name <email>`")))?;
        return Signature::now(&name, &email).map_err(GitError::from);
    }
    repo.signature().map_err(|_| GitError::Git(
        "no author — set git config user.name + user.email, or pass `author` in the body".into()
    ))
}

fn parse_author(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    let open = s.rfind('<')?;
    let close = s.rfind('>')?;
    if close <= open { return None; }
    let name = s[..open].trim().to_string();
    let email = s[open + 1..close].trim().to_string();
    if name.is_empty() || email.is_empty() { return None; }
    Some((name, email))
}

// ── remote credentials ───────────────────────────────────────────────────

fn remote_callbacks<'a>() -> RemoteCallbacks<'a> {
    let mut cbs = RemoteCallbacks::new();
    cbs.credentials(|_url, username_from_url, allowed| {
        if allowed.contains(git2::CredentialType::SSH_KEY) {
            let user = username_from_url.unwrap_or("git");
            if let Ok(c) = Cred::ssh_key_from_agent(user) {
                return Ok(c);
            }
        }
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
            && let Ok(cfg) = git2::Config::open_default()
                && let Ok(c) = Cred::credential_helper(&cfg, _url, username_from_url) {
                    return Ok(c);
                }
        if allowed.contains(git2::CredentialType::DEFAULT) {
            return Cred::default();
        }
        Err(git2::Error::from_str("no supported credentials available"))
    });
    cbs
}

// ── push / fetch / pull ──────────────────────────────────────────────────

/// Push current branch to `remote` (default `origin`). Force-push when
/// `force=true`. Returns the remote name actually used.
pub fn push_current_branch(
    root: &Path,
    remote: Option<&str>,
    force: bool,
) -> Result<String, GitError> {
    let repo = open_repo(root)?;
    let branch = current_branch_name(&repo)
        .ok_or_else(|| GitError::BadRef("detached HEAD — nothing to push".into()))?;
    let remote_name = remote.unwrap_or("origin");
    let mut remote = repo
        .find_remote(remote_name)
        .map_err(|e| GitError::BadRef(format!("remote `{remote_name}`: {e}")))?;
    let prefix = if force { "+" } else { "" };
    let refspec = format!("{prefix}refs/heads/{branch}:refs/heads/{branch}");

    let mut opts = PushOptions::new();
    opts.remote_callbacks(remote_callbacks());
    remote.push(&[refspec.as_str()], Some(&mut opts))?;
    Ok(remote_name.to_string())
}

/// Fetch from `origin`. Updates `refs/remotes/origin/*` but performs no merge.
pub fn fetch(root: &Path, remote: Option<&str>) -> Result<String, GitError> {
    let repo = open_repo(root)?;
    let remote_name = remote.unwrap_or("origin");
    let mut remote = repo
        .find_remote(remote_name)
        .map_err(|e| GitError::BadRef(format!("remote `{remote_name}`: {e}")))?;

    let mut opts = FetchOptions::new();
    opts.remote_callbacks(remote_callbacks());
    opts.download_tags(AutotagOption::All);
    let refspecs: Vec<String> = remote
        .fetch_refspecs()?
        .iter()
        .filter_map(|s| s.map(String::from))
        .collect();
    remote.fetch(&refspecs, Some(&mut opts), None)?;
    Ok(remote_name.to_string())
}

/// Fast-forward-only pull. Returns new HEAD sha on success. On non-ff
/// returns `GitError::NonFastForward { ahead, behind }` — the handler
/// translates this to a 409.
pub fn pull_ff_only(root: &Path, remote: Option<&str>) -> Result<String, GitError> {
    let _ = fetch(root, remote)?;
    let repo = open_repo(root)?;
    let head = repo.head()?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| GitError::BadRef("detached HEAD".into()))?
        .to_string();
    let local_oid = head.target().ok_or_else(|| GitError::BadRef("no HEAD target".into()))?;
    let remote_name = remote.unwrap_or("origin");
    let up_ref = format!("refs/remotes/{remote_name}/{branch_name}");
    let up_oid = repo
        .refname_to_id(&up_ref)
        .map_err(|_| GitError::BadRef(format!("no upstream `{up_ref}`")))?;

    let (ahead, behind) = repo.graph_ahead_behind(local_oid, up_oid)?;
    if ahead > 0 {
        return Err(GitError::NonFastForward { ahead, behind });
    }
    if behind == 0 {
        return Ok(local_oid.to_string()); // already up-to-date
    }

    // Fast-forward the local branch ref to the upstream oid.
    let upstream_commit: AnnotatedCommit = repo.find_annotated_commit(up_oid)?;
    let mut branch_ref = repo.find_reference(&format!("refs/heads/{branch_name}"))?;
    branch_ref.set_target(upstream_commit.id(), "fast-forward pull")?;
    repo.set_head(&format!("refs/heads/{branch_name}"))?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default().force(),
    ))?;
    Ok(upstream_commit.id().to_string())
}

// ── branch create / delete ──────────────────────────────────────────────

/// Create a new local branch from HEAD. Fails if the branch already exists.
pub fn branch_create(root: &Path, name: &str) -> Result<String, GitError> {
    let repo = open_repo(root)?;
    let head_commit = repo
        .head()
        .map_err(|e| GitError::BadRef(format!("no HEAD: {e}")))?
        .peel_to_commit()?;
    repo.branch(name, &head_commit, false)
        .map_err(|e| GitError::BadRef(format!("branch `{name}`: {e}")))?;
    Ok(name.to_string())
}

/// Delete a local branch. `force=true` allows deleting unmerged branches.
/// Refuses to delete the currently-checked-out branch.
pub fn branch_delete(root: &Path, name: &str, force: bool) -> Result<String, GitError> {
    let repo = open_repo(root)?;
    if current_branch_name(&repo).as_deref() == Some(name) {
        return Err(GitError::BadRef(format!(
            "cannot delete `{name}` — it is the current branch"
        )));
    }
    let mut branch = repo
        .find_branch(name, BranchType::Local)
        .map_err(|e| GitError::BadRef(format!("branch `{name}`: {e}")))?;
    if !force {
        // Reject deletion if the branch is not merged into HEAD.
        let head_oid = repo.head()?.target();
        let br_oid = branch.get().target();
        if let (Some(head), Some(br)) = (head_oid, br_oid) {
            let (ahead, _) = repo.graph_ahead_behind(br, head)?;
            if ahead > 0 {
                return Err(GitError::BadRef(format!(
                    "branch `{name}` is not fully merged — pass force=true to override"
                )));
            }
        }
    }
    branch.delete()?;
    Ok(name.to_string())
}

// ── repo init ────────────────────────────────────────────────────────────

/// Initialize a new git repository at `root`. No-op if `.git` already exists.
pub fn init_repo(root: &Path) -> Result<(), GitError> {
    if root.join(".git").exists() {
        return Ok(());
    }
    Repository::init(root).map_err(GitError::from)?;
    Ok(())
}

// ── squash-merge (land) ────────────────────────────────────────────────────

/// Outcome of a [`squash_merge`]. A conflict is NOT an error — it's a typed
/// result the caller (the lander) handles by resolving and retrying. `GitError`
/// is reserved for genuine failures (bad ref, nothing to land, io).
#[derive(Debug, Clone)]
pub enum SquashOutcome {
    /// Clean squash-merge: one new commit on the target branch.
    Committed {
        /// The new (single-parent) squash commit sha on `target_branch`.
        sha: String,
        /// The branch that was advanced.
        branch: String,
    },
    /// The three-way merge conflicted. Nothing was written; these are the
    /// conflicting paths for the lander to resolve before retrying.
    Conflicts { paths: Vec<String> },
}

/// Squash-merge `source_branch` into `target_branch` as a SINGLE commit.
///
/// This is the `land` operation: all of `source_branch`'s changes (relative to
/// the merge base with `target_branch`) collapse into ONE new commit whose sole
/// parent is the current `target_branch` tip — no merge parent, no per-commit
/// history from the job branch. The branch ref is advanced to the new commit;
/// if `target_branch` is the checked-out branch, the index + working tree are
/// synced to match.
///
/// On a merge conflict we DO NOT auto-resolve and DO NOT write anything — we
/// return [`SquashOutcome::Conflicts`] with the conflicting paths. Conflict
/// resolution is the lander's job (see `ship-ops/src/workflow/model.ts`'s
/// `land()` returning `{ok:false, conflicts}`).
///
/// `author` is `"Name <email>"`; `None` pulls from git config.
pub fn squash_merge(
    root: &Path,
    source_branch: &str,
    target_branch: &str,
    message: &str,
    author: Option<&str>,
) -> Result<SquashOutcome, GitError> {
    let repo = open_repo(root)?;

    let source_commit = repo
        .find_branch(source_branch, BranchType::Local)
        .map_err(|e| GitError::BadRef(format!("source branch `{source_branch}`: {e}")))?
        .get()
        .peel_to_commit()?;
    let target_commit = repo
        .find_branch(target_branch, BranchType::Local)
        .map_err(|e| GitError::BadRef(format!("target branch `{target_branch}`: {e}")))?
        .get()
        .peel_to_commit()?;

    // Merge base for the three-way merge. If source is already an ancestor of
    // target, it has already landed — nothing to do.
    let base_oid = repo
        .merge_base(target_commit.id(), source_commit.id())
        .map_err(|e| GitError::Git(format!("no merge base for `{source_branch}`/`{target_branch}`: {e}")))?;
    if base_oid == source_commit.id() {
        return Err(GitError::NothingStaged); // source already merged into target
    }
    let base_commit = repo.find_commit(base_oid)?;

    // Three-way tree merge: ancestor=base, ours=target, theirs=source. The
    // resulting index lives in memory only (it is never written to the repo
    // index), so a conflict leaves the repo completely untouched.
    let mut merged = repo.merge_trees(
        &base_commit.tree()?,
        &target_commit.tree()?,
        &source_commit.tree()?,
        None,
    )?;

    if merged.has_conflicts() {
        let mut paths = std::collections::BTreeSet::new();
        for c in merged.conflicts()? {
            let c = c?;
            if let Some(entry) = c.our.or(c.their).or(c.ancestor) {
                paths.insert(String::from_utf8_lossy(&entry.path).into_owned());
            }
        }
        return Ok(SquashOutcome::Conflicts {
            paths: paths.into_iter().collect(),
        });
    }

    let tree_oid = merged.write_tree_to(&repo)?;
    if tree_oid == target_commit.tree_id() {
        // The merge produced no change against target — nothing to land.
        return Err(GitError::NothingStaged);
    }
    let tree = repo.find_tree(tree_oid)?;

    let sig = build_signature(&repo, author)?;
    // Squash = ONE commit, sole parent = target tip (no merge parent). We pass
    // `None` for the ref so we can advance the branch explicitly below.
    let oid = repo.commit(None, &sig, &sig, message, &tree, &[&target_commit])?;

    // Advance the target branch ref to the squash commit.
    repo.reference(
        &format!("refs/heads/{target_branch}"),
        oid,
        true,
        &format!("squash-merge {source_branch} -> {target_branch}"),
    )?;

    // If target is the checked-out branch, sync index + working tree.
    if current_branch_name(&repo).as_deref() == Some(target_branch) {
        repo.checkout_tree(
            tree.as_object(),
            Some(git2::build::CheckoutBuilder::default().force()),
        )?;
        repo.set_head(&format!("refs/heads/{target_branch}"))?;
    }

    Ok(SquashOutcome::Committed {
        sha: oid.to_string(),
        branch: target_branch.to_string(),
    })
}

// ── tag (release) ────────────────────────────────────────────────────────

/// Create an ANNOTATED git tag named `name` at `target` and return the tag name.
///
/// This is the `release` tag step: after [`squash_merge`] consolidates a
/// mission's integration branch onto `main`, we stamp the result with the
/// version tag. `target` is a ref name or sha; `None`/empty tags the current
/// HEAD (the just-written release commit). The tag is annotated (carries the
/// `message` + tagger), not lightweight.
///
/// Refuses to clobber an existing tag (force=false) — a re-release with the same
/// version is a [`GitError::BadRef`], not a silent move. `author` is the tagger
/// `"Name <email>"`; `None` pulls from git config.
pub fn create_tag(
    root: &Path,
    name: &str,
    target: Option<&str>,
    message: &str,
    author: Option<&str>,
) -> Result<String, GitError> {
    let repo = open_repo(root)?;
    // Resolve the object to tag: an explicit ref/sha, else current HEAD.
    let obj = match target.filter(|t| !t.is_empty()) {
        Some(t) => repo
            .revparse_single(t)
            .map_err(|e| GitError::BadRef(format!("tag target `{t}`: {e}")))?,
        None => repo
            .head()
            .map_err(|e| GitError::BadRef(format!("no HEAD to tag: {e}")))?
            .peel(ObjectType::Commit)?,
    };
    let sig = build_signature(&repo, author)?;
    repo.tag(name, &obj, &sig, message, false)
        .map_err(|e| GitError::BadRef(format!("tag `{name}`: {e}")))?;
    Ok(name.to_string())
}

// ── checkout ─────────────────────────────────────────────────────────────

/// Checkout a ref — branch name or commit sha. Fails fast on dirty working
/// tree (libgit2's default checkout behaviour).
pub fn checkout(root: &Path, refname: &str) -> Result<String, GitError> {
    let repo = open_repo(root)?;
    // Prefer local branch, else tag/commit.
    if let Ok(branch) = repo.find_branch(refname, BranchType::Local) {
        let branch_ref = branch.get();
        let name = branch_ref
            .name()
            .ok_or_else(|| GitError::BadRef("branch has no name".into()))?
            .to_string();
        let obj = branch_ref.peel(ObjectType::Commit)?;
        repo.checkout_tree(&obj, None)?;
        repo.set_head(&name)?;
        return Ok(refname.to_string());
    }
    // Try as sha / tag.
    let obj = repo
        .revparse_single(refname)
        .map_err(|e| GitError::BadRef(format!("no such ref `{refname}`: {e}")))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head_detached(obj.id())?;
    Ok(obj.id().to_string())
}

// ── worktree (create) ─────────────────────────────────────────────────────
//
// The single subprocess exception in the leaf: `git worktree add -b <branch>
// <path> <base>` is implemented via `std::process::Command`, not libgit2.
// libgit2 0.19's `Repository::worktree()` requires `GIT_DIR`/`GIT_COMMON_DIR`
// in the environment and several bookkeeping steps (`.git/worktrees/<name>`,
// per-worktree HEAD, etc.) that `git worktree add` handles for us; the
// subprocess path matches what `dispatch.sh` runs in shell, so semantics are
// identical (including the worktree's HEAD ref, .git pointer file, and
// gitdir entry). Marked explicitly so the audit knows the one place that
// breaks the libgit2-direct invariant.

/// Outcome of [`create_worktree`]. The leaf returns a typed result, not a
/// `Result<_, GitError>`, because a `Reused` is NOT an error — it's a normal
/// idempotent hit the caller (the dispatch workflow) records as success.
/// `Failed` carries the failure reason; the caller translates to a 5xx.
#[derive(Debug, Clone)]
pub enum WorktreeOutcome {
    /// New worktree + branch were created.
    Created {
        branch: String,
        worktree_path: std::path::PathBuf,
        base_ref: String,
    },
    /// Idempotent hit: the worktree already exists on `branch`; nothing was
    /// written. Mirrors `src/dispatch.ts:175-181`'s "worktree already on
    /// `job/<slug>`" reuse check.
    Reused {
        branch: String,
        worktree_path: std::path::PathBuf,
    },
    /// The write was refused. `error` is a human-readable reason; `branch`
    /// and `worktree_path` echo back the request fields for logging.
    Failed {
        branch: String,
        worktree_path: std::path::PathBuf,
        error: String,
    },
}

/// Create a new git worktree at `worktree_path` on a new `branch` starting
/// from `base_ref`. Mirrors the operator's `git worktree add -b <branch>
/// <path> <base>` invocation (see `src/dispatch.ts:183`, audit row #10) and
/// is idempotent when the worktree already exists on `branch`.
///
/// Behavior:
/// - `worktree_path` is empty / doesn't exist → CREATED. Runs
///   `git -C <root> worktree add -b <branch> <worktree_path> <base_ref>`
///   via subprocess (see module-level note). Empty `base_ref` falls back to
///   HEAD (matching `git worktree add -b <branch> <path>` without a base).
/// - `worktree_path` exists AND its HEAD is on `branch` → REUSED (idempotent
///   success; the dispatch caller records "already there" and moves on).
/// - `worktree_path` exists AND its HEAD is on a DIFFERENT branch → FAILED
///   with an `error` like "worktree at <path> is on branch `<other>`, not
///   `<branch>`". The dispatch caller translates to a 5xx so the operator
///   resolves the conflict (remove the worktree, pick a different slug, …).
/// - A `git worktree add` failure (bad ref, branch already exists, etc.)
///   bubbles up as FAILED with the underlying stderr in `error`.
///
/// The worktree's HEAD branch is read with `git -C <wt> rev-parse
/// --abbrev-ref HEAD` (the same call helm's `src/dispatch.ts:176-181` uses
/// for the reuse check) — also a subprocess, scoped to the existing path
/// only on the idempotent path.
pub fn create_worktree(
    root: &Path,
    worktree_path: &Path,
    branch: &str,
    base_ref: &str,
    idempotent: bool,
) -> WorktreeOutcome {
    if branch.is_empty() {
        return WorktreeOutcome::Failed {
            branch: branch.to_string(),
            worktree_path: worktree_path.to_path_buf(),
            error: "branch name is required".to_string(),
        };
    }

    // ── Idempotent hit: path exists and HEAD branch matches ──────────────
    if worktree_path.exists() {
        let existing_branch = existing_worktree_branch(worktree_path);
        if let Some(existing) = existing_branch {
            if existing == branch {
                return WorktreeOutcome::Reused {
                    branch: branch.to_string(),
                    worktree_path: worktree_path.to_path_buf(),
                };
            }
            // Path exists on a different branch — refuse.
            let msg = format!(
                "worktree at `{}` is on branch `{}`, not `{}`",
                worktree_path.display(),
                existing,
                branch
            );
            return WorktreeOutcome::Failed {
                branch: branch.to_string(),
                worktree_path: worktree_path.to_path_buf(),
                error: msg,
            };
        }
        // Path exists but is not a git worktree (or HEAD isn't readable).
        // On idempotent=true we still refuse — the caller picked a path
        // that's occupied by something else.
        if idempotent {
            return WorktreeOutcome::Failed {
                branch: branch.to_string(),
                worktree_path: worktree_path.to_path_buf(),
                error: format!(
                    "path `{}` exists but is not a git worktree",
                    worktree_path.display()
                ),
            };
        }
    }

    // ── Create: `git worktree add -b <branch> <path> <base_ref?>` ─────────
    let mut args: Vec<String> = vec![
        "-C".into(),
        root.display().to_string(),
        "worktree".into(),
        "add".into(),
        "-b".into(),
        branch.to_string(),
    ];
    args.push(worktree_path.display().to_string());
    if !base_ref.is_empty() {
        args.push(base_ref.to_string());
    }
    let output = std::process::Command::new("git").args(&args).output();
    match output {
        Ok(out) if out.status.success() => WorktreeOutcome::Created {
            branch: branch.to_string(),
            worktree_path: worktree_path.to_path_buf(),
            base_ref: base_ref.to_string(),
        },
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let detail = if !stderr.is_empty() { stderr } else { stdout };
            let detail = if detail.is_empty() { "git worktree add failed".to_string() } else { detail };
            WorktreeOutcome::Failed {
                branch: branch.to_string(),
                worktree_path: worktree_path.to_path_buf(),
                error: detail,
            }
        }
        Err(e) => WorktreeOutcome::Failed {
            branch: branch.to_string(),
            worktree_path: worktree_path.to_path_buf(),
            error: format!("failed to spawn `git worktree add`: {e}"),
        },
    }
}

/// Read the HEAD branch of an existing worktree at `worktree_path` via
/// `git -C <path> rev-parse --abbrev-ref HEAD`. Returns `None` on any
/// failure (path not a worktree, no HEAD, etc.) — the caller treats
/// `None` as "not an idempotent hit" and falls through to create.
fn existing_worktree_branch(worktree_path: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args([
            "-C",
            &worktree_path.display().to_string(),
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() || s == "HEAD" {
        // Detached HEAD — not a match for any named branch.
        return None;
    }
    Some(s)
}


// ── branch delete (guarded) ───────────────────────────────────────────────
//
// The TEARDOWN gate's delete op: a thin wrapper around [`branch_delete`]
// that adds a job-branch prefix guard. The gate only cleans `job/*` branches
// (the workflow's job-branch namespace — matches `dispatch.ts`'s
// `job/<slug>` convention). `main` / `feat/*` / `feat/next` are protected:
// the operator must pass `force=true` to delete them. `force=true` is also
// required to delete an unmerged branch (delegated to the inner
// `branch_delete`).
//
// Returns `Ok(name)` on success. On a guard failure, returns a typed
// `GitError::BadRef` with a human-readable reason — the handler maps it to
// a 400 with `error` populated, mirroring the rest of the leaf's
// typed-error surface.
pub fn delete_branch(root: &Path, name: &str, force: bool) -> Result<String, GitError> {
    if name.is_empty() {
        return Err(GitError::BadRef("branch name is required".to_string()));
    }
    // Guard: refuse non-`job/*` branches unless the operator passes
    // `force=true`. The prefix check is `name.starts_with("job/")` — bare
    // `job` (no slash) is NOT a job branch.
    if !force && !name.starts_with("job/") {
        return Err(GitError::BadRef(format!(
            "refusing to delete `{name}` — not a `job/*` branch (pass force=true to override)"
        )));
    }
    branch_delete(root, name, force)
}

// ── worktree remove ───────────────────────────────────────────────────────
//
// The TEARDOWN gate's worktree cleanup op: `git worktree remove [--force]`
// for a worktree path. Subprocess-backed for the same reason as
// [`create_worktree`] — libgit2 0.19's worktree-remove surface is fragile
// (it requires the worktree's HEAD to be clean + several bookkeeping steps
// that `git worktree remove` handles). Marked explicitly as the second
// subprocess exception in the leaf.
//
// `force` is a pass-through to `git worktree remove --force` for worktrees
// with local modifications. The leaf does NOT prune the per-worktree
// administrative dir (`.git/worktrees/<name>`) — `git worktree remove`
// does that for us; the caller does not need to clean up.
pub fn remove_worktree(root: &Path, worktree_path: &Path, force: bool) -> Result<(), GitError> {
    if worktree_path.as_os_str().is_empty() {
        return Err(GitError::BadRef("worktree_path is required".to_string()));
    }
    let mut args: Vec<String> = vec![
        "-C".into(),
        root.display().to_string(),
        "worktree".into(),
        "remove".into(),
    ];
    if force {
        args.push("--force".into());
    }
    args.push(worktree_path.display().to_string());
    let output = std::process::Command::new("git").args(&args).output();
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let detail = if !stderr.is_empty() { stderr } else { stdout };
            let detail = if detail.is_empty() {
                "git worktree remove failed".to_string()
            } else {
                detail
            };
            Err(GitError::Git(detail))
        }
        Err(e) => Err(GitError::Git(format!("failed to spawn `git worktree remove`: {e}"))),
    }
}
