//! Integration test for GitService/SquashMerge on shipd (moved from helmd, j-02 step 2).
//!
//! Spins up the git Connect router in-process and POSTs SquashMerge:
//!   1. Clean squash commits on the target; source branch + worktree remain.
//!   2. Tracked-dirty target → failed_precondition; repo untouched.

use std::path::PathBuf;
use std::process::Command;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

fn build_router() -> axum::Router {
    shipd::rpc::ops::git_wire::axum_router()
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
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .expect("in-process SquashMerge");
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
    assert_eq!(
        sha.len(),
        40,
        "committed sha must be a real sha; body={body}"
    );

    assert!(wt_path.exists(), "primitive must NOT remove the worktree");
    let branch_list = Command::new("git")
        .args(["-C", root.to_str().unwrap(), "branch", "--list", "job/auth"])
        .output()
        .expect("git branch list");
    assert!(
        !String::from_utf8_lossy(&branch_list.stdout)
            .trim()
            .is_empty(),
        "primitive must NOT delete the source branch"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn squash_merge_refuses_tracked_dirty_target() {
    let app = build_router();
    let (_tmp, root, wt_path) = setup_fixture("wip");

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
