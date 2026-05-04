//! Tests for event validators: namespace, elevated, and alignment.

use ship_types::envelope::EventEnvelope;
use crate::events::validator::*;

fn skill_ctx(skill_id: &str) -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: Some(skill_id.to_string()),
        project_id: None,
        session_id: None,
    }
}

fn caller_ctx(kind: CallerKind) -> EmitContext {
    EmitContext {
        caller_kind: kind,
        skill_id: None,
        project_id: None,
        session_id: None,
    }
}

fn make_event(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

// ── Namespace validator ─────────────────────────────────────────────────

#[test]
fn namespace_validator_allows_matching_skill() {
    let v = NamespaceValidator;
    let event = make_event("mcp-setup.server_ready");
    let ctx = skill_ctx("mcp-setup");
    assert!(v.validate(&event, &ctx).is_ok());
}

#[test]
fn namespace_validator_rejects_wrong_skill() {
    let v = NamespaceValidator;
    let event = make_event("visual-brainstorm.annotation");
    let ctx = skill_ctx("mcp-setup");
    let err = v.validate(&event, &ctx).unwrap_err();
    assert!(matches!(err, ValidationError::NamespaceViolation { .. }));
}

#[test]
fn namespace_validator_blocks_system_namespace() {
    let v = NamespaceValidator;
    let event = make_event("workspace.created");
    let ctx = skill_ctx("workspace");
    let err = v.validate(&event, &ctx).unwrap_err();
    assert!(matches!(err, ValidationError::ReservedNamespace { .. }));
}

#[test]
fn runtime_caller_can_emit_system_events() {
    let v = NamespaceValidator;
    let event = make_event("workspace.created");
    let ctx = caller_ctx(CallerKind::Runtime);
    assert!(v.validate(&event, &ctx).is_ok());
}

#[test]
fn cli_caller_can_emit_system_events() {
    let v = NamespaceValidator;
    let event = make_event("session.started");
    let ctx = caller_ctx(CallerKind::Cli);
    assert!(v.validate(&event, &ctx).is_ok());
}

// ── Namespace alignment ─────────────────────────────────────────────────

#[test]
fn namespace_lists_are_aligned() {
    let mut derived: Vec<&str> = KERNEL_NAMESPACES
        .iter()
        .chain(PLATFORM_ONLY_NAMESPACES.iter())
        .copied()
        .collect();
    derived.sort();
    let mut reserved: Vec<&str> = RESERVED_NAMESPACES.to_vec();
    reserved.sort();
    assert_eq!(
        derived, reserved,
        "RESERVED_NAMESPACES must equal KERNEL_NAMESPACES ∪ PLATFORM_ONLY_NAMESPACES"
    );
}

#[test]
fn kernel_and_platform_only_are_disjoint() {
    for ns in KERNEL_NAMESPACES {
        assert!(
            !PLATFORM_ONLY_NAMESPACES.contains(ns),
            "namespace '{ns}' appears in both KERNEL and PLATFORM_ONLY"
        );
    }
}

// ── Elevated validator ──────────────────────────────────────────────────

#[test]
fn elevated_from_untrusted_rejected() {
    let v = ElevatedValidator;
    let event = make_event("workspace.created").elevate();
    let ctx = caller_ctx(CallerKind::Mcp);
    let err = v.validate(&event, &ctx).unwrap_err();
    assert!(matches!(err, ValidationError::DirectionViolation { .. }));
}

#[test]
fn elevated_from_sdk_rejected() {
    let v = ElevatedValidator;
    let event = make_event("session.started").elevate();
    let ctx = caller_ctx(CallerKind::Sdk);
    let err = v.validate(&event, &ctx).unwrap_err();
    assert!(matches!(err, ValidationError::DirectionViolation { .. }));
}

#[test]
fn elevated_from_runtime_passes() {
    let v = ElevatedValidator;
    let event = make_event("workspace.created").elevate();
    let ctx = caller_ctx(CallerKind::Runtime);
    assert!(v.validate(&event, &ctx).is_ok());
}

#[test]
fn elevated_from_cli_passes() {
    let v = ElevatedValidator;
    let event = make_event("session.started").elevate();
    let ctx = caller_ctx(CallerKind::Cli);
    assert!(v.validate(&event, &ctx).is_ok());
}

#[test]
fn non_elevated_from_untrusted_passes() {
    let v = ElevatedValidator;
    let event = make_event("custom.event");
    let ctx = caller_ctx(CallerKind::Mcp);
    assert!(v.validate(&event, &ctx).is_ok());
}
