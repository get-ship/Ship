//! Integration test for the SquashMerge handler — the git-native primitive
//! the client-side `land`/`release` compositions (helm
//! `src/git-ops/{land,release}.ts`) build on.
//!
//! Spins up the ConnectRPC router in-process (the same one helmd serves) and
//! POSTs `SquashMerge` requests, covering the two wire-level contracts:
//!
//!   1. A clean squash commits ONE commit on the target and performs NO
//!      cleanup — the source branch and its worktree remain. Workflow
//!      cleanup (branch delete, worktree removal) is the client
//!      composition's job; the server carries no workflow vocabulary.
//!   2. A tracked-dirty target is refused with `failed_precondition` +
//!      the dirty paths BEFORE the merge runs
//!      (c-2026-07-05T18-10-00-000Z-a7b8c9d0), repo untouched.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[path = "../src/main.rs"]
mod helmd_main;

use helmd_main::proto::GitServiceExt;
use helmd_main::GitImpl;

fn build_router() -> axum::Router {
    let connect = connectrpc::Router::new();
    let connect = Arc::new(GitImpl).register(connect);
    axum::Router::new().fallback_service(connect.into_axum_service())
}

async fn call_squash_merge(
    app: &axum::Router,
    body: serde_json::Value,
) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri("/ship.internal.v1.GitService/SquashMerge")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.expect("in-process SquashMerge");
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("response not valid JSON: {e}; bytes={:?}", bytes));
    (status, parsed)
}

fn git(cwd: &PathBuf, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git failed");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Build a one-commit repo with a `job/<slug>` branch carrying one commit of
/// its own, plus an attached worktree. The `job/<slug>` commit exists so the
/// squash has a real diff to merge (not NothingStaged).
fn setup_fixture(slug: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    let worktree_path = tmp.path().join("worktrees").join(slug);
    std::fs::create_dir_all(worktree_path.parent().unwrap()).unwrap();

    git(&root, &["init", "--initial-branch=main"]);
    git(&root, &["config", "user.email", "t@example.com"]);
    git(&root, &["config", "user.name", "test"]);
    std::fs::write(root.join("README.md"), "base\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "init"]);
    git(
        &root,
        &[
            "worktree",
            "add",
            "-b",
            &format!("job/{slug}"),
            worktree_path.to_str().unwrap(),
        ],
    );
    std::fs::write(worktree_path.join("feat.txt"), format!("from {slug}\n")).unwrap();
    git(&worktree_path, &["add", "feat.txt"]);
    git(&worktree_path, &["commit", "-m", "add feat"]);
    (tmp, root, worktree_path)
}

/// A clean squash commits on the target and performs NO cleanup — the
/// primitive has no workflow vocabulary, so the source branch and its
/// worktree must survive the call.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn squash_merge_commits_and_leaves_source_branch_and_worktree() {
    let app = build_router();
    let (_tmp, root, wt_path) = setup_fixture("auth");
    assert!(wt_path.exists(), "fixture worktree must exist");

    let (status, body) = call_squash_merge(
        &app,
        serde_json::json!({
            "repoPath": root.to_string_lossy(),
            "sourceRef": "job/auth",
            "targetBranch": "main",
            "message": "squash job/auth",
        }),
    )
    .await;

    assert_eq!(status, 200, "clean squash must succeed; body={body}");
    let sha = body
        .pointer("/committed")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(sha.len(), 40, "committed sha must be a real sha; body={body}");

    // NO cleanup: branch + worktree remain — that's the composition's job.
    assert!(wt_path.exists(), "primitive must NOT remove the worktree");
    let branch_list = Command::new("git")
        .args(["-C", root.to_str().unwrap(), "branch", "--list", "job/auth"])
        .output()
        .expect("git branch list");
    assert!(
        !String::from_utf8_lossy(&branch_list.stdout).trim().is_empty(),
        "primitive must NOT delete the source branch"
    );
}

/// The wire-level guard for the dirty-target case
/// (c-2026-07-05T18-10-00-000Z-a7b8c9d0): when the target branch is the
/// checked-out branch AND the working tree has tracked modifications,
/// SquashMerge returns `failed_precondition` with the dirty paths and the
/// repo is untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn squash_merge_refuses_tracked_dirty_target() {
    let app = build_router();
    let (_tmp, root, wt_path) = setup_fixture("wip");

    // Operator's tracked WIP on `main`.
    std::fs::write(root.join("README.md"), "operator-wip").unwrap();

    let (status, body) = call_squash_merge(
        &app,
        serde_json::json!({
            "repoPath": root.to_string_lossy(),
            "sourceRef": "job/wip",
            "targetBranch": "main",
            "message": "squash job/wip",
        }),
    )
    .await;

    // Connect-RPC maps its `failed_precondition` code to HTTP 400 (the
    // standard Connect error-code → HTTP mapping, distinct from gRPC's 412).
    // The body's `code` field is authoritative.
    assert!(
        body.pointer("/code").and_then(|v| v.as_str()) == Some("failed_precondition"),
        "expected failed_precondition error body; got status={status} body={body}"
    );
    let error_text = body
        .pointer("/message")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(
        error_text.contains("README.md"),
        "expected dirty path in error message; got: {error_text}"
    );
    assert!(
        error_text.contains("commit or stash"),
        "expected remediation hint; got: {error_text}"
    );

    // Repo untouched: WIP bytes preserved, source branch + worktree still present.
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "operator-wip"
    );
    assert!(wt_path.exists(), "worktree must NOT be touched on refusal");
    let branch_list = Command::new("git")
        .args(["-C", root.to_str().unwrap(), "branch", "--list", "job/wip"])
        .output()
        .expect("git branch list");
    assert!(
        !String::from_utf8_lossy(&branch_list.stdout)
            .trim()
            .is_empty(),
        "source branch must remain (the squash was refused)"
    );
}
