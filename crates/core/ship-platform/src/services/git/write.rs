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
