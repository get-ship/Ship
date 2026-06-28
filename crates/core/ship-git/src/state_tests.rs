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

// ── changed-files (presence footprint) ──────────────────────────────────────

#[test]
fn changed_files_lists_branch_footprint() {
    let r = init_repo();
    // base: a.txt
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let base = read::load_branch(&r.root).unwrap(); // "main" or "master"

    // job/x branches off base and touches b.txt + c.txt across two commits.
    branch_and_checkout(&r.root, "job/x");
    write_file(&r.root, "b.txt", "one\n");
    commit_on_head(&r.root, "add b");
    write_file(&r.root, "c.txt", "two\n");
    commit_on_head(&r.root, "add c");

    // Footprint = the files job/x changed since the merge base, sorted + deduped.
    let files = read::changed_files(&r.root, &base, "job/x").unwrap();
    assert_eq!(files, vec!["b.txt".to_string(), "c.txt".to_string()]);

    // a.txt was untouched by the branch — it is NOT in the footprint.
    assert!(!files.contains(&"a.txt".to_string()));
}

#[test]
fn changed_files_empty_when_branch_at_base() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let base = read::load_branch(&r.root).unwrap();

    // job/x has no commits beyond base → footprint is empty (nothing touched).
    write::branch_create(&r.root, "job/x").unwrap();
    let files = read::changed_files(&r.root, &base, "job/x").unwrap();
    assert!(files.is_empty(), "expected empty footprint, got {files:?}");
}

#[test]
fn changed_files_bad_ref_errors() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    let base = read::load_branch(&r.root).unwrap();

    // An unknown branch ref is a typed BadRef, not a silent empty list.
    let err = read::changed_files(&r.root, &base, "job/missing").unwrap_err();
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

// ── structured diff (the git modal's render-ready tree) ─────────────────────

#[test]
fn structured_working_diff_shapes_rows() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1\n");
    initial_commit(&r.root);
    write_file(&r.root, "a.txt", "v2\n");

    // No ref → the working tree vs HEAD (the uncommitted change).
    let view = read::structured_diff(&r.root, None, None).unwrap();
    assert_eq!(view.files.len(), 1);
    let f = &view.files[0];
    assert_eq!(f.path, "a.txt");
    assert_eq!(f.old_path, "a.txt");
    assert_eq!(f.status, "modified");
    assert_eq!(f.additions, 1);
    assert_eq!(f.deletions, 1);
    assert!(!f.binary);
    assert_eq!(view.additions, 1);
    assert_eq!(view.deletions, 1);
    assert!(!view.truncated);

    // First row is the hunk header; then a del row (v1) and an add row (v2).
    assert!(f.rows[0].hunk, "first row is a hunk header");
    assert!(f.rows[0].header.starts_with("@@"));
    let del = f.rows.iter().find(|row| row.kind == "del").expect("a del row");
    assert_eq!(del.text, "v1");
    assert_eq!(del.new_line, "", "a del has no new-side number");
    let add = f.rows.iter().find(|row| row.kind == "add").expect("an add row");
    assert_eq!(add.text, "v2");
    assert_eq!(add.old_line, "", "an add has no old-side number");
    // Keys are unique across the diff (stable {#for} keys).
    let keys: std::collections::BTreeSet<&str> = f.rows.iter().map(|row| row.key.as_str()).collect();
    assert_eq!(keys.len(), f.rows.len(), "row keys are unique");
}

#[test]
fn structured_diff_commit_ref_shows_own_change() {
    let r = init_repo();
    // First commit: a.txt only.
    write_file(&r.root, "a.txt", "base\n");
    initial_commit(&r.root);
    // Second commit: adds b.txt — that commit's OWN change is just b.txt.
    write_file(&r.root, "b.txt", "hello\n");
    let sha = commit_on_head(&r.root, "add b");

    let view = read::structured_diff(&r.root, None, Some(&sha)).unwrap();
    assert_eq!(view.files.len(), 1, "the commit's own change is one file");
    let f = &view.files[0];
    assert_eq!(f.path, "b.txt");
    assert_eq!(f.status, "added");
    assert_eq!(f.additions, 1);
    assert_eq!(f.deletions, 0);
    assert!(f.rows.iter().any(|row| row.kind == "add" && row.text == "hello"));
}

#[test]
fn structured_diff_clean_tree_is_empty() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1\n");
    initial_commit(&r.root);

    let view = read::structured_diff(&r.root, None, None).unwrap();
    assert!(view.files.is_empty(), "clean tree has no changed files");
    assert_eq!(view.additions, 0);
    assert_eq!(view.deletions, 0);
    assert!(!view.truncated);
}

#[test]
fn structured_diff_scopes_to_pathspec() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "a1\n");
    write_file(&r.root, "b.txt", "b1\n");
    initial_commit(&r.root);
    write_file(&r.root, "a.txt", "a2\n");
    write_file(&r.root, "b.txt", "b2\n");

    // Whole-repo diff sees both files; a pathspec scopes to one.
    let all = read::structured_diff(&r.root, None, None).unwrap();
    assert_eq!(all.files.len(), 2);
    let scoped = read::structured_diff(&r.root, Some("b.txt"), None).unwrap();
    assert_eq!(scoped.files.len(), 1);
    assert_eq!(scoped.files[0].path, "b.txt");
}

#[test]
fn structured_diff_truncates_past_the_cap() {
    let r = init_repo();
    // Commit an empty file, then fill it past the row cap so the working diff is
    // all additions: one hunk-header row + N add rows > MAX_DIFF_ROWS.
    write_file(&r.root, "big.txt", "");
    initial_commit(&r.root);
    let n = read::MAX_DIFF_ROWS + 100;
    let body: String = (0..n).map(|i| format!("line {i}\n")).collect();
    write_file(&r.root, "big.txt", &body);

    let view = read::structured_diff(&r.root, None, None).unwrap();
    assert!(view.truncated, "a diff past the cap is truncated");
    let rows: usize = view.files.iter().map(|f| f.rows.len()).sum();
    assert_eq!(rows, read::MAX_DIFF_ROWS, "rows are capped at MAX_DIFF_ROWS");
    // Counts still reflect the FULL diff even though rows were dropped.
    assert_eq!(view.additions as usize, n, "counts reflect the full diff");
}

#[test]
fn structured_diff_bad_ref_errors() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1\n");
    initial_commit(&r.root);
    let err = read::structured_diff(&r.root, None, Some("not-a-real-ref")).unwrap_err();
    assert!(matches!(err, super::state::GitError::BadRef(_)));
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

// ── tag (release) ──────────────────────────────────────────────────────────

#[test]
fn create_tag_annotates_head() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    let sha = initial_commit(&r.root);

    let name = write::create_tag(&r.root, "v0.0.1", None, "release v0.0.1", None).unwrap();
    assert_eq!(name, "v0.0.1");

    let repo = Repository::open(&r.root).unwrap();
    // The annotated tag object carries the message and points at HEAD.
    let obj = repo.revparse_single("v0.0.1").unwrap();
    let tag = obj.as_tag().expect("annotated tag object");
    assert_eq!(tag.message().unwrap().trim(), "release v0.0.1");
    assert_eq!(tag.target().unwrap().id().to_string(), sha);

    // It is discoverable as a tag ref.
    let names = repo.tag_names(None).unwrap();
    assert!(names.iter().flatten().any(|n| n == "v0.0.1"), "tag listed");
}

#[test]
fn create_tag_at_explicit_sha() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    let first = initial_commit(&r.root);
    // A second commit advances HEAD past `first`.
    write_file(&r.root, "b.txt", "v2");
    write::stage_paths(&r.root, &["b.txt".into()]).unwrap();
    write::commit_staged(&r.root, "second", None).unwrap();

    // Tag the FIRST commit explicitly, not HEAD.
    write::create_tag(&r.root, "v0.0.1", Some(&first), "tag first", None).unwrap();
    let repo = Repository::open(&r.root).unwrap();
    let tag = repo.revparse_single("v0.0.1").unwrap();
    assert_eq!(
        tag.as_tag().unwrap().target().unwrap().id().to_string(),
        first,
        "tag points at the explicit sha, not HEAD"
    );
}

#[test]
fn create_tag_rejects_duplicate() {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    write::create_tag(&r.root, "v0.0.1", None, "first", None).unwrap();
    // A re-release with the same version is rejected (no silent clobber).
    let err = write::create_tag(&r.root, "v0.0.1", None, "again", None).unwrap_err();
    assert!(matches!(err, super::state::GitError::BadRef(_)));
}


// ── RepoStatus (consolidated Repos-pane read) ──────────────────────────────
//
// These tests pin the shape of `read::repo_status` against a temp repo —
// the same loader helmd's `GitService.RepoStatus` handler delegates to.
// They cover the verdict bits the proto response surfaces
// (`is_integration`, `is_release_target`, `has_worktree`,
// `merged_into_integration`, ahead/behind).

/// Build a repo with `main` (with one commit), `feat/integration` (one
/// ahead commit on top of `main`), and `job/foo` (one ahead commit on top
/// of `feat/integration`). The current HEAD is `feat/integration`.
fn init_repo_with_branches() -> (Repo, String, String, String) {
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    let _ = initial_commit(&r.root);

    let repo = Repository::open(&r.root).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();

    // Rename `master` (the git2 default) -> `main`.
    repo.branch("main", &head, true).unwrap();
    let main_commit = head.id().to_string();

    // `feat/integration`: one commit on top of main.
    repo.branch("feat/integration", &head, false).unwrap();
    repo.set_head("refs/heads/feat/integration").unwrap();
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default().force(),
    ))
    .unwrap();
    write_file(&r.root, "b.txt", "v2");
    {
        let mut idx = repo.index().unwrap();
        idx.add_path(Path::new("b.txt")).unwrap();
        idx.write().unwrap();
        let tree = repo.find_tree(idx.write_tree().unwrap()).unwrap();
        let sig = Signature::now("T", "t@e").unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "integration bump", &tree, &[&head])
            .unwrap();
    }
    let integration_commit = repo.head().unwrap().target().unwrap().to_string();

    // `job/foo`: one commit on top of integration.
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("job/foo", &head, false).unwrap();
    repo.set_head("refs/heads/job/foo").unwrap();
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default().force(),
    ))
    .unwrap();
    write_file(&r.root, "c.txt", "v3");
    {
        let mut idx = repo.index().unwrap();
        idx.add_path(Path::new("c.txt")).unwrap();
        idx.write().unwrap();
        let tree = repo.find_tree(idx.write_tree().unwrap()).unwrap();
        let sig = Signature::now("T", "t@e").unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "job work", &tree, &[&head])
            .unwrap();
    }

    // Park HEAD on the integration branch — that's the branch the Repos
    // pane treats as the "current" branch and what `repo_status` reports
    // as `branch` / `is_integration` / `is_current`.
    repo.set_head("refs/heads/feat/integration").unwrap();
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default().force(),
    ))
    .unwrap();

    (r, main_commit, integration_commit.clone(), integration_commit)
}

#[test]
fn repo_status_on_non_repo_yields_default() {
    // A path that isn't a git repo must return an empty/defaulted
    // RepoStatus — never error. helmd relies on this tolerance.
    let tmp = tempdir().unwrap();
    let status = read::repo_status(tmp.path());
    assert!(status.branch.is_empty());
    assert!(status.head_short.is_empty());
    assert_eq!(status.dirty_count, 0);
    assert!(status.clean);
    assert_eq!(status.worktree_count, 0);
    assert!(status.worktrees.is_empty());
    assert!(status.release_branch.is_empty());
    assert_eq!(status.release_ahead, 0);
    assert_eq!(status.release_behind, 0);
    assert!(status.branches.is_empty());
}

#[test]
fn repo_status_identity_and_status() {
    let (r, ..) = init_repo_with_branches();
    write_file(&r.root, "dirty.txt", "uncommitted");
    let status = read::repo_status(&r.root);

    // HEAD is `feat/integration` per the fixture.
    assert_eq!(status.branch, "feat/integration");
    assert!(!status.head_short.is_empty(), "head_short must be set");
    assert_eq!(status.head_short.len(), 7, "head_short is the 7-char sha");

    // One untracked file (`dirty.txt`).
    assert!(!status.clean);
    assert!(status.dirty_count >= 1, "dirty_count includes the untracked file");
}

#[test]
fn repo_status_resolves_release_target() {
    let (r, ..) = init_repo_with_branches();
    // Fixture has `main`; `repo_status` should resolve it as the release target.
    let status = read::repo_status(&r.root);
    assert_eq!(status.release_branch, "main");
    // integration (feat/integration) is 1 ahead of main.
    assert_eq!(status.release_ahead, 1);
    assert_eq!(status.release_behind, 0);
}

#[test]
fn repo_status_falls_back_to_master_when_main_missing() {
    // Repo with only `master` (no `main`): release_branch should fall back.
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    initial_commit(&r.root);
    // git2's default branch name may be `master` already; assert the fixture
    // has either `main` or `master` and the helper picks whichever exists.
    let status = read::repo_status(&r.root);
    assert!(
        status.release_branch == "main" || status.release_branch == "master",
        "expected main|master, got {:?}",
        status.release_branch
    );
    // No release branch mismatch — integration == release_branch is 0/0.
    assert_eq!(status.release_ahead, 0);
    assert_eq!(status.release_behind, 0);
}

#[test]
fn repo_status_branch_verdicts() {
    let (r, ..) = init_repo_with_branches();
    let status = read::repo_status(&r.root);

    // The branch list must contain all three fixture branches.
    let names: Vec<&str> = status.branches.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"main"), "branches: {names:?}");
    assert!(names.contains(&"feat/integration"), "branches: {names:?}");
    assert!(names.contains(&"job/foo"), "branches: {names:?}");

    // HEAD is `feat/integration` so:
    let integ = status
        .branches
        .iter()
        .find(|b| b.name == "feat/integration")
        .unwrap();
    assert!(integ.is_integration);
    // The main worktree always carries HEAD (== integration here), so
    // `has_worktree` is true for the integration branch.
    assert!(integ.has_worktree);

    let main = status.branches.iter().find(|b| b.name == "main").unwrap();
    assert!(main.is_release_target);
    assert!(!main.is_integration);
    assert!(!main.is_current);
    // main is the merge-base — `merged_into_integration` is true for main
    // because integration strictly contains it.
    assert!(main.merged_into_integration);

    let job = status.branches.iter().find(|b| b.name == "job/foo").unwrap();
    assert!(!job.is_integration);
    assert!(!job.merged_into_integration, "job/foo is AHEAD of integration");
    assert!(!job.is_current);
}

#[test]
fn repo_status_ahead_behind_per_branch() {
    let (r, ..) = init_repo_with_branches();
    let status = read::repo_status(&r.root);

    // job/foo is 1 ahead of integration (the only ahead-of-integration branch).
    let job = status.branches.iter().find(|b| b.name == "job/foo").unwrap();
    assert_eq!(job.behind, 1, "job/foo is 1 ahead of integration");
    assert_eq!(job.ahead, 0, "integration is 0 ahead of job/foo");

    // main is 1 BEHIND integration.
    let main = status.branches.iter().find(|b| b.name == "main").unwrap();
    assert_eq!(main.ahead, 1, "integration is 1 ahead of main");
    assert_eq!(main.behind, 0, "main is 0 ahead of integration");
}

#[test]
fn repo_status_lists_at_least_one_worktree() {
    let (r, ..) = init_repo_with_branches();
    let status = read::repo_status(&r.root);
    // The fixture has the main worktree only (no `git worktree add`).
    assert!(status.worktree_count >= 1);
    assert!(!status.worktrees.is_empty());
}

#[test]
fn repo_status_non_repo_does_not_panic() {
    // Belt-and-braces: a path under a tmpdir that doesn't contain `.git`
    // must not panic, must not error, and must return the default shape.
    let tmp = tempdir().unwrap();
    let nested = tmp.path().join("no/such/repo");
    std::fs::create_dir_all(&nested).unwrap();
    let status = read::repo_status(&nested);
    assert!(status.branch.is_empty());
    assert!(status.branches.is_empty());
    assert_eq!(status.worktree_count, 0);
}

// ── ListWorktrees (leaf for GitService.ListWorktrees) ─────────────────────
//
// Mirrors the `repo_status_*` test pattern: spin up a temp repo via git2,
// run the loader, assert against the structured WorktreeRow shape (path,
// branch, head_short, is_current). The `list_worktrees` loader is the leaf
// half of the new `ListWorktrees` RPC; these tests pin its shape so the
// helmd handler is a flat translation.

#[test]
fn list_worktrees_on_non_repo_yields_empty() {
    // A non-git path must not panic / error — return vec![].
    let tmp = tempdir().unwrap();
    let rows = read::list_worktrees(tmp.path());
    assert!(rows.is_empty());
}

#[test]
fn list_worktrees_main_only_returns_one_row_marked_current() {
    // The fixture has the main worktree only (no linked worktrees). The
    // main row is always first, marked `is_current`, and carries the
    // HEAD's short sha.
    let (r, ..) = init_repo_with_branches();
    let rows = read::list_worktrees(&r.root);
    assert_eq!(rows.len(), 1, "no linked worktrees yet, got: {:?}", rows);
    let main = &rows[0];
    assert!(main.is_current, "main worktree must be is_current");
    assert!(!main.path.is_empty());
    assert_eq!(main.branch, "feat/integration", "HEAD is feat/integration");
    assert_eq!(main.head_short.len(), 7, "head_short is the 7-char sha");
    assert!(!main.head_short.is_empty());
}

#[test]
fn list_worktrees_includes_linked_worktree() {
    // Add a linked worktree at `<tmp>/wt-job` on `job/wt-test`. The list
    // must include it as a non-current row whose `branch` is the actual
    // branch name (`job/wt-test`), not the worktree's admin name.
    let (r, ..) = init_repo_with_branches();
    let wt_path = r._tmp.path().join("wt-job");
    let out = std::process::Command::new("git")
        .args([
            "-C",
            &r.root.display().to_string(),
            "worktree",
            "add",
            "-b",
            "job/wt-test",
            &wt_path.display().to_string(),
            "job/foo",
        ])
        .output()
        .expect("git worktree add");
    assert!(out.status.success(), "git worktree add failed: {:?}", out);

    let rows = read::list_worktrees(&r.root);
    assert_eq!(rows.len(), 2, "main + linked worktree; got: {:?}", rows);

    // Main first, is_current=true.
    assert!(rows[0].is_current);
    assert_eq!(rows[0].branch, "feat/integration");

    // Linked second, is_current=false, branch == job/wt-test (the actual
    // checked-out branch, not the worktree's admin name `wt-job`).
    let linked = &rows[1];
    assert!(!linked.is_current, "linked worktree is never is_current");
    assert_eq!(linked.branch, "job/wt-test");
    assert!(
        linked.path.ends_with("wt-job"),
        "linked worktree path should end in wt-job; got `{}`",
        linked.path
    );
    assert_eq!(
        linked.head_short.len(),
        7,
        "linked head_short is the 7-char sha"
    );
}

#[test]
fn list_worktrees_empty_branch_for_detached_main() {
    // Detach HEAD on the main worktree; the main row's `branch` should
    // be empty (matches `git worktree list --porcelain` behavior on
    // detached HEAD). `is_current` and `head_short` stay populated.
    let r = init_repo();
    write_file(&r.root, "a.txt", "v1");
    let _ = initial_commit(&r.root);
    {
        let repo = Repository::open(&r.root).unwrap();
        repo.set_head_detached(repo.head().unwrap().target().unwrap())
            .unwrap();
    }
    let rows = read::list_worktrees(&r.root);
    assert_eq!(rows.len(), 1);
    let main = &rows[0];
    assert!(main.is_current);
    assert!(main.branch.is_empty(), "detached HEAD -> empty branch");
    assert_eq!(main.head_short.len(), 7);
}

// ── CreateWorktree (leaf for GitService.CreateWorktree) ────────────────────
//
// These tests pin `write::create_worktree` (subprocess-backed, the one
// libgit2-direct exception in the leaf) across the CREATED / REUSED /
// FAILED tri-state. They use a real `git` binary because the leaf
// deliberately shells out (see `write.rs` module doc for the rationale).

#[test]
fn create_worktree_creates_new_worktree_and_branch() {
    let (r, ..) = init_repo_with_branches();
    let wt_path = r._tmp.path().join("wt-new");
    let outcome = write::create_worktree(&r.root, &wt_path, "job/new", "main", false);
    match outcome {
        write::WorktreeOutcome::Created { branch, worktree_path, base_ref } => {
            assert_eq!(branch, "job/new");
            assert_eq!(worktree_path, wt_path);
            assert_eq!(base_ref, "main");
        }
        other => panic!("expected Created, got {other:?}"),
    }
    // Sanity: the worktree exists, is on job/new, and lists in `list_worktrees`.
    assert!(wt_path.exists());
    let rows = read::list_worktrees(&r.root);
    assert!(
        rows.iter().any(|w| w.branch == "job/new" && !w.is_current),
        "list_worktrees should include the new linked worktree; got: {:?}",
        rows
    );
}

#[test]
fn create_worktree_reuses_existing_on_same_branch() {
    // Add a worktree once, then call create_worktree again with the same
    // args and idempotent=true — second call should return Reused, not
    // fail with "branch already used" or "worktree already exists".
    let (r, ..) = init_repo_with_branches();
    let wt_path = r._tmp.path().join("wt-reuse");
    let first = write::create_worktree(&r.root, &wt_path, "job/reuse", "main", false);
    assert!(matches!(first, write::WorktreeOutcome::Created { .. }));
    let second = write::create_worktree(&r.root, &wt_path, "job/reuse", "main", true);
    assert!(
        matches!(second, write::WorktreeOutcome::Reused { .. }),
        "expected Reused, got {second:?}"
    );
}

#[test]
fn create_worktree_fails_when_path_is_on_different_branch() {
    // Pre-create a worktree on `job/other`, then ask for the same path
    // on `job/wanted` with idempotent=true — must fail with an error
    // naming the actual branch.
    let (r, ..) = init_repo_with_branches();
    let wt_path = r._tmp.path().join("wt-collision");
    let first = write::create_worktree(&r.root, &wt_path, "job/other", "main", false);
    assert!(matches!(first, write::WorktreeOutcome::Created { .. }));
    let second = write::create_worktree(&r.root, &wt_path, "job/wanted", "main", true);
    match second {
        write::WorktreeOutcome::Failed { error, branch, .. } => {
            assert_eq!(branch, "job/wanted");
            assert!(
                error.contains("job/other"),
                "error should name the actual branch; got `{error}`"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn create_worktree_rejects_empty_branch() {
    let r = init_repo();
    let wt_path = r._tmp.path().join("wt-empty");
    let outcome = write::create_worktree(&r.root, &wt_path, "", "main", false);
    assert!(matches!(outcome, write::WorktreeOutcome::Failed { .. }));
}

#[test]
fn create_worktree_propagates_git_failure() {
    // Asking for a branch that already exists at HEAD (without `-B`)
    // makes `git worktree add` fail; the leaf must surface FAILED with
    // a non-empty error, not Created or Reused.
    let (r, ..) = init_repo_with_branches();
    // `main` already exists in the fixture — using it as the new branch
    // name will fail because the leaf doesn't pass `-B`.
    let wt_path = r._tmp.path().join("wt-fail");
    let outcome = write::create_worktree(&r.root, &wt_path, "main", "main", false);
    match outcome {
        write::WorktreeOutcome::Failed { error, .. } => {
            assert!(!error.is_empty(), "error must be populated");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}
