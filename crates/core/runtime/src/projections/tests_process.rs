//! Tests for the ProcessRecord projection.

use crate::events::EventEnvelope;
use crate::projections::process::*;

fn make_event(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, "test-slug", &payload).unwrap()
}

fn spawned_payload(slug: &str, pid: Option<u32>, kind: &str) -> serde_json::Value {
    serde_json::json!({
        "slug": slug,
        "job_id": "job-1",
        "pid": pid,
        "cmd": "claude",
        "argv": ["--print", "-p", "do stuff"],
        "env_keys": ["SHIP_ACTOR_ID"],
        "cwd": "/tmp/work",
        "executor_kind": kind,
        "step_id": null,
    })
}

#[test]
fn spawn_creates_running_record() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(1234), "process")),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, ProcessStatus::Running);
    assert_eq!(records[0].pid, Some(1234));
    assert_eq!(records[0].slug, "s1");
    assert_eq!(records[0].cmd, "claude");
}

#[test]
fn spawn_then_exited_transitions_to_exited() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(1234), "process")),
        make_event("process.exited", serde_json::json!({
            "slug": "s1", "pid": 1234, "exit_code": 0,
            "signal": null, "wall_duration_ms": 5000, "executor_kind": "process",
        })),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, ProcessStatus::Exited);
    assert_eq!(records[0].exit_code, Some(0));
    assert!(records[0].exited_at.is_some());
}

#[test]
fn spawn_signal_then_exited_preserves_signal_history() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(1234), "process")),
        make_event("process.signal_sent", serde_json::json!({
            "slug": "s1", "pid": 1234, "signum": "SIGTERM",
            "reason": "shutdown", "actor": "supervisor",
        })),
        make_event("process.exited", serde_json::json!({
            "slug": "s1", "pid": 1234, "exit_code": null,
            "signal": "SIGTERM", "wall_duration_ms": 12000, "executor_kind": "process",
        })),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, ProcessStatus::Exited);
    assert_eq!(records[0].signals_sent.len(), 1);
    assert_eq!(records[0].signals_sent[0].signum, "SIGTERM");
    assert_eq!(records[0].exit_signal, Some("SIGTERM".to_string()));
}

#[test]
fn spawn_then_killed_transitions_to_killed() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(1234), "process")),
        make_event("process.killed", serde_json::json!({
            "slug": "s1", "pid": 1234, "signal": "SIGKILL",
            "reason": "cancellation", "actor": "operator",
        })),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, ProcessStatus::Killed);
    assert_eq!(records[0].signals_sent.len(), 1);
}

#[test]
fn multiple_processes_per_slug() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(100), "process")),
        make_event("process.exited", serde_json::json!({
            "slug": "s1", "pid": 100, "exit_code": 1,
            "signal": null, "wall_duration_ms": 1000, "executor_kind": "process",
        })),
        make_event("process.spawned", spawned_payload("s1", Some(200), "process")),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].pid, Some(100));
    assert_eq!(records[0].status, ProcessStatus::Exited);
    assert_eq!(records[1].pid, Some(200));
    assert_eq!(records[1].status, ProcessStatus::Running);
}

#[test]
fn tmux_process_without_pid() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", None, "tmux")),
        make_event("process.exited", serde_json::json!({
            "slug": "s1", "pid": null, "exit_code": null,
            "signal": null, "wall_duration_ms": 60000, "executor_kind": "tmux",
        })),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].pid, None);
    assert_eq!(records[0].executor_kind, "tmux");
    assert_eq!(records[0].status, ProcessStatus::Exited);
}

#[test]
fn orphaned_status() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(999), "process")),
        make_event("process.orphaned", serde_json::json!({
            "pid": 999, "cmd": "claude",
            "last_known_slug": "s1", "detected_by": "sweep",
            "slug": "s1",
        })),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, ProcessStatus::Orphaned);
}

#[test]
fn non_process_events_are_ignored() {
    let events = vec![
        make_event("process.spawned", spawned_payload("s1", Some(1), "process")),
        EventEnvelope::new("job.created", "j1", &serde_json::json!({"job_id": "j1"})).unwrap(),
    ];
    let records = project(&events);
    assert_eq!(records.len(), 1);
}
