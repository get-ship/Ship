//! Parity capture for slice 2b close-out: invoke the live `HarnessService/Compile`
//! against the worktree root, SHA-256 every emitted `CompiledFile`, sort by
//! (harness, path), and write the full map to a file (or print to stdout).
//!
//! Run:
//!   HELMD_PARITY_OUT=/tmp/parity-before.txt cargo test -p helmd --test parity_capture -- --nocapture
//!
//! The single test `capture_full_compile_output_against_worktree_root` does the capture.

use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[path = "../src/main.rs"]
mod helmd_main;

use helmd_main::{proto::HarnessServiceExt, HarnessImpl};

fn build_router() -> axum::Router {
    let connect = connectrpc::Router::new();
    let connect = Arc::new(HarnessImpl).register(connect);
    axum::Router::new().fallback_service(connect.into_axum_service())
}

async fn call_compile(app: &axum::Router, body: serde_json::Value) -> (u16, serde_json::Value) {
    let bytes = serde_json::to_vec(&body).expect("request body encodable");
    let req = Request::builder()
        .method("POST")
        .uri("/ship.internal.v1.HarnessService/Compile")
        .header("content-type", "application/json")
        .body(Body::from(bytes))
        .expect("build compile request");
    let response = app.clone().oneshot(req).await.expect("compile resp");
    let status = response.status().as_u16();
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&body_bytes)
        .unwrap_or_else(|e| panic!("response not valid JSON: {e}; bytes={:?}", body_bytes));
    (status, parsed)
}

fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    use std::fmt::Write;
    for b in digest {
        let _ = write!(out, "{:02x}", b);
    }
    out
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capture_full_compile_output_against_worktree_root() {
    let app = build_router();
    let root = std::env::current_dir()
        .expect("cwd")
        .ancestors()
        .find(|p| p.join("stack.config.ts").exists())
        .map(|p| p.to_path_buf())
        .expect("could not locate stack.config.ts in cwd ancestors");
    eprintln!("[parity] worktree root: {}", root.display());

    // First call: pick the full harness list from the resolved config.
    let (status, body) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": root.to_string_lossy(),
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(status, 200, "Compile failed; body={body}");

    let harnesses: Vec<String> = body["harnesses"]
        .as_array()
        .expect("harnesses array")
        .iter()
        .map(|h| h.as_str().expect("harness string").to_string())
        .collect();
    assert!(!harnesses.is_empty(), "Compile must use at least one harness");
    let mut sorted_harnesses = harnesses.clone();
    sorted_harnesses.sort_unstable();

    // Now run once PER harness with `harness` pinned, so we get a clean
    // attributed file list per harness.
    let mut rows: Vec<(String, String, String, String)> = Vec::new(); // (harness, path, sha256, byte_len)
    for h in &sorted_harnesses {
        let (s, b) = call_compile(
            &app,
            serde_json::json!({
                "repoPath": root.to_string_lossy(),
                "harness": h,
                "dryRun": true,
            }),
        )
        .await;
        assert_eq!(s, 200, "Compile({h}) failed; body={b}");
        let files = b["files"].as_array().expect("files");
        for f in files {
            let path = f["path"].as_str().expect("path").to_string();
            let content = f["content"].as_str().expect("content").to_string();
            let hash = sha256_hex(&content);
            rows.push((h.clone(), path, hash, content.len().to_string()));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = String::new();
    out.push_str("# parity capture (worktree-wide; per-harness Compile output)\n");
    out.push_str(&format!("# worktree: {}\n", root.display()));
    out.push_str(&format!("# harnesses: {}\n", sorted_harnesses.join(",")));
    out.push_str(&format!("# file_count: {}\n", rows.len()));
    out.push_str("# columns: harness\\tpath\\tsha256\\tbyte_len\n");
    for (harness, path, hash, byte_len) in &rows {
        out.push_str(&format!("{harness}\t{path}\t{hash}\t{byte_len}\n"));
    }

    if let Ok(path) = std::env::var("HELMD_PARITY_OUT") {
        std::fs::write(&path, &out).expect("write parity snapshot");
        eprintln!(
            "[parity] wrote {} entries across {} harnesses to {path}",
            rows.len(),
            sorted_harnesses.len()
        );
    } else {
        print!("{out}");
    }

    // Dump CLAUDE.md + rules content (debug aid — diffs the on-disk walk effect).
    let claude_row = rows.iter().find(|(_, p, _, _)| p == "CLAUDE.md");
    if let Some((_, _, _, _)) = claude_row {
        // Re-run Compile to get CLAUDE.md content (already in `body`)
        let (_, b) = call_compile(
            &app,
            serde_json::json!({
                "repoPath": root.to_string_lossy(),
                "harness": "claude",
                "dryRun": true,
            }),
        )
        .await;
        if let Some(f) = b["files"].as_array().and_then(|a| a.iter().find(|f| f["path"] == "CLAUDE.md")) {
            eprintln!("=== CLAUDE.md content (first 400 bytes) ===");
            let c = f["content"].as_str().unwrap_or("");
            let head: String = c.chars().take(400).collect();
            eprintln!("{head}");
            eprintln!("=== CLAUDE.md full length: {} ===", c.len());
            std::fs::write("/tmp/claude-md-current.txt", c).unwrap();
        }
    }
}
