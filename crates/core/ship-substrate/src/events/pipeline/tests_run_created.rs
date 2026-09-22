//! Round-trip tests for `PipelineRunStartedPayload` field wire contracts.
//!
//! Pins down the serde shape for fields that other layers (e.g. the gate
//! reviewer) read off the event log — without these guards, dropping or
//! renaming a field could silently regress gate behavior in production.

use super::*;

#[test]
fn source_spec_path_round_trips_when_present() {
    let json = r#"{
        "pipeline_run_id": "01XXX",
        "slug": "test",
        "agent": "rust-runtime",
        "branch": "job/test",
        "spec_path": ".artifacts/job-spec.md",
        "plan_id": null,
        "model": null,
        "harness": null,
        "source_spec_path": "/abs/path/to/spec.md"
    }"#;
    let p: PipelineRunStartedPayload = serde_json::from_str(json).unwrap();
    assert_eq!(p.source_spec_path.as_deref(), Some("/abs/path/to/spec.md"));
    // Round-trip through serialization preserves the field.
    let reserialized = serde_json::to_string(&p).unwrap();
    assert!(reserialized.contains("\"source_spec_path\":\"/abs/path/to/spec.md\""));
}

#[test]
fn baseline_sha_round_trips_when_present() {
    // Pin the wire contract for the field gates rely on. Both the
    // serialize-with-value and deserialize-from-value paths must hold
    // — without that, the reviewer step's gate can't see commits made
    // by earlier pipeline steps and fails with an empty diff range.
    let json = r#"{
        "pipeline_run_id": "01ZZZ",
        "slug": "test",
        "agent": "rust-runtime",
        "branch": "job/test",
        "spec_path": ".artifacts/job-spec.md",
        "plan_id": null,
        "model": null,
        "harness": null,
        "baseline_sha": "deadbeefcafe"
    }"#;
    let p: PipelineRunStartedPayload = serde_json::from_str(json).unwrap();
    assert_eq!(p.baseline_sha.as_deref(), Some("deadbeefcafe"));
    let reserialized = serde_json::to_string(&p).unwrap();
    assert!(reserialized.contains("\"baseline_sha\":\"deadbeefcafe\""));
}

#[test]
fn baseline_sha_absent_in_legacy_events() {
    // Legacy payloads must still deserialize and not emit the field
    // when None, so the event log doesn't grow explicit nulls on
    // every new write.
    let legacy_json = r#"{
        "pipeline_run_id": "01WWW",
        "slug": "old",
        "agent": "rust-runtime",
        "branch": "job/old",
        "spec_path": ".artifacts/job-spec.md",
        "plan_id": null,
        "model": null,
        "harness": null
    }"#;
    let p: PipelineRunStartedPayload = serde_json::from_str(legacy_json).unwrap();
    assert!(p.baseline_sha.is_none());
    let reserialized = serde_json::to_string(&p).unwrap();
    assert!(
        !reserialized.contains("baseline_sha"),
        "None must skip serializing, got: {reserialized}"
    );
}

#[test]
fn source_spec_path_absent_in_legacy_events() {
    // Legacy payloads predating this field must still deserialize — and the
    // field must serialize away when None (skip_serializing_if) so the
    // event log doesn't grow explicit nulls on every new write either.
    let legacy_json = r#"{
        "pipeline_run_id": "01YYY",
        "slug": "old",
        "agent": "rust-runtime",
        "branch": "job/old",
        "spec_path": ".artifacts/job-spec.md",
        "plan_id": null,
        "model": null,
        "harness": null
    }"#;
    let p: PipelineRunStartedPayload = serde_json::from_str(legacy_json).unwrap();
    assert!(p.source_spec_path.is_none());
    let reserialized = serde_json::to_string(&p).unwrap();
    assert!(
        !reserialized.contains("source_spec_path"),
        "None must skip serializing, got: {reserialized}"
    );
}
