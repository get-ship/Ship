//! Unit tests for the trigger engine.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::Mutex;

use crate::events::{EventEnvelope, KernelRouter};

use super::def::ActionKind;
use super::engine::{TriggerEngine, MAX_TRIGGER_DEPTH};
use super::{filter, loader};

fn tmp_ship_dir() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(ship_dir.join("triggers")).unwrap();
    std::fs::create_dir_all(ship_dir.join("schemas")).unwrap();
    crate::db::ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

fn test_kernel(ship_dir: &Path) -> Arc<Mutex<KernelRouter>> {
    let kr = KernelRouter::new(ship_dir.to_path_buf()).unwrap();
    Arc::new(Mutex::new(kr))
}

fn write_trigger(ship_dir: &Path, filename: &str, content: &str) {
    let path = ship_dir.join("triggers").join(filename);
    std::fs::write(path, content).unwrap();
}

fn write_schema(ship_dir: &Path, id: &str) {
    let schema = format!(
        r#"{{
            "id": "{id}",
            "fields": {{
                "title": {{ "type": "string" }},
                "body": {{ "type": "markdown" }},
                "status": {{ "type": "string" }}
            }}
        }}"#
    );
    std::fs::write(ship_dir.join("schemas").join(format!("{id}.jsonc")), schema).unwrap();
}

fn job_failed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "job.failed",
        "job-1",
        &json!({ "job_id": "job-1", "error": "boom", "status": "failed" }),
    )
    .unwrap()
}

fn job_completed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "job.completed",
        "job-1",
        &json!({ "job_id": "job-1", "slug": "job-1", "status": "ok" }),
    )
    .unwrap()
}

// ── loader ──────────────────────────────────────────────────────────────────

#[test]
fn loader_parses_valid_trigger() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(
        &ship_dir,
        "notify.jsonc",
        r#"{
            // notify-on-failed-job
            "name": "notify-failed",
            "trigger": { "event": "job.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "trigger.tripped", "payload": { "hi": "there" } }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.triggers.len(), 1);
    assert!(report.errors.is_empty());
    let trigger = &report.triggers[0];
    assert_eq!(trigger.name, "notify-failed");
    assert_eq!(trigger.trigger.event, "job.failed");
    assert_eq!(trigger.actions.len(), 1);
    matches!(trigger.actions[0].action_type, ActionKind::EventEmit { .. });
}

#[test]
fn loader_rejects_unknown_event() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(
        &ship_dir,
        "bad.jsonc",
        r#"{
            "name": "bogus",
            "trigger": { "event": "not.a.real.event" },
            "actions": []
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.triggers.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].reason.contains("unknown event type"));
}

#[test]
fn loader_rejects_missing_template_file() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(
        &ship_dir,
        "bad-tmpl.jsonc",
        r#"{
            "name": "bad",
            "trigger": { "event": "job.completed" },
            "actions": [
                { "type": "collection.insert", "collection": "notes", "template_file": "does-not-exist.j2" }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.triggers.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].reason.contains("template_file"));
}

#[test]
fn loader_continues_past_bad_trigger() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "good.jsonc", r#"{
        "name": "good", "trigger": { "event": "job.completed" }, "actions": []
    }"#);
    write_trigger(&ship_dir, "bad.jsonc", "not valid jsonc {");
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.triggers.len(), 1);
    assert_eq!(report.errors.len(), 1);
}

// ── filter ──────────────────────────────────────────────────────────────────

#[test]
fn filter_matches_dot_paths() {
    let env = job_failed_envelope();
    let ctx = filter::build_event_context(&env);
    assert!(filter::matches(
        Some(&json!({ "payload.status": "failed" })),
        &ctx
    ));
    assert!(!filter::matches(
        Some(&json!({ "payload.status": "ok" })),
        &ctx
    ));
}

// ── engine ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_matching_runs_matching_trigger_only() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "fail.jsonc", r#"{
        "name": "on-fail",
        "trigger": { "event": "job.failed" },
        "actions": [
            { "type": "event.emit", "event_type": "trigger.tripped", "payload": { "from": "fail" } }
        ]
    }"#);
    write_trigger(&ship_dir, "done.jsonc", r#"{
        "name": "on-done",
        "trigger": { "event": "job.completed" },
        "actions": [
            { "type": "event.emit", "event_type": "trigger.tripped", "payload": { "from": "done" } }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 2);

    let before = recent_trigger_events(&ship_dir);
    engine.dispatch_event(&job_failed_envelope()).await;
    let after = recent_trigger_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 1, "only the on-fail trigger fires");
    assert_eq!(emitted[0].event_type, "trigger.tripped");
    let payload: Value = serde_json::from_str(&emitted[0].payload_json).unwrap();
    assert_eq!(payload["from"], "fail");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filter_excludes_non_matching_payload() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "only-failed.jsonc", r#"{
        "name": "only-failed",
        "trigger": {
            "event": "job.completed",
            "filter": { "payload.status": "failed" }
        },
        "actions": [
            { "type": "event.emit", "event_type": "trigger.tripped", "payload": {} }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_trigger_events(&ship_dir);
    engine.dispatch_event(&job_completed_envelope()).await;
    let after = recent_trigger_events(&ship_dir);
    assert!(diff(&before, &after).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jinja_renders_event_fields() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "mirror.jsonc", r#"{
        "name": "mirror",
        "trigger": { "event": "job.failed" },
        "actions": [
            { "type": "event.emit", "event_type": "trigger.tripped",
              "payload": { "entity": "{{ event.entity_id }}" } }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_trigger_events(&ship_dir);
    engine.dispatch_event(&job_failed_envelope()).await;
    let after = recent_trigger_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 1);
    let payload: Value = serde_json::from_str(&emitted[0].payload_json).unwrap();
    assert_eq!(payload["entity"], "job-1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn when_condition_skips_action() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "skip.jsonc", r#"{
        "name": "skip",
        "trigger": { "event": "job.failed" },
        "actions": [
            { "type": "event.emit", "event_type": "trigger.tripped", "when": "1 == 2",
              "payload": {} }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_trigger_events(&ship_dir);
    engine.dispatch_event(&job_failed_envelope()).await;
    let after = recent_trigger_events(&ship_dir);
    assert!(diff(&before, &after).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn results_chain_across_actions() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "chain.jsonc", r#"{
        "name": "chain",
        "trigger": { "event": "job.failed" },
        "actions": [
            { "type": "event.emit", "event_type": "trigger.tripped", "payload": { "step": 1 } },
            { "type": "event.emit", "event_type": "trigger.tripped",
              "payload": { "prev_event_id": "{{ results[0].event_id }}" } }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_trigger_events(&ship_dir);
    engine.dispatch_event(&job_failed_envelope()).await;
    let after = recent_trigger_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 2);
    let first_id = &emitted[0].id;
    let second_payload: Value = serde_json::from_str(&emitted[1].payload_json).unwrap();
    assert_eq!(second_payload["prev_event_id"], Value::String(first_id.clone()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn template_file_is_rendered() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_schema(&ship_dir, "notes");
    std::fs::create_dir_all(ship_dir.join("triggers/templates")).unwrap();
    std::fs::write(
        ship_dir.join("triggers/templates/note.j2"),
        r#"{ "title": "{{ event.entity_id }}", "body": "failed", "status": "open" }"#,
    )
    .unwrap();
    write_trigger(&ship_dir, "record.jsonc", r#"{
        "name": "record",
        "trigger": { "event": "job.failed" },
        "actions": [
            { "type": "collection.insert",
              "collection": "notes",
              "template_file": "triggers/templates/note.j2" }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 1);

    engine.dispatch_event(&job_failed_envelope()).await;

    let mut svc = crate::collections::store::open_docs_db(&ship_dir).unwrap();
    let schema = crate::collections::schema::load_schemas(&ship_dir)
        .unwrap()
        .into_iter()
        .find(|s| s.id == "notes")
        .unwrap();
    let items = crate::collections::store::query_items(
        &mut svc,
        &schema,
        &crate::collections::query::QueryRequest {
            collection: "notes".to_string(),
            filter: Default::default(),
            sort: Default::default(),
            limit: 50,
            offset: 0,
        },
    )
    .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].data["title"], "job-1");
    assert_eq!(items[0].data["body"], "failed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recursion_guard_blocks_deep_triggers() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_trigger(&ship_dir, "loop.jsonc", r#"{
        "name": "loop",
        "trigger": { "event": "job.failed" },
        "actions": [
            { "type": "event.emit", "event_type": "job.failed", "payload": {
                "job_id": "child", "error": "x", "status": "failed"
            } }
        ]
    }"#);

    let kernel = test_kernel(&ship_dir);
    let engine = TriggerEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let mut envelope = job_failed_envelope();
    let mut payload: Value = serde_json::from_str(&envelope.payload_json).unwrap();
    payload["_trigger_depth"] = Value::from(MAX_TRIGGER_DEPTH);
    envelope.payload_json = payload.to_string();

    let before = recent_trigger_events(&ship_dir);
    engine.dispatch_event(&envelope).await;
    let after = recent_trigger_events(&ship_dir);
    assert!(diff(&before, &after).is_empty(), "max depth must halt");
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn recent_trigger_events(ship_dir: &Path) -> Vec<EventEnvelope> {
    let all =
        crate::db::events::list_recent_events_at(ship_dir, 200).unwrap_or_default();
    all.into_iter().filter(|e| e.event_type == "trigger.tripped").collect()
}

fn diff(before: &[EventEnvelope], after: &[EventEnvelope]) -> Vec<EventEnvelope> {
    after
        .iter()
        .filter(|a| !before.iter().any(|b| b.id == a.id))
        .cloned()
        .collect()
}
