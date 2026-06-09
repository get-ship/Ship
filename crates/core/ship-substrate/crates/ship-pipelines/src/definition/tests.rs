//! Behavioral tests for the unified `PipelineDefinition` authoring contract.
//!
//! R18: behavior, not impl — round-trip serialize/parse, provider preservation,
//! lenient-read of legacy docs, and validation outcomes.

use super::*;

/// A fully-populated definition exercising metadata, per-step specs, a typed
/// provider override, and a gate.
fn rich_def() -> PipelineDefinition {
    let mut vars = HashMap::new();
    vars.insert("provider".to_string(), "claude".to_string());

    PipelineDefinition {
        name: Some("ship-dev".into()),
        description: Some("implement → verify → review".into()),
        vars,
        timeout: Some("30m".into()),
        max_retries: Some(2),
        auto_merge: true,
        steps: vec![
            PipelineStepDef {
                id: "implement".into(),
                step_type: Some(StepType::Agent),
                agent: Some("rust-lane".into()),
                goal: None,
                after: vec![],
                command: None,
                max_retries: None,
                gate: None,
                produces: vec![],
                consumes: vec![],
                promotes: vec![],
                fanout: None,
                spec: Some("# Implementer\n\nDo the thing.".into()),
                provider: ProviderOverride {
                    provider: Some("claude".into()),
                    model: Some("opus".into()),
                    harness: None,
                    invocation: Some(Invocation::Print),
                },
            },
            PipelineStepDef {
                id: "verify".into(),
                step_type: Some(StepType::Run),
                agent: None,
                goal: None,
                after: vec!["implement".into()],
                command: Some("cargo check --workspace".into()),
                max_retries: None,
                gate: None,
                produces: vec![],
                consumes: vec![],
                promotes: vec![],
                fanout: None,
                spec: Some("# Verify\n\nshell only".into()),
                provider: ProviderOverride::default(),
            },
        ],
        defaults: ProviderOverride {
            provider: Some("claude".into()),
            ..Default::default()
        },
    }
}

/// (a) Round-trip a `PipelineDefinition` through serialize → deserialize and
/// assert the data is unchanged.
#[test]
fn round_trip_preserves_definition() {
    let def = rich_def();
    let json = serde_json::to_string(&def).expect("serialize");
    let back: PipelineDefinition = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        serde_json::to_value(&def).unwrap(),
        serde_json::to_value(&back).unwrap(),
        "definition must survive a serialize/deserialize round trip"
    );
}

/// (d) The typed `provider` override is preserved (not dropped) through a
/// round trip — both at step level and as the pipeline-level default.
#[test]
fn provider_is_preserved_through_round_trip() {
    let def = rich_def();
    let json = serde_json::to_string(&def).expect("serialize");

    // Provider must appear on the wire (flattened, not nested).
    assert!(json.contains("\"provider\":\"claude\""), "wire: {json}");

    let back: PipelineDefinition = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        back.steps[0].provider.provider.as_deref(),
        Some("claude"),
        "step-level provider must survive the round trip"
    );
    assert_eq!(
        back.steps[0].provider.invocation,
        Some(Invocation::Print),
        "typed invocation must survive the round trip"
    );
    assert_eq!(
        back.defaults.provider.as_deref(),
        Some("claude"),
        "pipeline-level provider default must survive the round trip"
    );
}

/// A step authored with a flat `provider` key deserializes into the typed
/// field rather than being silently dropped (the old `deny_unknown_fields`-off
/// behavior).
#[test]
fn flat_provider_key_deserializes_into_typed_field() {
    let json = r#"{
        "name": "p",
        "steps": [ { "id": "impl", "agent": "rust-lane", "provider": "codex", "invocation": "stream" } ]
    }"#;
    let def: PipelineDefinition = serde_json::from_str(json).expect("parse");
    assert_eq!(def.steps[0].provider.provider.as_deref(), Some("codex"));
    assert_eq!(def.steps[0].provider.invocation, Some(Invocation::Stream));
    // type omitted → inferred as agent.
    assert_eq!(def.steps[0].resolved_type(), StepType::Agent);
}

/// (c) Lenient read: a legacy doc using the `command` wire key and a top-level
/// positional `step_specs` array parses without error. The legacy keys are
/// tolerated; the canonical homes are `run` and per-step `spec`.
#[test]
fn lenient_read_parses_legacy_command_fixture() {
    let legacy = r##"{
        "steps": [ { "id": "build", "type": "run", "command": "cargo build" } ],
        "step_specs": [ "# legacy positional spec" ]
    }"##;
    let def: PipelineDefinition =
        serde_json::from_str(legacy).expect("legacy doc must parse leniently");
    assert_eq!(def.steps.len(), 1);
    assert_eq!(def.steps[0].resolved_type(), StepType::Run);
    // Legacy `command` wire key is not the canonical `run` key, so it is
    // tolerated-but-ignored exactly as the runtime `PipelineStep` does.
    assert!(def.steps[0].command.is_none());
}

/// The `run` shorthand (no explicit `type`) infers a run step.
#[test]
fn run_shorthand_infers_run_type() {
    let json = r#"{ "steps": [ { "id": "build", "run": "cargo build" } ] }"#;
    let def: PipelineDefinition = serde_json::from_str(json).expect("parse");
    assert_eq!(def.steps[0].resolved_type(), StepType::Run);
    assert_eq!(def.steps[0].command.as_deref(), Some("cargo build"));
}

/// An invalid invocation value yields a clear deserialize error (unknown enum
/// variant) rather than being accepted.
#[test]
fn invalid_invocation_is_rejected() {
    let json =
        r#"{ "steps": [ { "id": "x", "agent": "a", "invocation": "bogus" } ] }"#;
    let err = serde_json::from_str::<PipelineDefinition>(json)
        .err()
        .expect("invalid invocation must be rejected");
    assert!(
        err.to_string().contains("invocation") || err.to_string().contains("variant"),
        "error should mention the invocation field, got: {err}"
    );
}

/// `validate()` accepts a well-formed definition and rejects one where an
/// agent step is missing its agent.
#[test]
fn validate_accepts_valid_rejects_missing_agent() {
    assert!(rich_def().validate().is_ok(), "rich def must validate");

    let json = r#"{ "steps": [ { "id": "x", "type": "agent" } ] }"#;
    let bad: PipelineDefinition = serde_json::from_str(json).unwrap();
    let err = bad.validate().expect_err("agent step without agent must fail");
    assert!(err.contains("agent"), "error should mention agent: {err}");
}
