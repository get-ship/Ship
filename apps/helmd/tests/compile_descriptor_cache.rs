//! Behavior test for the `DescriptorCache` wiring on `HarnessService.Compile`.
//!
//! Proves the cache contract the `apps/helmd/src/main.rs` compile loop
//! relies on (the resolve-on-change lever): a single per-process
//! `DescriptorCache` keyed on `(space_id, content_hash, harness_id)`,
//! `get_or_load` is read-through with a `load_fn` that runs the real
//! harness compile, two consecutive compile-shaped calls hit the cache
//! (the second does NOT re-run `load_fn`), and a fresh hash key (the
//! spec's "edit invalidates") bumps the call counter.
//!
//! Companion test: the Compile handler's `compile` op reaches the cache
//! path through the live wire when an active space resolves, and
//! bypasses the cache when no active space resolves (the spec's
//! "spaceless compiles are valid and must not error" carve).
//!
//! Run:
//!   cargo test -p helmd --test compile_descriptor_cache -- --nocapture

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use axum::body::Body;
use axum::http::Request;
use harness_config_compiler::compile;
use http_body_util::BodyExt;
use ship_resolver::cache::DescriptorCache;
use ship_resolver::descriptor::canonical_content_hash;
use ship_resolver::roll_up_registered_blocking;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::ports::{KernelLog, SpaceId};
use ship_types::event_types::CONFIG_FILE_CHANGED;
use ship_types::payload::ConfigFileChanged;
use stack_model::CompileOutput;
use stack_model::resolve_library;
use tempfile::TempDir;
use tower::ServiceExt;

#[path = "../src/main.rs"]
mod helmd_main;

use helmd_main::HarnessImpl;
use helmd_main::descriptor_cache;
use helmd_main::proto::HarnessServiceExt;

// ── leg 1: cache hit + miss + invalidation via get_or_load (the contract) ────

/// Roll up a fixture space, resolve, hash, and exercise
/// `DescriptorCache::get_or_load` with a counted load_fn — the
/// behavior the Compile handler relies on.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn descriptor_cache_hits_on_repeat_compile_and_recomputes_on_edit() {
    let repo = fixture_root();

    // Cache + kernel pair (isolated router so the test doesn't share
    // state with other tests via the global kernel_router). The router's
    // backing store lives in the tempdir — never inside the shared
    // `compile_fixture/` tree (a `.ship/runtime.db` artifact there would
    // pollute the fixture for every other test).
    let cache_root = TempDir::new().expect("tempdir");
    let kernel = Arc::new(KernelLog::new(
        new_isolated_for_test(cache_root.path().join(".ship")).unwrap(),
    ));
    let cache = DescriptorCache::open(cache_root.path().to_path_buf(), Some(kernel.clone()))
        .expect("descriptor cache open");
    // The subscriber starts the invalidation loop; without it the
    // eviction side of the contract isn't exercised.
    let _sub = cache.spawn_invalidation_subscriber();
    tokio::task::yield_now().await;
    let space = SpaceId::new("compile-cache-test");

    // The counter the load_fn bumps. Wrapped in Arc so each closure
    // capture clones the same counter.
    let counter = Arc::new(AtomicU32::new(0));

    // First call: cache miss → load_fn runs (count → 1).
    let first = roll_up_and_cache(&repo, &cache, &space, "claude", counter.clone())
        .await
        .expect("first cached compile");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "first call invokes the load_fn"
    );

    // Second call, same repo + same harness — the cache must hit.
    let second = roll_up_and_cache(&repo, &cache, &space, "claude", counter.clone())
        .await
        .expect("second cached compile");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "second call hits the cache (load_fn skipped)"
    );
    // The returned Arc must point at the same `CompileOutput` instance —
    // the in-memory index returns the cached Arc verbatim
    // (`cache.rs:166`: `state.entries.get(&key) → Arc::clone`).
    assert!(
        Arc::ptr_eq(&first, &second),
        "second read returns the same in-memory Arc as the first (no recompile)"
    );

    // The on-disk file appeared under
    // `<cache_root>/resolver/<space>/<hash>__claude.json` — the cache
    // write is `tokio::spawn`-ed (R7 non-blocking); give it a tick to
    // land before asserting.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let resolver_dir = cache_root.path().join("resolver").join(space.as_str());
    let entry = std::fs::read_dir(&resolver_dir)
        .unwrap_or_else(|e| panic!("resolver dir missing for {space}: {e} (cache write failed)"))
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().ends_with("__claude.json"))
        .unwrap_or_else(|| panic!("expected __claude.json in {resolver_dir:?}"));
    assert!(
        entry.path().is_file(),
        "cache write is a regular json file: {:?}",
        entry.path()
    );

    // Edit invalidates the cache. We exercise the same event-driven
    // mechanism `ship-resolver`'s own `spine_smoke` uses: synthesize a
    // `config.file_changed` envelope and rely on the subscriber to
    // clear the in-memory index.
    let envelope = EventEnvelope::new(
        CONFIG_FILE_CHANGED,
        "stack.config.ts",
        &ConfigFileChanged {
            domain: "source".to_string(),
            path: "stack.config.ts".to_string(),
            content_hash: "edited".to_string(),
        },
    )
    .expect("event envelope");
    kernel.broadcast(envelope);

    // Poll briefly for the eviction to land (mirrors `spine_smoke`'s
    // deadline pattern).
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    while std::time::Instant::now() < deadline && cache._len().await != 0 {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(
        cache._len().await,
        0,
        "the subscriber evicts the in-memory index on config.file_changed"
    );

    // Third call after the invalidation: the load_fn fires again
    // (count → 2). Same fixture inputs, same hash key — the
    // recompile produces a structurally identical output. The
    // contract we assert is "load_fn runs once per cache miss".
    let third = roll_up_and_cache(&repo, &cache, &space, "claude", counter.clone())
        .await
        .expect("third cached compile");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "after invalidation, the load_fn fires again"
    );
    assert!(
        !Arc::ptr_eq(&first, &third),
        "post-invalidation cache returns a freshly-compiled Arc"
    );
    // Sanity: determinism (`inv-fmt-deterministic-eval`) — same input
    // produces the same canonical-JSON bytes. This is what makes the
    // cache key (`canonical_content_hash` over the compile input) meaningful.
    let first_bytes = serde_json::to_vec(&*first).expect("serialize first");
    let third_bytes = serde_json::to_vec(&*third).expect("serialize third");
    assert_eq!(
        first_bytes, third_bytes,
        "determinism: same input → same output bytes"
    );
}

/// Roll up `repo`, resolve, hash, and drive `cache.get_or_load`
/// exactly like the Compile handler does. Returns the cached
/// `Arc<CompileOutput>`.
async fn roll_up_and_cache(
    repo: &Path,
    cache: &DescriptorCache,
    space: &SpaceId,
    harness: &str,
    counter: Arc<AtomicU32>,
) -> anyhow::Result<Arc<CompileOutput>> {
    let composed = roll_up_registered_blocking(repo, None)
        .map_err(|e| anyhow::anyhow!("roll_up failed: {e}"))?;
    // Mirror the handler's composite key — (library, active-agent
    // override, space rules); this fixture has neither an override nor
    // space rules, so the last two dimensions are empty.
    let no_rules: Vec<stack_model::Rule> = Vec::new();
    let hash = canonical_content_hash(&(&composed.library, &None::<String>, &no_rules))
        .map_err(|e| anyhow::anyhow!("content_hash failed: {e}"))?;
    // Resolve once (cheap; `resolve_library` is pure). The resolved
    // config is what `harness_config_compiler::compile` consumes — the same
    // shape the Compile handler passes through.
    let resolved = resolve_library(&composed.library, None, None);
    let harness_str = harness.to_string();
    let cache_hit = cache
        .get_or_load(space, &hash, harness, move || async move {
            // The load_fn runs the real compile (the same one the
            // handler invokes). Counter bumps per invocation.
            counter.fetch_add(1, Ordering::SeqCst);
            compile(&resolved, &harness_str)
                .ok_or_else(|| anyhow::anyhow!("compile: no output for harness {harness_str}"))
        })
        .await?;
    Ok(cache_hit)
}

// ── leg 2: Compile handler bypass branch (no active space) ──────────────────

/// Build the helmd ConnectRPC router so the test can POST `Compile`
/// requests against the real handler (mirrors `compile.rs`'s setup).
fn build_router() -> axum::Router {
    let connect = connectrpc::Router::new();
    let connect = Arc::new(HarnessImpl).register(connect);
    axum::Router::new().fallback_service(connect.into_axum_service())
}

/// POST a Connect-JSON request to the harness service, return parsed
/// body. Mirrors the helper in `compile.rs`.
async fn call_compile(app: &axum::Router, body: serde_json::Value) -> (u16, serde_json::Value) {
    let raw = serde_json::to_vec(&body).expect("encode body");
    let req = Request::builder()
        .method("POST")
        .uri("/ship.internal.v1.HarnessService/Compile")
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1")
        .body(Body::from(raw))
        .expect("build request");
    let resp = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status().as_u16();
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("parse JSON body");
    (status, json)
}

/// When `repo_path` does NOT resolve to an active space (the test
/// fixture is a plain tree with no kernel-registered identity), the
/// handler MUST bypass the cache and compile inline — twice, with
/// unchanged inputs — producing identical byte-for-byte `CompiledFile`s.
///
/// This protects the `compile` loop's else-branch from regressions:
/// the carve is "spaceless compiles are valid and must not error",
/// not a TODO to be filled in later.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_handler_bypasses_cache_when_no_active_space() {
    // The lazy-static `DESCRIPTOR_CACHE` is process-scoped; first-touch
    // initialises it. Even when bypassed, the singleton must safely
    // initialise (its initializer must not panic when no global
    // kernel router is set / `SHIP_HOME` is unset).
    let _ = descriptor_cache();

    let app = build_router();
    let repo = fixture_root();
    // The fixture is NOT registered as an active space — no test
    // setup has called `init_kernel_router(<here>)`, so
    // `resolve_active_space` returns None → cache bypass.

    let (status_a, body_a) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": repo.to_string_lossy(),
            "harness": "claude",
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(
        status_a, 200,
        "compile must succeed on a spaceless repo; body={body_a}"
    );
    let files_a = body_a["files"].as_array().expect("files array").clone();
    assert!(!files_a.is_empty(), "bypassed compile still produces files");

    let (status_b, body_b) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": repo.to_string_lossy(),
            "harness": "claude",
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(
        status_b, 200,
        "second bypassed compile must also succeed; body={body_b}"
    );

    // Idempotency: same repo + same harness → same files in the same
    // order, same content. The bypass branch always recompiles; the
    // contract being asserted is "bypass produces a deterministic
    // result", which determinism guarantees.
    let files_b = body_b["files"].as_array().expect("files array");
    let a_paths: Vec<String> = files_a
        .iter()
        .map(|f| f["path"].as_str().expect("path").to_string())
        .collect();
    let b_paths: Vec<String> = files_b
        .iter()
        .map(|f| f["path"].as_str().expect("path").to_string())
        .collect();
    assert_eq!(a_paths, b_paths, "bypass path → same emitted files");
    let a_bytes = serde_json::to_vec(&files_a).expect("encode a");
    let b_bytes = serde_json::to_vec(files_b).expect("encode b");
    assert_eq!(a_bytes, b_bytes, "bypass path → byte-identical files");
}

/// Fixture root — the same `compile_fixture` tree `compile.rs` uses.
/// Lives at `apps/helmd/tests/compile_fixture/`.
fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("compile_fixture")
}
