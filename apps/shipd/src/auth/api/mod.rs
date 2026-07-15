//! Auth HTTP surface — server-side JWT attestation.
pub mod attest;

pub use attest::{AttestRequest, AttestResponse, server_attest};

#[cfg(test)]
mod tests {
    use super::super::AuthContext;
    use ship_substrate::events::CallerKind;

    #[test]
    fn auth_context_to_emit_context() {
        // Phase 2: the session is the sole binding minter. An MCP agent with no
        // resolved space keys `None` — there is NO home-space fallback on the
        // emit path.
        let auth = AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("agent.test".into()),
            space_id: None,
            session_id: Some("sess-1".into()),
            grants: vec![],
            operator_cred: false,
        };
        let ctx = auth.to_emit_context(CallerKind::Mcp);
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
        assert_eq!(
            ctx.binding.key_for(ship_types::event_registry::EventScope::Space),
            None,
            "no resolved space → keyed None (no home-space fallback)"
        );
        assert_eq!(ctx.session_id, Some("sess-1".into()));
    }
}
