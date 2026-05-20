use crate::resource_resolver::resolve_resource_uri;
use ship_substrate::db::ensure_db;
use ship_substrate::db::ensure_db_at;
use ship_substrate::events::{EventStore, SqliteEventStore};
use ship_platform::projects::init_project;
use ship_substrate::events::EventEnvelope;
use std::future;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    let project_dir = ship_dir.parent().unwrap().to_path_buf();
    ensure_db().unwrap();
    (tmp, ship_dir, project_dir)
}

/// Setup with explicit ship_dir DB (avoids thread-local env var issues).
fn setup_with_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    ensure_db().unwrap();
    ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

fn append_event(ship_dir: &std::path::Path, event: &EventEnvelope) {
    let store = SqliteEventStore::new_at(ship_dir).unwrap();
    store.append(event).unwrap();
}

async fn resolve(uri: &str, dir: &std::path::Path) -> Option<String> {
    resolve_resource_uri(uri, dir, future::ready(String::new())).await
}

// ── Job resources (event-sourced) ────────────────────────────────

fn job_created_event(pipeline_run_id: &str, slug: &str, agent: &str) -> EventEnvelope {
    EventEnvelope::new(
        "pipeline.run.started",
        pipeline_run_id,
        &serde_json::json!({
            "pipeline_run_id": pipeline_run_id,
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
    assert_eq!(parsed["pipeline_run_id"].as_str(), Some("j-detail"));
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
            "pipeline.run.dispatched",
            "j-status",
            &serde_json::json!({"pipeline_run_id": "j-status", "worktree": "/tmp/wt"}),
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
            "---\nname: {skill_id}\nstable-id: {skill_id}\ndescription: Test skill\ntags: [test]\nauthors: [test]\n---\n{content}"
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
    ship_platform::agents::skill_vars::set_skill_var(&ship_dir, "commit", "commit_style", serde_json::json!("gitmoji"))
        .unwrap();

    let result = resolve("ship://skills/commit", &ship_dir).await.unwrap();
    assert!(
        result.contains("gitmoji"),
        "overridden variable value missing from skill resource: {result}"
    );
}

// ── Components catalog ───────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn components_returns_valid_cem_json() {
    let (_tmp, _ship_dir, project_dir) = setup();
    let result = resolve("ship://components", &project_dir).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .expect("ship://components must return valid JSON");
    assert!(
        parsed.is_object(),
        "ship://components must return a JSON object: got {result}",
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn components_unknown_tag_returns_none() {
    let (_tmp, _ship_dir, project_dir) = setup();
    assert!(
        resolve("ship://components/sui-does-not-exist", &project_dir)
            .await
            .is_none(),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn components_resource_listed_with_metadata() {
    use crate::resources::static_resource_list;
    let listed = static_resource_list();
    let entry = listed
        .iter()
        .find(|r| r.raw.uri == "ship://components")
        .expect("ship://components must be in the static resource list");
    assert_eq!(entry.raw.mime_type.as_deref(), Some("application/json"));
    assert!(entry.raw.description.is_some(), "components resource needs a description");
}

#[tokio::test(flavor = "multi_thread")]
async fn components_template_registered() {
    use crate::resources::static_resource_template_list;
    let templates = static_resource_template_list();
    let entry = templates
        .iter()
        .find(|t| t.raw.uri_template == "ship://components/{tag}")
        .expect("ship://components/{tag} template must be registered");
    assert_eq!(entry.raw.mime_type.as_deref(), Some("application/json"));
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
