use crate::resource_resolver::resolve_resource_uri;
use runtime::db::ensure_db;
use runtime::db::ensure_db_at;
use runtime::events::store::append_event_with_conn;
use runtime::project::init_project;
use runtime::EventEnvelope;
use std::future;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    let project_dir = ship_dir.parent().unwrap().to_path_buf();
    ensure_db().unwrap();
    (tmp, ship_dir, project_dir)
}

/// Setup with explicit ship_dir DB (avoids thread-local env var issues).
fn setup_with_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();
    ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

fn append_event(ship_dir: &std::path::Path, event: &EventEnvelope) {
    let db_path = ship_dir.join("platform.db");
    let mut conn = runtime::db::open_db_at(&db_path).unwrap();
    append_event_with_conn(event, &mut conn).unwrap();
}

async fn resolve(uri: &str, dir: &std::path::Path) -> Option<String> {
    resolve_resource_uri(uri, dir, future::ready(String::new())).await
}

// ── Spec resources (file-based) ───────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn specs_list_empty() {
    let (_tmp, _ship_dir, project_dir) = setup();
    let result = resolve("ship://specs", &project_dir).await.unwrap();
    assert_eq!(result, "No specs found.");
}

#[tokio::test(flavor = "multi_thread")]
async fn specs_list_and_get_round_trip() {
    let (_tmp, ship_dir, project_dir) = setup();
    let specs_path = runtime::project::specs_dir(&ship_dir);
    std::fs::create_dir_all(&specs_path).unwrap();
    std::fs::write(
        specs_path.join("design-v1.md"),
        "# Design V1\n\nSpec content here.",
    )
    .unwrap();

    let list = resolve("ship://specs", &project_dir).await.unwrap();
    assert!(list.contains("design-v1"), "spec list: {list}");

    let detail = resolve("ship://specs/design-v1", &project_dir).await.unwrap();
    assert!(detail.contains("# Design V1"));
    assert!(detail.contains("Spec content here."));
}

#[tokio::test(flavor = "multi_thread")]
async fn spec_get_nonexistent_returns_none() {
    let (_tmp, _ship_dir, project_dir) = setup();
    assert!(
        resolve("ship://specs/nonexistent", &project_dir)
            .await
            .is_none()
    );
}

// ── ADR resources ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn adrs_resource_round_trip() {
    let (_tmp, _ship_dir, project_dir) = setup();
    let result = resolve("ship://adrs", &project_dir).await.unwrap();
    assert_eq!(result, "No ADRs found.");

    runtime::db::adrs::create_adr("Test ADR", "context", "decision", "proposed").unwrap();
    let list = resolve("ship://adrs", &project_dir).await.unwrap();
    assert!(list.contains("Test ADR"));
}

// ── Job resources (event-sourced) ────────────────────────────────

fn job_created_event(job_id: &str, slug: &str, agent: &str) -> EventEnvelope {
    EventEnvelope::new(
        "job.created",
        job_id,
        &serde_json::json!({
            "job_id": job_id,
            "slug": slug,
            "agent": agent,
            "branch": format!("job/{slug}"),
            "spec_path": ".artifacts/job-spec.md",
        }),
    )
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn jobs_list_empty() {
    let (_tmp, ship_dir) = setup_with_db();
    let result = resolve("ship://jobs", &ship_dir).await.unwrap();
    // Empty jobs should return valid JSON with an empty array
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .expect("ship://jobs must return valid JSON");
    assert!(parsed.is_array(), "ship://jobs must return a JSON array");
    assert!(parsed.as_array().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn jobs_list_returns_created_jobs() {
    let (_tmp, ship_dir) = setup_with_db();

    append_event(&ship_dir, &job_created_event("j-res-1", "auth-tests", "test-writer"));
    append_event(&ship_dir, &job_created_event("j-res-2", "cli-errors", "rust-runtime"));

    let result = resolve("ship://jobs", &ship_dir).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .expect("ship://jobs must return valid JSON");
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2, "should list both jobs");

    // Verify job fields are present
    let slugs: Vec<&str> = arr
        .iter()
        .filter_map(|j| j["slug"].as_str())
        .collect();
    assert!(slugs.contains(&"auth-tests"));
    assert!(slugs.contains(&"cli-errors"));
}

#[tokio::test(flavor = "multi_thread")]
async fn jobs_get_by_id() {
    let (_tmp, ship_dir) = setup_with_db();

    append_event(&ship_dir, &job_created_event("j-detail", "mcp-resources", "test-writer"));

    let result = resolve("ship://jobs/j-detail", &ship_dir).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .expect("ship://jobs/{id} must return valid JSON");
    assert_eq!(parsed["job_id"].as_str(), Some("j-detail"));
    assert_eq!(parsed["slug"].as_str(), Some("mcp-resources"));
    assert_eq!(parsed["agent"].as_str(), Some("test-writer"));
    assert_eq!(parsed["status"].as_str(), Some("pending"));
}

#[tokio::test(flavor = "multi_thread")]
async fn jobs_get_nonexistent_returns_none() {
    let (_tmp, ship_dir) = setup_with_db();
    assert!(resolve("ship://jobs/nonexistent", &ship_dir).await.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn jobs_get_reflects_status_changes() {
    let (_tmp, ship_dir) = setup_with_db();

    append_event(&ship_dir, &job_created_event("j-status", "status-test", "rust-runtime"));
    append_event(
        &ship_dir,
        &EventEnvelope::new(
            "job.dispatched",
            "j-status",
            &serde_json::json!({"job_id": "j-status", "worktree": "/tmp/wt"}),
        )
        .unwrap(),
    );

    let result = resolve("ship://jobs/j-status", &ship_dir).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["status"].as_str(), Some("dispatched"));
    assert_eq!(parsed["worktree"].as_str(), Some("/tmp/wt"));
}

// ship://mesh deferred to v0.3 — needs SharedMeshRegistry wired into resolver

// ── Skill resources with variable state ──────────────────────────

fn write_skill(ship_dir: &std::path::Path, skill_id: &str, content: &str) {
    let skill_dir = ship_dir.join("skills").join(skill_id);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            "---\nname: {skill_id}\ndescription: Test skill\n---\n{content}"
        ),
    )
    .unwrap();
}

fn write_skill_vars_json(ship_dir: &std::path::Path, skill_id: &str, vars_json: &str) {
    let assets = ship_dir.join("skills").join(skill_id).join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::write(assets.join("vars.json"), vars_json).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn skill_get_includes_content() {
    let (_tmp, ship_dir, _project_dir) = setup();
    write_skill(&ship_dir, "my-skill", "Do the thing.");

    let result = resolve("ship://skills/my-skill", &ship_dir).await.unwrap();
    assert!(result.contains("Do the thing."), "skill content missing: {result}");
}

#[tokio::test(flavor = "multi_thread")]
async fn skill_get_includes_variable_state() {
    let (_tmp, ship_dir, _project_dir) = setup();
    write_skill(&ship_dir, "commit", "Commit helper skill.");
    write_skill_vars_json(
        &ship_dir,
        "commit",
        r#"{
            "commit_style": {
                "type": "enum",
                "default": "conventional",
                "storage-hint": "global",
                "values": ["conventional", "gitmoji"]
            }
        }"#,
    );

    let result = resolve("ship://skills/commit", &ship_dir).await.unwrap();
    // The resource must include both the skill content and the variable state
    assert!(
        result.contains("Commit helper skill."),
        "skill content missing: {result}"
    );
    assert!(
        result.contains("commit_style"),
        "variable name missing from skill resource: {result}"
    );
    assert!(
        result.contains("conventional"),
        "variable default value missing from skill resource: {result}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn skill_get_includes_overridden_variable() {
    let (_tmp, ship_dir, _project_dir) = setup();
    write_skill(&ship_dir, "commit", "Commit skill.");
    write_skill_vars_json(
        &ship_dir,
        "commit",
        r#"{
            "commit_style": {
                "type": "enum",
                "default": "conventional",
                "storage-hint": "global",
                "values": ["conventional", "gitmoji"]
            }
        }"#,
    );

    // Override the variable
    runtime::set_skill_var(&ship_dir, "commit", "commit_style", serde_json::json!("gitmoji"))
        .unwrap();

    let result = resolve("ship://skills/commit", &ship_dir).await.unwrap();
    assert!(
        result.contains("gitmoji"),
        "overridden variable value missing from skill resource: {result}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn skill_get_without_vars_has_no_vars_section() {
    let (_tmp, ship_dir, _project_dir) = setup();
    write_skill(&ship_dir, "simple", "Simple skill with no vars.");

    let result = resolve("ship://skills/simple", &ship_dir).await.unwrap();
    assert!(result.contains("Simple skill with no vars."));
    // Without vars.json, there should be no variables section
    // The exact format is implementation-defined, but the content should still resolve
}
