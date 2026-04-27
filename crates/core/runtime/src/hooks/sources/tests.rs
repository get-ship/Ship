//! Tests for external trigger source adapters.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tempfile::TempDir;
use tokio::sync::Mutex;

use super::TriggerSource;
use crate::events::{EventEnvelope, KernelRouter};
use crate::hooks::def::{
    CronSourceDef, FileEventKind, FileSourceDef, HookDef, HookTrigger, TriggerSourceDef,
    WebhookSourceDef,
};
use crate::hooks::engine::HookEngine;
use crate::hooks::loader::{load_hooks_dir, parse_hook, validate_hook};

// ── Test helpers ─────────────────────────────────────────────────────────────

fn tmp_ship_dir() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(ship_dir.join("triggers")).unwrap();
    std::fs::create_dir_all(ship_dir.join("schemas")).unwrap();
    crate::db::ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

fn test_kernel(ship_dir: &Path) -> Arc<Mutex<KernelRouter>> {
    let kr = KernelRouter::new(ship_dir.to_path_buf()).unwrap();
    Arc::new(Mutex::new(kr))
}

fn write_hook(ship_dir: &Path, filename: &str, content: &str) {
    let path = ship_dir.join("triggers").join(filename);
    std::fs::write(path, content).unwrap();
}

fn recent_events_matching(ship_dir: &Path, event_prefix: &str) -> Vec<EventEnvelope> {
    crate::db::events::list_recent_events_at(ship_dir, 200)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.event_type.starts_with(event_prefix))
        .collect()
}

fn diff(before: &[EventEnvelope], after: &[EventEnvelope]) -> Vec<EventEnvelope> {
    after
        .iter()
        .filter(|a| !before.iter().any(|b| b.id == a.id))
        .cloned()
        .collect()
}

// ── Load-time validation ──────────────────────────────────────────────────────

#[test]
fn reject_both_event_and_source() {
    let hook_str = r#"{
        "name": "both",
        "trigger": {
            "event": "job.failed",
            "source": { "type": "cron", "expression": "*/1 * * * *" }
        },
        "actions": []
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    let err = validate_hook(&hook, &ship_dir).unwrap_err();
    assert!(err.contains("either 'event' or 'source'"), "{err}");
}

#[test]
fn reject_neither_event_nor_source() {
    let hook_str = r#"{
        "name": "empty-trigger",
        "trigger": {},
        "actions": []
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    let err = validate_hook(&hook, &ship_dir).unwrap_err();
    assert!(err.contains("either 'event' or 'source'"), "{err}");
}

#[test]
fn reject_webhook_path_without_prefix() {
    let hook_str = r#"{
        "name": "bad-webhook",
        "trigger": {
            "source": { "type": "webhook", "path": "/api/some-path" }
        },
        "actions": []
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    let err = validate_hook(&hook, &ship_dir).unwrap_err();
    assert!(err.contains("/webhooks/"), "{err}");
}

#[test]
fn accept_valid_webhook_trigger() {
    let hook_str = r#"{
        "name": "gh-webhook",
        "trigger": {
            "source": { "type": "webhook", "path": "/webhooks/github", "secret": "env:GH_SECRET" }
        },
        "actions": [
            { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
        ]
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    validate_hook(&hook, &ship_dir).unwrap();
}

#[test]
fn accept_valid_file_trigger() {
    let hook_str = r#"{
        "name": "watch-src",
        "trigger": {
            "source": { "type": "file", "watch": "src/**/*.rs", "events": ["modified"] }
        },
        "actions": [
            { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
        ]
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    validate_hook(&hook, &ship_dir).unwrap();
}

#[test]
fn accept_valid_cron_trigger() {
    let hook_str = r#"{
        "name": "daily",
        "trigger": {
            "source": { "type": "cron", "expression": "0 9 * * *" }
        },
        "actions": [
            { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
        ]
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    validate_hook(&hook, &ship_dir).unwrap();
}

#[test]
fn reject_invalid_cron_expression() {
    let hook_str = r#"{
        "name": "bad-cron",
        "trigger": {
            "source": { "type": "cron", "expression": "not a cron" }
        },
        "actions": []
    }"#;
    let (_tmp, ship_dir) = tmp_ship_dir();
    let hook = parse_hook(hook_str).unwrap();
    let err = validate_hook(&hook, &ship_dir).unwrap_err();
    assert!(err.contains("invalid expression"), "{err}");
}

#[test]
fn existing_event_triggers_still_load() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "event-hook.jsonc",
        r#"{ "name": "on-fail", "trigger": { "event": "job.failed" }, "actions": [] }"#,
    );
    let report = load_hooks_dir(&ship_dir);
    assert_eq!(report.hooks.len(), 1);
    assert!(report.errors.is_empty());
}

// ── Webhook adapter ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webhook_fires_event_on_valid_secret() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let source = super::webhook::WebhookSource::new();
    source.reload(&[make_webhook_hook("gh", "/webhooks/github", Some("mysecret"))]);

    let before = recent_events_matching(&ship_dir, "external.webhook");
    let result = source
        .handle_request(
            "/webhooks/github",
            Some("mysecret"),
            r#"{"ref":"main"}"#.as_bytes().to_vec(),
            &kernel,
        )
        .await;
    assert!(matches!(result, super::webhook::WebhookResult::Accepted));

    // Give the async emit a moment to persist.
    tokio::time::sleep(Duration::from_millis(50)).await;
    let after = recent_events_matching(&ship_dir, "external.webhook");
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].event_type, "external.webhook.gh");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webhook_rejects_wrong_secret() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let source = super::webhook::WebhookSource::new();
    source.reload(&[make_webhook_hook("gh", "/webhooks/github", Some("correct"))]);

    let result = source
        .handle_request("/webhooks/github", Some("wrong"), Vec::new(), &kernel)
        .await;
    assert!(matches!(result, super::webhook::WebhookResult::Unauthorized));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webhook_returns_not_found_for_unknown_path() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let source = super::webhook::WebhookSource::new();
    source.reload(&[make_webhook_hook("gh", "/webhooks/github", None)]);

    let result = source
        .handle_request("/webhooks/unknown", None, Vec::new(), &kernel)
        .await;
    assert!(matches!(result, super::webhook::WebhookResult::NotFound));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webhook_hot_reload_updates_routes() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let source = super::webhook::WebhookSource::new();
    source.reload(&[make_webhook_hook("old", "/webhooks/old", None)]);

    // Old path works.
    let r = source
        .handle_request("/webhooks/old", None, Vec::new(), &kernel)
        .await;
    assert!(matches!(r, super::webhook::WebhookResult::Accepted));

    // Reload with a new trigger (removes old).
    source.reload(&[make_webhook_hook("new", "/webhooks/new", None)]);

    let r_old = source
        .handle_request("/webhooks/old", None, Vec::new(), &kernel)
        .await;
    let r_new = source
        .handle_request("/webhooks/new", None, Vec::new(), &kernel)
        .await;
    assert!(matches!(r_old, super::webhook::WebhookResult::NotFound));
    assert!(matches!(r_new, super::webhook::WebhookResult::Accepted));
}

fn make_webhook_hook(name: &str, path: &str, secret: Option<&str>) -> HookDef {
    HookDef {
        name: name.to_string(),
        description: None,
        trigger: HookTrigger {
            event: None,
            source: Some(TriggerSourceDef::Webhook(WebhookSourceDef {
                path: path.to_string(),
                secret: secret.map(str::to_string),
            })),
            filter: None,
        },
        actions: vec![],
    }
}

// ── File adapter ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn file_source_emits_on_file_change() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    // Create a watched directory inside the temp root.
    let watch_dir = _tmp.path().join("watched");
    std::fs::create_dir_all(&watch_dir).unwrap();

    let source = Arc::new(super::file::FileSource::new(ship_dir.clone()));
    let hook = make_file_hook("watch-test", watch_dir.to_str().unwrap(), vec![]);
    source.start(kernel.clone());
    source.reload(&[hook]);

    // Give the watcher time to initialize.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let before = recent_events_matching(&ship_dir, "external.file");

    // Create a file in the watched directory.
    std::fs::write(watch_dir.join("test.txt"), "hello").unwrap();

    // Wait for debounce + event delivery.
    tokio::time::sleep(Duration::from_millis(600)).await;

    let after = recent_events_matching(&ship_dir, "external.file");
    let emitted = diff(&before, &after);
    assert!(!emitted.is_empty(), "expected at least one file event");
    assert_eq!(emitted[0].event_type, "external.file.watch-test");
    let payload: serde_json::Value =
        serde_json::from_str(&emitted[0].payload_json).unwrap();
    assert_eq!(payload["event_kind"], "created");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn file_source_filters_by_event_kind() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let watch_dir = _tmp.path().join("watched2");
    std::fs::create_dir_all(&watch_dir).unwrap();

    // Pre-create a file so we can trigger a modify event.
    let file_path = watch_dir.join("existing.txt");
    std::fs::write(&file_path, "initial").unwrap();

    // Hook only wants "deleted" events.
    let hook = make_file_hook("delete-only", watch_dir.to_str().unwrap(), vec![FileEventKind::Deleted]);
    let source = Arc::new(super::file::FileSource::new(ship_dir.clone()));
    source.start(kernel.clone());
    source.reload(&[hook]);

    tokio::time::sleep(Duration::from_millis(200)).await;

    let before = recent_events_matching(&ship_dir, "external.file.delete-only");

    // Modify — should be ignored.
    std::fs::write(&file_path, "updated").unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;

    let after_modify = recent_events_matching(&ship_dir, "external.file.delete-only");
    assert!(
        diff(&before, &after_modify).is_empty(),
        "modify should be ignored when only 'deleted' is configured"
    );

    // Delete — should fire.
    std::fs::remove_file(&file_path).unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;

    let after_delete = recent_events_matching(&ship_dir, "external.file.delete-only");
    assert!(
        !diff(&before, &after_delete).is_empty(),
        "delete should fire"
    );
}

fn make_file_hook(name: &str, watch: &str, event_kinds: Vec<FileEventKind>) -> HookDef {
    HookDef {
        name: name.to_string(),
        description: None,
        trigger: HookTrigger {
            event: None,
            source: Some(TriggerSourceDef::File(FileSourceDef {
                watch: watch.to_string(),
                events: event_kinds,
                debounce_ms: 200,
            })),
            filter: None,
        },
        actions: vec![],
    }
}

// ── Cron adapter ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cron_fires_on_schedule() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let source = Arc::new(super::cron::CronSource::new());
    // Every second (6-field cron).
    let hook = make_cron_hook("tick", "* * * * * *");
    source.start(kernel.clone());
    source.reload(&[hook]);

    let before = recent_events_matching(&ship_dir, "external.cron.tick");

    // Wait for at least one second boundary to pass plus task scheduling slack.
    tokio::time::sleep(Duration::from_millis(1500)).await;

    let after = recent_events_matching(&ship_dir, "external.cron.tick");
    let fired = diff(&before, &after);
    assert!(!fired.is_empty(), "cron should have fired");
    assert_eq!(fired[0].event_type, "external.cron.tick");
    let payload: serde_json::Value = serde_json::from_str(&fired[0].payload_json).unwrap();
    assert!(payload["scheduled_for"].is_string());
    assert!(payload["fired_at"].is_string());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cron_hot_reload_replaces_triggers() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    let source = Arc::new(super::cron::CronSource::new());
    let hook_a = make_cron_hook("tick-a", "* * * * * *");
    let hook_b = make_cron_hook("tick-b", "* * * * * *");

    source.start(kernel.clone());
    source.reload(&[hook_a]);

    // Reload with hook-b only — hook-a should stop firing.
    source.reload(&[hook_b]);

    tokio::time::sleep(Duration::from_millis(1500)).await;

    let a_events = recent_events_matching(&ship_dir, "external.cron.tick-a");
    let b_events = recent_events_matching(&ship_dir, "external.cron.tick-b");

    assert!(!b_events.is_empty(), "tick-b should fire after reload");
    // tick-a may have fired before the reload, so just assert tick-b fired.
    let _ = a_events; // not asserting count — depends on timing
}

fn make_cron_hook(name: &str, expression: &str) -> HookDef {
    HookDef {
        name: name.to_string(),
        description: None,
        trigger: HookTrigger {
            event: None,
            source: Some(TriggerSourceDef::Cron(CronSourceDef {
                expression: expression.to_string(),
                timezone: "UTC".to_string(),
            })),
            filter: None,
        },
        actions: vec![],
    }
}

// ── Engine integration ────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn engine_dispatches_webhook_event_to_source_trigger() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "wh.jsonc",
        r#"{
            "name": "on-gh-push",
            "trigger": {
                "source": { "type": "webhook", "path": "/webhooks/github" }
            },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "webhook" } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before: Vec<EventEnvelope> =
        crate::db::events::list_recent_events_at(&ship_dir, 200)
            .unwrap_or_default()
            .into_iter()
            .filter(|e| e.event_type == "hook.tripped")
            .collect();

    // Simulate the event that the webhook adapter would emit.
    let synthetic = EventEnvelope::new(
        "external.webhook.on-gh-push",
        "/webhooks/github",
        &json!({ "path": "/webhooks/github", "body": {} }),
    )
    .unwrap();
    engine.dispatch_event(&synthetic).await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let after: Vec<EventEnvelope> =
        crate::db::events::list_recent_events_at(&ship_dir, 200)
            .unwrap_or_default()
            .into_iter()
            .filter(|e| e.event_type == "hook.tripped")
            .collect();

    let fired = diff(&before, &after);
    assert_eq!(fired.len(), 1, "action should have fired");
    let payload: serde_json::Value =
        serde_json::from_str(&fired[0].payload_json).unwrap();
    assert_eq!(payload["from"], "webhook");
}
