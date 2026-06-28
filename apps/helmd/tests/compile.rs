//! Integration test for `HarnessService.Compile` over the live wire.
//!
//! Spins up the ConnectRPC router in-process (the same one helmd serves) on
//! an ephemeral port and POSTs a `Compile` request against a fixture
//! `.ship/` directory. Verifies the response carries the expected
//! per-harness `CompiledFile` rows (CLAUDE.md, .claude/skills/<id>/SKILL.md,
//! .mcp.json) and the `dry_run` flag round-trips correctly.
//!
//! This is the GREEN-side half of the leaf-carve TDD: the loader carve ships
//! a real (not-unimplemented) Compile op, the wire test proves the call
//! round-trips end to end.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[path = "../src/main.rs"]
mod helmd_main;

// Bring the proto + handlers into scope via the helmd binary's main module.
use helmd_main::{proto::HarnessServiceExt, HarnessImpl};

/// Path to the fixture `.ship/` dir checked into the test tree. The fixture
/// has one agent (test-agent), one skill (example-skill), one mcp server
/// (test-server), and a rule.
fn fixture_ship_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("compile_fixture")
        .join(".ship")
}

/// Build the ConnectRPC router the helmd binary serves, with the harness
/// wire registered (git wire is skipped — this test only exercises Compile).
fn build_router() -> axum::Router {
    let connect = connectrpc::Router::new();
    let connect = Arc::new(HarnessImpl).register(connect);
    axum::Router::new().fallback_service(connect.into_axum_service())
}

/// POST a Connect-JSON request to the harness service. Returns the parsed
/// JSON body on a 2xx response, panics otherwise (test failure).
async fn call_compile(
    app: &axum::Router,
    body: serde_json::Value,
) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri("/ship.internal.v1.HarnessService/Compile")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.expect("in-process Compile");
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("response not valid JSON: {e}; bytes={:?}", bytes));
    (status, parsed)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_emits_claude_files_for_fixture() {
    let app = build_router();
    let ship_dir = fixture_ship_dir();
    assert!(ship_dir.exists(), "fixture .ship/ must exist at {ship_dir:?}");

    let (status, body) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": ship_dir.parent().unwrap().to_string_lossy(),
            "harness": "claude",
            "dryRun": true,
        }),
    )
    .await;

    assert_eq!(status, 200, "Compile RPC must succeed; body={body}");

    // The harness field was honored.
    let harnesses = body["harnesses"].as_array().expect("harnesses array");
    assert_eq!(harnesses.len(), 1);
    assert_eq!(harnesses[0], "claude");

    // dryRun round-trips.
    assert_eq!(body["dryRun"], true);

    // The file set covers the canonical Claude outputs.
    let files = body["files"].as_array().expect("files array");
    let paths: Vec<&str> = files
        .iter()
        .map(|f| f["path"].as_str().expect("path string"))
        .collect();
    assert!(
        paths.contains(&"CLAUDE.md"),
        "must emit CLAUDE.md; got {paths:?}"
    );
    assert!(
        paths.contains(&".claude/skills/example-skill/SKILL.md"),
        "must emit the example-skill file; got {paths:?}"
    );
    assert!(
        paths.contains(&".mcp.json"),
        "must emit the .mcp.json MCP config; got {paths:?}"
    );
    // The .claude/settings.json patch is always emitted by the engine.
    assert!(
        paths.contains(&".claude/settings.json"),
        "must emit .claude/settings.json; got {paths:?}"
    );

    // Per-file content sanity: the SKILL.md content is the original raw file.
    let skill_file = files
        .iter()
        .find(|f| f["path"] == ".claude/skills/example-skill/SKILL.md")
        .expect("example-skill file");
    let content = skill_file["content"].as_str().expect("content string");
    assert!(
        content.contains("Example Skill"),
        "skill content must round-trip; got:\n{content}"
    );

    // The MCP JSON file must include the test-server entry.
    let mcp_file = files
        .iter()
        .find(|f| f["path"] == ".mcp.json")
        .expect(".mcp.json file");
    let mcp_content = mcp_file["content"].as_str().expect("mcp content");
    assert!(
        mcp_content.contains("test-server"),
        "mcp.json must include the test-server entry; got:\n{mcp_content}"
    );
    assert!(
        mcp_content.contains("\"command\": \"npx\""),
        "mcp.json must include the stdio command; got:\n{mcp_content}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_rejects_missing_repo_path() {
    let app = build_router();
    let (status, _body) = call_compile(
        &app,
        serde_json::json!({ "repoPath": "", "harness": "claude", "dryRun": true }),
    )
    .await;
    assert_eq!(
        status, 400,
        "empty repo_path must be a 400 (invalid_argument), not 200"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_dry_run_does_not_write_files() {
    // Use a fresh temp dir as the "repo" so we can verify no files were
    // written under it. Compile with dry_run=true.
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path();
    let ship_dir = repo.join(".ship");
    std::fs::create_dir_all(&ship_dir).expect("create .ship");
    // Mirror the fixture files into the temp repo so load_library succeeds.
    let fixture_ship = fixture_ship_dir();
    for entry in std::fs::read_dir(&fixture_ship).expect("read fixture") {
        let entry = entry.expect("entry");
        let from = entry.path();
        let to = ship_dir.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to).expect("copy dir");
        } else {
            std::fs::copy(&from, &to).expect("copy file");
        }
    }

    let app = build_router();
    let (status, body) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": repo.to_string_lossy(),
            "harness": "claude",
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(status, 200, "Compile dry_run must succeed; body={body}");
    assert_eq!(body["dryRun"], true);

    // dry_run must NOT have written CLAUDE.md to disk.
    assert!(
        !repo.join("CLAUDE.md").exists(),
        "dry_run=true must not write CLAUDE.md to disk"
    );
    assert!(
        !repo.join(".claude/skills/example-skill/SKILL.md").exists(),
        "dry_run=true must not write skill files to disk"
    );
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
