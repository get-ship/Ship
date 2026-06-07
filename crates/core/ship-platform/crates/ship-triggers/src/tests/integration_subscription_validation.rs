/// Subscription validation integration tests.
///
/// The trigger validator uses `event_registry::lookup()` to gate event-type
/// triggers. Tests cover:
/// - F2 regression: subscribing to a nonexistent event must fail cleanly
/// - Registered events (pipeline.run.failed, skill.changed, rule.changed) pass
/// - Source-based triggers bypass registry lookup entirely
use crate::loader;

use super::integration_engine::{tmp_ship_dir, write_hook};

// ── Registry lookup — event triggers ─────────────────────────────────────────

#[test]
fn triggers_subscription_validation_unknown_event_fails() {
    // F2 regression: the old `log-failed-jobs` trigger subscribed to the
    // nonexistent `job.failed` event type. Validate that any event NOT in the
    // registry is rejected at load time with a clear error.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "bad.jsonc",
        r#"{
            "name": "bogus",
            "trigger": { "event": "not.a.real.event" },
            "actions": []
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.hooks.is_empty(), "hook with unknown event must be rejected");
    assert_eq!(report.errors.len(), 1);
    assert!(
        report.errors[0].reason.contains("unknown event type"),
        "error must mention 'unknown event type', got: {}",
        report.errors[0].reason
    );
}

#[test]
fn triggers_subscription_validation_job_failed_event_rejected() {
    // Explicit regression for F2: the legacy `job.failed` event no longer
    // exists after the job→pipeline-run rename. Confirm it is rejected.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "log-failed-jobs.jsonc",
        r#"{
            "name": "log-failed-jobs",
            "trigger": { "event": "job.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(
        report.hooks.is_empty(),
        "job.failed is not a registered event and must be rejected"
    );
    assert!(
        !report.errors.is_empty(),
        "expected a validation error for job.failed"
    );
}

#[test]
fn triggers_subscription_validation_registered_pipeline_events_pass() {
    // pipeline.run.failed and pipeline.run.completed are registered events.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "on-fail.jsonc",
        r#"{ "name": "on-fail", "trigger": { "event": "pipeline.run.failed" }, "actions": [] }"#,
    );
    write_hook(
        &ship_dir,
        "on-done.jsonc",
        r#"{ "name": "on-done", "trigger": { "event": "pipeline.run.completed" }, "actions": [] }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(report.hooks.len(), 2, "both registered events must pass validation");
    assert!(report.errors.is_empty());
}

#[test]
fn triggers_subscription_validation_skill_changed_event_passes() {
    // F3 added skill.changed emit — triggers subscribing to it must validate.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "on-skill.jsonc",
        r#"{
            "name": "on-skill-changed",
            "trigger": { "event": "skill.changed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "what": "skill" } }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(
        report.hooks.len(),
        1,
        "skill.changed is registered (F3) and must pass; errors: {:?}",
        report.errors
    );
    assert!(report.errors.is_empty());
}

#[test]
fn triggers_subscription_validation_rule_changed_event_passes() {
    // F3 added rule.changed emit — triggers subscribing to it must validate.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "on-rule.jsonc",
        r#"{
            "name": "on-rule-changed",
            "trigger": { "event": "rule.changed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "what": "rule" } }
            ]
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(
        report.hooks.len(),
        1,
        "rule.changed is registered (F3) and must pass; errors: {:?}",
        report.errors
    );
    assert!(report.errors.is_empty());
}

#[test]
fn triggers_subscription_validation_source_trigger_bypasses_registry_lookup() {
    // Source-based triggers (cron/file/webhook) synthesize their own event
    // type at dispatch time — the registry lookup in validate_hook does NOT
    // apply to them. A cron trigger with no `event` field must load cleanly.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "daily.jsonc",
        r#"{
            "name": "daily",
            "trigger": { "source": { "type": "cron", "expression": "0 9 * * *" } },
            "actions": []
        }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert_eq!(
        report.hooks.len(),
        1,
        "source trigger must load without registry lookup; errors: {:?}",
        report.errors
    );
    assert!(report.errors.is_empty());
}

#[test]
fn triggers_subscription_validation_multiple_unknown_events_all_fail() {
    // Each bad hook produces its own LoadError — loader continues past all.
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "bad1.jsonc",
        r#"{ "name": "b1", "trigger": { "event": "nonexistent.one" }, "actions": [] }"#,
    );
    write_hook(
        &ship_dir,
        "bad2.jsonc",
        r#"{ "name": "b2", "trigger": { "event": "nonexistent.two" }, "actions": [] }"#,
    );
    let report = loader::load_triggers_dir(&ship_dir);
    assert!(report.hooks.is_empty());
    assert_eq!(report.errors.len(), 2);
    for err in &report.errors {
        assert!(
            err.reason.contains("unknown event type"),
            "each error must say 'unknown event type', got: {}",
            err.reason
        );
    }
}
