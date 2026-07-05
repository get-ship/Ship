//! Integration test for the Land handler's land-or-clean cleanup gate.
//!
//! Spins up the ConnectRPC router in-process (the same one helmd serves) on
//! an ephemeral router and POSTs `Land` requests. The leaf-only mod
//! (`main.rs::land_cleanup_tests`) exercises `post_land_cleanup_at` directly
//! — this file drives the same scenarios through the Wire handler so the
//! router → handler → leaf path is exercised end-to-end
//! (c-2026-07-05T17-30-00-000Z-f6a7b8c9: the live Land path emitted nothing
//! because the cleanup line never surfaced;
//! c-2026-07-05T18-10-00-000Z-a7b8c9d0: WIP must be refused BEFORE the merge
//! runs).
//!
//! Sets `SHIP_WORKTREE_DIR` to a tempdir root so the canonical
//! `ship_config::worktrees::worktrees_dir` resolver points at the fixture
//! worktree path — the handler then walks the same path the dispatcher
//! writes to in production.

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

async fn call_land(app: &axum::Router, body: serde_json::Value) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri("/ship.internal.v1.GitService/Land")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.expect("in-process Land");
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
/// its own, plus an attached worktree rooted at `wt_root/<slug>`. Pins
/// `SHIP_WORKTREE_DIR` so the handler resolves the same worktree root the
/// fixture uses. The `job/<slug>` commit exists so the land has a real diff
/// to merge (not NothingStaged).
fn setup_land_handler_fixture(slug: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    let wt_root = tmp.path().join("worktrees");
    std::fs::create_dir_all(&wt_root).unwrap();
    let worktree_path = wt_root.join(slug);
    std::fs::create_dir_all(worktree_path.parent().unwrap()).unwrap();

    // SAFETY: the value is a tempdir-scoped path; cargo-test serializes
    // module-level test execution here. Setting process env is the supported
    // seam for `ship_config::worktrees::worktrees_dir`.
    unsafe {
        std::env::set_var("SHIP_WORKTREE_DIR", &wt_root);
    }

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
    // Drop a real change on the job branch inside its worktree — the leaf
    // refuses `NothingStaged` (squash of an ancestor == nothing to merge),
    // so the fixture must give the land something concrete to merge.
    std::fs::write(worktree_path.join("feat.txt"), format!("from {slug}\n")).unwrap();
    git(&worktree_path, &["add", "feat.txt"]);
    git(&worktree_path, &["commit", "-m", "add feat"]);
    (tmp, root, worktree_path)
}

/// Headline handler-level proof (c-2026-07-05T17-30-00-000Z-f6a7b8c9): the
/// Land handler reaches the cleanup leaf and tears down a clean
/// `job/<slug>` end-to-end. Observable via filesystem state (branch +
/// worktree gone + `committed` in response) — the same evidence the operator
/// sees in helmd's stderr (the gate's `eprintln!` carries
/// `branch_deleted=true worktree_removed=true`).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn land_handler_removes_branch_and_clean_worktree() {
    let app = build_router();
    let (_tmp, root, wt_path) = setup_land_handler_fixture("auth");
    assert!(wt_path.exists(), "fixture worktree must exist");

    let (status, body) = call_land(
        &app,
        serde_json::json!({
            "repoPath": root.to_string_lossy(),
            "sourceBranch": "job/auth",
            "targetBranch": "main",
            "message": "land job/auth",
        }),
    )
    .await;

    assert_eq!(status, 200, "clean land must succeed; body={body}");
    let sha = body
        .pointer("/committed")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(sha.len(), 40, "committed sha must be a real sha; body={body}");

    // Cleanup proved by filesystem: the SOURCE BRANCH is gone, the WORKTREE
    // is gone — those are the booleans the gate's `eprintln!` carries. The
    // leaf reached `git branch -D` + `git worktree remove` from the handler
    // (the path the live missed).
    assert!(!wt_path.exists(), "worktree must be removed");
    let branch_list = Command::new("git")
        .args(["-C", root.to_str().unwrap(), "branch", "--list", "job/auth"])
        .output()
        .expect("git branch list");
    assert!(
        String::from_utf8_lossy(&branch_list.stdout).trim().is_empty(),
        "job/auth must be absent from the branch list"
    );
}

/// The handler-level guard for the dirty-target case
/// (c-2026-07-05T18-10-00-000Z-a7b8c9d0): when the target branch is the
/// checked-out branch AND the working tree has tracked modifications,
/// Land returns `failed_precondition` with the dirty paths and the repo
/// is untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn land_handler_refuses_tracked_dirty_target() {
    let app = build_router();
    let (_tmp, root, wt_path) = setup_land_handler_fixture("wip");

    // Operator's tracked WIP on `main`.
    std::fs::write(root.join("README.md"), "operator-wip").unwrap();

    let (status, body) = call_land(
        &app,
        serde_json::json!({
            "repoPath": root.to_string_lossy(),
            "sourceBranch": "job/wip",
            "targetBranch": "main",
            "message": "land job/wip",
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
        "source branch must remain (land was refused)"
    );
}
