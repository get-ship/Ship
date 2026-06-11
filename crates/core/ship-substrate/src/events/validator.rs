//! Event validation — ingress checks before persistence.

use std::fmt;

use crate::events::EventEnvelope;
pub use ship_types::{CallerKind, EmitContext};

/// Reasons an event can be rejected before persistence.
#[derive(Debug)]
pub enum ValidationError {
    NamespaceViolation { expected: String, got: String },
    ReservedNamespace { namespace: String },
    DirectionViolation { event_type: String, direction: String },
    SchemaViolation { event_type: String, error: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamespaceViolation { expected, got } => {
                write!(f, "namespace violation: expected '{expected}.*', got '{got}'")
            }
            Self::ReservedNamespace { namespace } => {
                write!(f, "reserved namespace: '{namespace}'")
            }
            Self::DirectionViolation {
                event_type,
                direction,
            } => {
                write!(f, "direction violation: '{event_type}' cannot be emitted {direction}")
            }
            Self::SchemaViolation { event_type, error } => {
                write!(f, "schema violation for '{event_type}': {error}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates an event before it is persisted.
pub trait EventValidator: Send + Sync {
    fn validate(&self, event: &EventEnvelope, ctx: &EmitContext)
        -> Result<(), ValidationError>;
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Namespaces routed through the kernel to actor mailboxes.
/// Actors can subscribe to events in these namespaces.
///
/// NOTE: routing ≠ ownership. Several namespaces here are OWNED at L1 by a
/// domain crate (their `register_event!` lines live there, not in
/// `payload_registry.rs`), but they remain in this kernel-routing list
/// because the kernel still routes those events to subscribed actors. The
/// inline comments below mark L1-owned namespaces so this distinction stays
/// explicit.
pub const KERNEL_NAMESPACES: &[&str] = &[
    "actor.",
    "collection.",
    "config.",
    "daemon.",
    "gate.",
    "pipeline.",
    "mesh.",
    "runtime.",
    "session.",
    "settings.",
    "studio.", // L1-owned: registrations in ship-agents/src/event_registrations.rs
    "supervisor.",
    "sync.",      // L1-owned: registrations in ship-cloud/src/event_registrations.rs
    "workspace.", // L1-owned: registrations in ship-space/src/event_registrations.rs
];

/// Platform-only namespaces: persisted in platform.db but NOT routed to actors.
/// These are UI/config events that actors don't need.
pub const PLATFORM_ONLY_NAMESPACES: &[&str] = &[
    "project.",
    "skill.",
];

/// All reserved namespaces (kernel + platform-only). Untrusted callers cannot
/// emit events with these prefixes. Must equal KERNEL_NAMESPACES ∪
/// PLATFORM_ONLY_NAMESPACES — enforced by `namespace_lists_are_aligned` test.
pub const RESERVED_NAMESPACES: &[&str] = &[
    "actor.",
    "collection.",
    "config.",
    "daemon.",
    "gate.",
    "pipeline.",
    "mesh.",
    "project.",
    "runtime.",
    "session.",
    "settings.",
    "skill.",
    "studio.",
    "supervisor.",
    "sync.",
    "workspace.",
];

fn is_system_event(event_type: &str) -> bool {
    KERNEL_NAMESPACES
        .iter()
        .chain(PLATFORM_ONLY_NAMESPACES.iter())
        .any(|ns| event_type.starts_with(ns))
}

/// Returns true if `event_type` belongs to a kernel-routed namespace.
pub fn is_kernel_event(event_type: &str) -> bool {
    KERNEL_NAMESPACES
        .iter()
        .any(|ns| event_type.starts_with(ns))
}

fn is_trusted_caller(kind: &CallerKind) -> bool {
    matches!(kind, CallerKind::Runtime | CallerKind::Cli)
}

// ── Built-in validators ──────────────────────────────────────────────────────

/// Skills can only emit events in their own namespace (`{skill_id}.*`).
/// System namespaces are always reserved.
pub struct NamespaceValidator;

impl EventValidator for NamespaceValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if is_trusted_caller(&ctx.caller_kind) {
            return Ok(());
        }
        if let Some(ref skill_id) = ctx.skill_id {
            let prefix = format!("{skill_id}.");
            if !event.event_type.starts_with(&prefix) {
                return Err(ValidationError::NamespaceViolation {
                    expected: skill_id.clone(),
                    got: event.event_type.clone(),
                });
            }
            if is_system_event(&event.event_type) {
                return Err(ValidationError::ReservedNamespace {
                    namespace: event.event_type.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Blocks non-trusted callers from emitting system-namespace events.
pub struct ReservedNamespaceValidator;

impl EventValidator for ReservedNamespaceValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if is_trusted_caller(&ctx.caller_kind) {
            return Ok(());
        }
        if is_system_event(&event.event_type) {
            return Err(ValidationError::ReservedNamespace {
                namespace: event.event_type.clone(),
            });
        }
        Ok(())
    }
}

/// Rejects elevated events from untrusted callers.
pub struct ElevatedValidator;

impl EventValidator for ElevatedValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if event.elevated && !is_trusted_caller(&ctx.caller_kind) {
            return Err(ValidationError::DirectionViolation {
                event_type: event.event_type.clone(),
                direction: "elevated by untrusted caller".to_string(),
            });
        }
        Ok(())
    }
}

