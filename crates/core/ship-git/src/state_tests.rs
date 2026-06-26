//! Unit tests for GitService state/read/write modules.
//!
//! All write tests use `git2::Repository::init` in tempdirs — no network,
//! no shared state. Draft tests use the platform DB via `init_project`.

use super::{read, write};
use git2::{Repository, Signature};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

// ── fixtures ─────────────────────────────────────────────────────────────

struct Repo {
    _tmp: TempDir,
    root: PathBuf,
}

fn init_repo() -> Repo {
    let tmp = tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    Repository::init(&root).unwrap();
    // Set a committer for `signature()` calls.
    let repo = Repository::open(&root).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Tester").unwrap();
    config.set_str("user.email", "t@example.com").unwrap();
    Repo { _tmp: tmp, root }
}

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

/// Stage + commit all pending changes. Returns the commit sha.
fn initial_commit(root: &Path) -> String {
    let repo = Repository::open(root).unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::now("Tester", "t@example.com").unwrap();
    let oid = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    oid.to_string()
}

// ── status / branches / log ──────────────────────────────────────────────

#[test]
fn status_on_empty_repo_is_clean() {
    let r = init_repo();
    let b = read::load_status_buckets(&r.root);
    assert!(b.clean);
    assert_eq!(b.total, 0);
}

#[test]
fn status_picks_up_untracked_file() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "hello");
    let b = read::load_status_buckets(&r.root);
    assert!(!b.clean);
    assert_eq!(b.untracked.len(), 1);
    assert_eq!(b.untracked[0].path, "a.txt");
}

#[test]
fn status_reports_staged_vs_modified() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    // Modify and leave unstaged.
    write_file(&r.root, "a.txt", "v2");
    let b = read::load_status_buckets(&r.root);
    assert_eq!(b.modified.len(), 1);
    assert_eq!(b.modified[0].status, "M");

    // Stage it.
    write::stage_paths(&r.root, &["a.txt".into()]).unwrap();
    let b = read::load_status_buckets(&r.root);
    assert_eq!(b.staged.len(), 1);
    assert_eq!(b.modified.len(), 0);
}

#[test]
fn log_returns_commit_summaries() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    let sha = initial_commit(&r.root);
    let commits = read::load_log(&r.root, None, 10);
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].sha, sha);
    assert_eq!(commits[0].subject, "initial");
}

#[test]
fn log_respects_limit() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    write_file(&r.root, "b.txt", "v2");
    // Second commit.
    let repo = Repository::open(&r.root).unwrap();
    let mut idx = repo.index().unwrap();
    idx.add_path(Path::new("b.txt")).unwrap();
    idx.write().unwrap();
    let tree = repo.find_tree(idx.write_tree().unwrap()).unwrap();
    let sig = Signature::now("T", "t@e").unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "second", &tree, &[&head]).unwrap();

    let one = read::load_log(&r.root, None, 1);
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].subject, "second");
}

#[test]
fn branches_lists_local_branches() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    let repo = Repository::open(&r.root).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("feature/x", &head, false).unwrap();
    let bs = read::load_branches(&r.root);
    let names: Vec<&str> = bs.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"feature/x"), "branches: {names:?}");
}

// ── stage / unstage ──────────────────────────────────────────────────────

#[test]
fn stage_adds_untracked_file() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "hi");
    let n = write::stage_paths(&r.root, &["a.txt".into()]).unwrap();
    assert_eq!(n, 1);
    let b = read::load_status_buckets(&r.root);
    assert_eq!(b.staged.len(), 1);
    assert_eq!(b.staged[0].status, "A");
}

#[test]
fn unstage_reverts_staged_entry_to_head() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    write_file(&r.root, "a.txt", "v2");
    write::stage_paths(&r.root, &["a.txt".into()]).unwrap();
    assert_eq!(read::load_status_buckets(&r.root).staged.len(), 1);
    write::unstage_paths(&r.root, &["a.txt".into()]).unwrap();
    let b = read::load_status_buckets(&r.root);
    assert_eq!(b.staged.len(), 0);
    assert_eq!(b.modified.len(), 1, "still modified vs HEAD");
}

#[test]
fn stage_handles_deleted_file() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    fs::remove_file(r.root.join("a.txt")).unwrap();
    write::stage_paths(&r.root, &["a.txt".into()]).unwrap();
    let b = read::load_status_buckets(&r.root);
    assert_eq!(b.staged.len(), 1);
    assert_eq!(b.staged[0].status, "D");
}

// ── commit ───────────────────────────────────────────────────────────────

#[test]
fn commit_refuses_when_nothing_staged() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    let err = write::commit_staged(&r.root, "no-op", None).unwrap_err();
    assert!(matches!(err, super::state::GitError::NothingStaged));
}

#[test]
fn commit_creates_new_head() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    write_file(&r.root, "b.txt", "hi");
    write::stage_paths(&r.root, &["b.txt".into()]).unwrap();
    let res = write::commit_staged(&r.root, "add b", None).unwrap();
    assert_eq!(res.sha.len(), 40);
    let commits = read::load_log(&r.root, None, 10);
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].subject, "add b");
}

#[test]
fn commit_accepts_explicit_author() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    write::stage_paths(&r.root, &["a.txt".into()]).unwrap();
    let res = write::commit_staged(
        &r.root,
        "initial",
        Some("Alice <a@x.com>"),
    )
    .unwrap();
    assert_eq!(res.sha.len(), 40);
    let commits = read::load_log(&r.root, None, 1);
    assert_eq!(commits[0].author, "Alice");
}

#[test]
fn commit_rejects_bad_author_string() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    write::stage_paths(&r.root, &["a.txt".into()]).unwrap();
    let err = write::commit_staged(&r.root, "x", Some("no-email")).unwrap_err();
    assert!(matches!(err, super::state::GitError::Git(_)));
}

// ── squash-merge (land) ────────────────────────────────────────────────────

/// Commit the currently-staged tree onto HEAD with `message`. Returns the sha.
fn commit_on_head(root: &Path, message: &str) -> String {
    let repo = Repository::open(root).unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let sig = Signature::now("Tester", "t@example.com").unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    let oid = repo
        .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
        .unwrap();
    oid.to_string()
}

/// Create a branch at HEAD and check it out.
fn branch_and_checkout(root: &Path, name: &str) {
    write::branch_create(root, name).unwrap();
    write::checkout(root, name).unwrap();
}

#[test]
fn squash_merge_clean_lands_one_commit() {
    let r = init_repo();
    // main: a.txt
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let main = read::load_branch(&r.root).unwrap(); // "main" or "master"

    // job/x branches off main, adds b.txt across TWO commits.
    branch_and_checkout(&r.root, "job/x");
    write_file(&r.root, "b.txt", "one\n");
    commit_on_head(&r.root, "add b (1)");
    write_file(&r.root, "b.txt", "one\ntwo\n");
    commit_on_head(&r.root, "extend b (2)");

    // Back on main, land job/x.
    write::checkout(&r.root, &main).unwrap();
    let before = read::load_log(&r.root, None, 50).len();

    let out = write::squash_merge(&r.root, "job/x", &main, "land job/x", None).unwrap();
    let sha = match out {
        write::SquashOutcome::Committed { sha, branch } => {
            assert_eq!(branch, main);
            sha
        }
        write::SquashOutcome::Conflicts { paths } => panic!("unexpected conflicts: {paths:?}"),
    };
    assert_eq!(sha.len(), 40);

    // Exactly ONE new commit on main, and it's a single-parent squash (not a
    // merge commit) carrying ALL of job/x's changes.
    let log = read::load_log(&r.root, None, 50);
    assert_eq!(log.len(), before + 1, "exactly one squashed commit");
    assert_eq!(log[0].subject, "land job/x");

    let repo = Repository::open(&r.root).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 1, "squash = single parent, not a merge");

    // Working tree + index reflect the landed content (target was checked out).
    assert_eq!(
        fs::read_to_string(r.root.join("b.txt")).unwrap(),
        "one\ntwo\n"
    );
    assert!(read::load_status_buckets(&r.root).clean, "tree clean after land");
}

#[test]
fn squash_merge_conflict_returns_paths_and_writes_nothing() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let main = read::load_branch(&r.root).unwrap();

    // job/x edits a.txt one way.
    branch_and_checkout(&r.root, "job/x");
    write_file(&r.root, "a.txt", "from-job\n");
    commit_on_head(&r.root, "job edits a");

    // main edits the SAME line a different way → conflict.
    write::checkout(&r.root, &main).unwrap();
    write_file(&r.root, "a.txt", "from-main\n");
    let main_tip = commit_on_head(&r.root, "main edits a");
    let log_before = read::load_log(&r.root, None, 50).len();

    let out = write::squash_merge(&r.root, "job/x", &main, "land job/x", None).unwrap();
    match out {
        write::SquashOutcome::Conflicts { paths } => {
            assert_eq!(paths, vec!["a.txt".to_string()]);
        }
        write::SquashOutcome::Committed { sha, .. } => {
            panic!("expected conflict, got commit {sha}")
        }
    }

    // NOTHING written: main tip unchanged, no new commit, working tree intact.
    let repo = Repository::open(&r.root).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.id().to_string(), main_tip, "main tip unmoved");
    assert_eq!(read::load_log(&r.root, None, 50).len(), log_before, "no new commit");
    assert_eq!(fs::read_to_string(r.root.join("a.txt")).unwrap(), "from-main\n");
    assert!(read::load_status_buckets(&r.root).clean, "tree untouched");
}

#[test]
fn squash_merge_already_landed_is_nothing_staged() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let main = read::load_branch(&r.root).unwrap();

    // job/x has no commits beyond main → already an ancestor.
    write::branch_create(&r.root, "job/x").unwrap();
    let err = write::squash_merge(&r.root, "job/x", &main, "noop", None).unwrap_err();
    assert!(matches!(err, super::state::GitError::NothingStaged));
}

#[test]
fn squash_merge_unknown_source_errors() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let main = read::load_branch(&r.root).unwrap();
    let err = write::squash_merge(&r.root, "job/missing", &main, "x", None).unwrap_err();
    assert!(matches!(err, super::state::GitError::BadRef(_)));
}

// ── diff ─────────────────────────────────────────────────────────────────

#[test]
fn working_diff_shows_modified_content() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1\n");
    initial_commit(&r.root);
    write_file(&r.root, "a.txt", "v2\n");
    let text = read::load_working_diff(&r.root);
    assert!(text.contains("-v1"), "working diff text: {text}");
    assert!(text.contains("+v2"));
}

#[test]
fn commit_diff_matches_commit_sha() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1\n");
    let sha = initial_commit(&r.root);
    let text = read::load_commit_diff(&r.root, &sha);
    assert!(text.contains("+v1"), "commit diff: {text}");
    assert!(read::load_commit_diff(&r.root, "not-a-sha").is_empty());
}

// ── checkout ─────────────────────────────────────────────────────────────

#[test]
fn checkout_switches_local_branch() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    let repo = Repository::open(&r.root).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("feature", &head, false).unwrap();
    write::checkout(&r.root, "feature").unwrap();
    assert_eq!(read::load_branch(&r.root).as_deref(), Some("feature"));
}

#[test]
fn checkout_unknown_ref_errors() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    let err = write::checkout(&r.root, "does-not-exist").unwrap_err();
    assert!(matches!(err, super::state::GitError::BadRef(_)));
}

