//! Integration test for `HarnessService.Compile` over the live wire.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[path = "../src/main.rs"]
mod helmd_main;

use helmd_main::{proto::HarnessServiceExt, HarnessImpl};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("compile_fixture")
}

fn build_router() -> axum::Router {
    let connect = connectrpc::Router::new();
    let connect = Arc::new(HarnessImpl).register(connect);
    axum::Router::new().fallback_service(connect.into_axum_service())
}

async fn call_compile(app: &axum::Router, body: serde_json::Value) -> (u16, serde_json::Value) {
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_emits_claude_files_for_fixture() {
    let app = build_router();
    let root = fixture_root();
    let (status, body) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": root.to_string_lossy(),
            "harness": "claude",
            "activeAgent": "test-agent",
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(status, 200, "Compile must succeed; body={body}");
    let files = body["files"].as_array().expect("files array");
    let paths: Vec<&str> = files
        .iter()
        .map(|f| f["path"].as_str().expect("path string"))
        .collect();
    assert!(paths.contains(&"CLAUDE.md"), "must emit CLAUDE.md; got {paths:?}");
    assert!(paths.contains(&".claude/skills/example-skill/SKILL.md"));
    assert!(paths.contains(&".mcp.json"));
    assert!(paths.contains(&".claude/settings.json"));
    let skill = files.iter().find(|f| f["path"] == ".claude/skills/example-skill/SKILL.md").unwrap();
    let sc = skill["content"].as_str().unwrap();
    assert!(sc.contains("Example Skill"), "skill body round-trips; got {sc}");
    let mcp = files.iter().find(|f| f["path"] == ".mcp.json").unwrap();
    let mc = mcp["content"].as_str().unwrap();
    assert!(mc.contains("test-server"));
    assert!(mc.contains("\"command\": \"npx\""));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_rejects_missing_repo_path() {
    let app = build_router();
    let (status, _body) = call_compile(
        &app,
        serde_json::json!({ "repoPath": "", "harness": "claude", "dryRun": true }),
    )
    .await;
    assert_eq!(status, 400);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_dry_run_does_not_write_files() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::copy(fixture_root().join("ship.config.ts"), repo.join("ship.config.ts")).unwrap();
    let app = build_router();
    let (status, _body) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": repo.to_string_lossy(),
            "harness": "claude",
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(status, 200);
    assert!(!repo.join("CLAUDE.md").exists());
    assert!(!repo.join(".claude/skills/example-skill/SKILL.md").exists());
}

struct SpaceRulesFixture {
    repo_canonical: PathBuf,
    space_id: String,
    _ship_home: ship_substrate::testutil::EnvGuard,
    _repo_tmp: tempfile::TempDir,
    _ship_home_tmp: tempfile::TempDir,
    kernel: Arc<ship_substrate::ports::KernelLog>,
}

async fn setup_space_rules_fixture() -> SpaceRulesFixture {
    let repo_tmp = tempfile::tempdir().unwrap();
    let ship_home_tmp = tempfile::tempdir().unwrap();
    let repo_path = repo_tmp.path().to_path_buf();
    let ship_dir = ship_home_tmp.path().to_path_buf();
    copy_dir_recursive(&fixture_root(), &repo_path).unwrap();

    let ship_home = ship_substrate::testutil::EnvGuard::set(ship_substrate::env::Var::Home, &ship_dir);

    // The kernel and the per-scope db accessors must agree on the base
    // path. `db::space_db_path` + `db::runtime_db_path` use
    // `get_global_dir()` (= `<SHIP_HOME>/data`); the kernel's
    // PartitionedSink writes to `<base>/runtime.db`. Pass the SAME data
    // path to both so the runtime.db the kernel writes to is the file
    // `db::spaces::find_by_path` reads from.
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();

    use ship_space::SpacesProjection;
    ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
        vec![Box::new(SpacesProjection)]
    }));

    let router = ship_substrate::events::new_isolated_for_test(data_dir).unwrap();
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(router));

    let canonical_repo = std::fs::canonicalize(&repo_path).unwrap();
    let record = ship_space::identity::ensure_identity_for_root(
        &kernel,
        &canonical_repo,
        Some("space-rules-test"),
    )
    .await
    .expect("ensure_identity_for_root");

    SpaceRulesFixture {
        repo_canonical: canonical_repo,
        space_id: record.space_id,
        _ship_home: ship_home,
        _repo_tmp: repo_tmp,
        _ship_home_tmp: ship_home_tmp,
        kernel,
    }
}

async fn seed_three_rules(fx: &SpaceRulesFixture) {
    // The settings_overlay reads/writes through `db::space_db_path`. The
    // `fx._ship_home_tmp` is `<tmp>`; the data dir is `<tmp>/data`, the
    // canonical ship_dir for `db::*_db_path`.
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    let account_db_path = ship_substrate::db::account_db_path(&data_dir);
    let account_svc = ship_substrate::data::DataService::open_at(&account_db_path).unwrap();
    let account_store: Arc<dyn ship_substrate::ports::DataStore> =
        Arc::new(ship_substrate::SingleBackendDataStore::new(account_svc, "account"));
    let space_db_path = ship_substrate::db::space_db_path(&data_dir, &fx.space_id);
    let space_svc = ship_substrate::data::DataService::open_at(&space_db_path).unwrap();
    let space_store: Arc<dyn ship_substrate::ports::DataStore> =
        Arc::new(ship_substrate::SingleBackendDataStore::new(space_svc, &fx.space_id));
    let ctx = ship_config::service::SettingsCtx {
        kernel: fx.kernel.clone(),
        account_store,
        space: Some((fx.space_id.clone(), space_store)),
        is_operator: true,
    };
    let settings = ship_config::service::SettingsService::new(ctx);
    use ship_config::service::SettingScope;

    settings
        .set(
            SettingScope::Space,
            "rules.bravo",
            json!({"body": "BRAVO_BODY_FROM_SPACE_RULE", "order": 20, "enabled": true, "attribution": "agent:compile-test"}),
        )
        .await
        .expect("set bravo");
    settings
        .set(
            SettingScope::Space,
            "rules.alpha",
            json!({"body": "ALPHA_BODY_FROM_SPACE_RULE", "order": 10, "enabled": true, "attribution": "agent:compile-test"}),
        )
        .await
        .expect("set alpha");
    settings
        .set(
            SettingScope::Space,
            "rules.disabled",
            json!({"body": "DISABLED_BODY_SHOULD_NOT_APPEAR", "order": 100, "enabled": false, "attribution": null}),
        )
        .await
        .expect("set disabled");
    settings
        .set(SettingScope::Space, "editor.font", json!("mono"))
        .await
        .expect("set editor.font");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compile_injects_space_rules_into_context_content() {
    let fx = setup_space_rules_fixture().await;
    seed_three_rules(&fx).await;

    // Sanity-check: the rules actually persisted.
    let pre = ship_space::rules::list_space_rules(&fx.space_id).await.unwrap();
    assert_eq!(pre.len(), 2, "expected 2 enabled rules; got {pre:?}");

    let app = build_router();
    let (status, body) = call_compile(
        &app,
        serde_json::json!({
            "repoPath": fx.repo_canonical.to_string_lossy(),
            "harness": "claude",
            "activeAgent": "test-agent",
            "dryRun": true,
        }),
    )
    .await;
    assert_eq!(status, 200, "Compile must succeed; body={body}");
    let files = body["files"].as_array().expect("files array");
    let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap_or("")).collect();
    let claude_md = files
        .iter()
        .find(|f| f["path"] == "CLAUDE.md")
        .unwrap_or_else(|| panic!("CLAUDE.md missing; paths={paths:?}"));
    let content = claude_md["content"].as_str().expect("content string");

    assert!(
        content.contains("ALPHA_BODY_FROM_SPACE_RULE"),
        "CLAUDE.md must carry the alpha rule body; got:\n{content}"
    );
    assert!(
        content.contains("BRAVO_BODY_FROM_SPACE_RULE"),
        "CLAUDE.md must carry the bravo rule body; got:\n{content}"
    );
    assert!(
        !content.contains("DISABLED_BODY_SHOULD_NOT_APPEAR"),
        "disabled rule must NOT be in emitted context; got:\n{content}"
    );
    let alpha_pos = content.find("ALPHA_BODY_FROM_SPACE_RULE").expect("alpha present");
    let bravo_pos = content.find("BRAVO_BODY_FROM_SPACE_RULE").expect("bravo present");
    assert!(
        alpha_pos < bravo_pos,
        "alpha@{alpha_pos} (order=10) before bravo@{bravo_pos} (order=20)",
    );
    drop(app);
    drop(fx);
}
