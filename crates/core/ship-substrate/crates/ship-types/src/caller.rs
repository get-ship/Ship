use serde::{Deserialize, Serialize};

/// Who is emitting the event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerKind {
    Cli,
    Mcp,
    Sdk,
    CloudSync,
    Runtime,
}

/// Context provided alongside every emit call.
pub struct EmitContext {
    pub caller_kind: CallerKind,
    pub skill_id: Option<String>,
    pub space_id: Option<String>,
    pub session_id: Option<String>,
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
            space_id: None,
            session_id: None,
        };
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
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
