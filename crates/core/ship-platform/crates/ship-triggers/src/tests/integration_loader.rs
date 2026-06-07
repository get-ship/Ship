/// Loader integration tests — JSONC parsing, validation, error recovery.
///
/// Covers: parse_hook roundtrip, validate_hook gating, load_triggers_dir error
/// accumulation, legacy-hooks-dir isolation, and extension filtering.
use crate::def::HookActionType;
use crate::loader;

use super::integration_engine::{tmp_ship_dir, write_hook};

// ── parse / validate round-trip ──────────────────────────────────────────────

#[test]
fn triggers_loader_parse_valid_hook_succeeds() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "notify.jsonc",
        r#"{
            // notify-on-failed-job
            "name": "notify-failed",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "hi": "there" } }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.hooks.len(), 1);
    assert!(report.errors.is_empty());
    let hook = &report.hooks[0];
    assert_eq!(hook.name, "notify-failed");
    assert_eq!(hook.trigger.event.as_deref(), Some("pipeline.run.failed"));
    assert_eq!(hook.actions.len(), 1);
    assert!(matches!(
        hook.actions[0].action_type,
        HookActionType::EventEmit { .. }
    ));
}

#[test]
fn triggers_loader_parse_invalid_json_returns_error() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(&ship_dir, "bad.jsonc", "not valid jsonc {");
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.hooks.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].reason.contains("parse failed"));
}

#[test]
fn triggers_loader_continues_past_bad_hook() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "good.jsonc",
        r#"{ "name": "good", "trigger": { "event": "pipeline.run.completed" }, "actions": [] }"#,
    );
    write_hook(&ship_dir, "bad.jsonc", "not valid jsonc {");
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.hooks.len(), 1);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.hooks[0].name, "good");
}

#[test]
fn triggers_loader_ignores_legacy_hooks_dir() {
    // Regression for the .ship/hooks/ → .ship/triggers/ rename: loader must
    // read only from `.ship/triggers/` and NOT silently double-read a
    // legacy `.ship/hooks/` sibling.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "in-triggers.jsonc",
        r#"{ "name": "in-triggers", "trigger": { "event": "pipeline.run.completed" }, "actions": [] }"#,
    );
    let legacy = ship_dir.join("hooks");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(
        legacy.join("in-hooks.jsonc"),
        r#"{ "name": "in-hooks", "trigger": { "event": "pipeline.run.completed" }, "actions": [] }"#,
    )
    .unwrap();

    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.hooks.len(), 1, "only the triggers/ entry loads");
    assert_eq!(report.hooks[0].name, "in-triggers");
}

#[test]
fn triggers_loader_ignores_non_jsonc_files() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    // .yaml and .toml files must be ignored.
    std::fs::write(
        ship_dir.join("triggers").join("ignored.yaml"),
        "name: bad\n",
    )
    .unwrap();
    std::fs::write(
        ship_dir.join("triggers").join("ignored.toml"),
        r#"name = "bad""#,
    )
    .unwrap();
    write_hook(
        &ship_dir,
        "valid.jsonc",
        r#"{ "name": "valid", "trigger": { "event": "pipeline.run.completed" }, "actions": [] }"#,
    );

    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.hooks.len(), 1);
    assert_eq!(report.hooks[0].name, "valid");
}

#[test]
fn triggers_loader_empty_triggers_dir_returns_empty_report() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.hooks.is_empty());
    assert!(report.errors.is_empty());
}

#[test]
fn triggers_loader_missing_template_file_returns_error() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "bad-tmpl.jsonc",
        r#"{
            "name": "bad",
            "trigger": { "event": "pipeline.run.completed" },
            "actions": [
                { "type": "collection.insert", "collection": "notes", "template_file": "does-not-exist.j2" }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.hooks.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].reason.contains("template_file"));
}
