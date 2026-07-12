/// Descriptor conversion tests — `HookDef::try_from_def` validation.
///
/// Replaces the retired JSONC-loader tests (`integration_loader`,
/// `integration_subscription_validation`): the loader's file-scan/parse
/// concerns are gone (format-migration Wave 4 phase A), but the validation
/// it ran at load time — unknown-event rejection, cron expression parsing,
/// template_file existence, source triggers bypassing the event registry —
/// still applies to `stack_model::TriggerDef` → `HookDef` conversion and is
/// covered here directly.
use crate::def::{HookActionType, HookDef};

use super::integration_engine::{tmp_ship_dir, trigger_def};

#[test]
fn triggers_descriptor_valid_def_converts() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "notify-failed",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "hi": "there" } }
            ]
        }"#,
    );
    let hook = HookDef::try_from_def(&def, &ship_dir).expect("valid def converts");
    assert_eq!(hook.name, "notify-failed");
    assert_eq!(hook.trigger.event.as_deref(), Some("pipeline.run.started"));
    assert_eq!(hook.actions.len(), 1);
    assert!(matches!(
        hook.actions[0].action_type,
        HookActionType::EventEmit { .. }
    ));
}

// F2 regression: the old `log-failed-jobs` trigger subscribed to the
// nonexistent `job.failed` event type. Any event NOT in the registry is
// rejected at conversion time with a clear error.
#[test]
fn triggers_descriptor_unknown_event_rejected() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{ "id": "bogus", "trigger": { "event": "not.a.real.event" }, "actions": [] }"#,
    );
    let err = HookDef::try_from_def(&def, &ship_dir).expect_err("unknown event must be rejected");
    assert!(
        err.contains("unknown event type"),
        "error must mention 'unknown event type', got: {err}"
    );
}

// Explicit regression for F2: the legacy `job.failed` event no longer
// exists after the job→pipeline-run rename.
#[test]
fn triggers_descriptor_job_failed_event_rejected() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "log-failed-jobs",
            "trigger": { "event": "job.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
            ]
        }"#,
    );
    HookDef::try_from_def(&def, &ship_dir)
        .expect_err("job.failed is not a registered event and must be rejected");
}

#[test]
fn triggers_descriptor_registered_pipeline_events_pass() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let on_fail = trigger_def(
        r#"{ "id": "on-fail", "trigger": { "event": "pipeline.run.started" }, "actions": [] }"#,
    );
    let on_done = trigger_def(
        r#"{ "id": "on-done", "trigger": { "event": "pipeline.run.started" }, "actions": [] }"#,
    );
    HookDef::try_from_def(&on_fail, &ship_dir).expect("pipeline.run.started is registered");
    HookDef::try_from_def(&on_done, &ship_dir).expect("pipeline.run.started is registered");
}

// F3 added skill.changed / rule.changed emits — triggers subscribing to
// them must convert cleanly.
#[test]
fn triggers_descriptor_skill_changed_event_passes() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "on-skill-changed",
            "trigger": { "event": "skill.changed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "what": "skill" } }
            ]
        }"#,
    );
    HookDef::try_from_def(&def, &ship_dir).expect("skill.changed is registered (F3)");
}

#[test]
fn triggers_descriptor_rule_changed_event_passes() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "on-rule-changed",
            "trigger": { "event": "rule.changed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "what": "rule" } }
            ]
        }"#,
    );
    HookDef::try_from_def(&def, &ship_dir).expect("rule.changed is registered (F3)");
}

// Source-based triggers (cron/file/webhook) synthesize their own event
// type at dispatch time — the event-registry lookup doesn't apply to them.
#[test]
fn triggers_descriptor_source_trigger_bypasses_registry_lookup() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "daily",
            "trigger": { "event": "pipeline.run.started" },
            "source": { "kind": "cron", "schedule": "0 9 * * *" },
            "actions": []
        }"#,
    );
    let hook = HookDef::try_from_def(&def, &ship_dir)
        .expect("source trigger must convert without registry lookup");
    assert!(hook.trigger.is_source());
}

#[test]
fn triggers_descriptor_invalid_cron_expression_rejected() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "bad-cron",
            "trigger": { "event": "pipeline.run.started" },
            "source": { "kind": "cron", "schedule": "not a cron" },
            "actions": []
        }"#,
    );
    HookDef::try_from_def(&def, &ship_dir).expect_err("invalid cron expression must be rejected");
}

#[test]
fn triggers_descriptor_missing_template_file_returns_error() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{
            "id": "bad",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "collection.insert", "collection": "notes", "template_file": "does-not-exist.j2" }
            ]
        }"#,
    );
    let err = HookDef::try_from_def(&def, &ship_dir).expect_err("missing template_file must error");
    assert!(err.contains("template_file"), "got: {err}");
}

#[test]
fn triggers_descriptor_empty_id_rejected() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let def = trigger_def(
        r#"{ "id": "", "trigger": { "event": "pipeline.run.started" }, "actions": [] }"#,
    );
    HookDef::try_from_def(&def, &ship_dir).expect_err("empty id must be rejected");
}
