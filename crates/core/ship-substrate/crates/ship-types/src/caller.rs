use serde::{Deserialize, Serialize};

use agent_schema::Authority;

use crate::binding::ScopeBinding;
use crate::identity::SpaceId;

/// Who is emitting the event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerKind {
    Cli,
    Mcp,
    Sdk,
    CloudSync,
    Runtime,
}

impl CallerKind {
    /// Returns a stable, snake_case string representation suitable for
    /// protocol bodies and test assertions — independent of the Debug format.
    pub fn as_str(&self) -> &'static str {
        match self {
            CallerKind::Cli => "cli",
            CallerKind::Mcp => "mcp",
            CallerKind::Sdk => "sdk",
            CallerKind::CloudSync => "cloud_sync",
            CallerKind::Runtime => "runtime",
        }
    }
}

/// Context provided alongside every emit call.
///
/// Carries the authoritative [`ScopeBinding`] minted by the auth seam (sessions)
/// or `ScopeBinding::daemon_internal` (background tasks) — never a loose
/// `space_id: Option<String>`.
pub struct EmitContext {
    pub caller_kind: CallerKind,
    pub skill_id: Option<String>,
    pub binding: ScopeBinding,
    pub session_id: Option<String>,
}

impl EmitContext {

    /// Convenience constructor for daemon-internal background emits:
    /// `CallerKind::Runtime` + a trusted-internal binding (`Authority::Sdk`).
    pub fn daemon_internal(space: Option<SpaceId>) -> Self {
        Self {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            binding: ScopeBinding::daemon_internal(space, Authority::Sdk),
            session_id: None,
        }
    }
}

/// Configuration provided to the kernel when spawning an actor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActorConfig {
    pub namespace: String,
    pub write_namespaces: Vec<String>,
    pub read_namespaces: Vec<String>,
    pub subscribe_namespaces: Vec<String>,
    /// When true, the kernel ALSO delivers events with `target_actor_id`
    /// set (i.e. directed events) to this subscriber when its namespace
    /// matches — in addition to the targeted actor. Used by observability
    /// surfaces (SSE event stream, audit log tailers, dashboards) that
    /// need to see all traffic, not just the broadcast subset.
    ///
    /// Default: false. The kernel's directed-event short-circuit (delivery
    /// to target only) remains the per-actor norm. Explicit opt-in keeps
    /// chat / agent actors from accidentally inheriting cross-session
    /// observability traffic.
    #[serde(default)]
    pub see_directed_events: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_kind_eq() {
        assert_eq!(CallerKind::Cli, CallerKind::Cli);
        assert_ne!(CallerKind::Mcp, CallerKind::Cli);
        assert_ne!(CallerKind::Runtime, CallerKind::Mcp);
    }

    #[test]
    fn emit_context_has_caller_kind() {
        let ctx = EmitContext {
            caller_kind: CallerKind::Mcp,
            skill_id: None,
            binding: ScopeBinding::daemon_internal(None, Authority::Sdk),
            session_id: None,
        };
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
    }

    #[test]
    fn daemon_internal_binding_roundtrips() {
        let ctx = EmitContext::daemon_internal(Some(SpaceId::new("space-X")));
        assert_eq!(ctx.caller_kind, CallerKind::Runtime);
        assert_eq!(
            ctx.binding.key_for(crate::event_registry::EventScope::Space),
            Some("space-X".to_string())
        );
        assert_eq!(ctx.binding.device_id(), &crate::identity::local_device_id());
        assert_eq!(ctx.binding.account_id(), None);
        assert_eq!(
            EmitContext::daemon_internal(None)
                .binding
                .key_for(crate::event_registry::EventScope::Space),
            None
        );
    }

    #[test]
    fn actor_config_fields() {
        let cfg = ActorConfig {
            namespace: "job".to_string(),
            write_namespaces: vec!["job.".to_string()],
            read_namespaces: vec![],
            subscribe_namespaces: vec![],
            see_directed_events: false,
        };
        assert!(cfg.write_namespaces.contains(&"job.".to_string()));
        assert!(cfg.read_namespaces.is_empty());
    }
}
