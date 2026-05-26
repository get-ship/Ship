//! Auth HTTP surface — three orthogonal flows split per concern:
//!
//! - [`pairing`] — browser pair-init / pair-confirm / pair-status
//! - [`attest`] — server-side JWT attestation
//! - [`sessions`] — auth-token list + revoke (daemon-token only)
//!
//! Re-exports flatten the import path; existing call sites
//! (`router.rs`, `routes/openapi.rs`) continue to resolve through
//! `crate::auth::*`.

pub mod attest;
pub mod pairing;
pub mod sessions;

pub use attest::{AttestRequest, AttestResponse, server_attest};
pub use pairing::{
    PairConfirmRequest, PairConfirmResponse, PairInitRequest, PairInitResponse, PairStatusQuery,
    PairStatusResponse, pair_confirm, pair_init, pair_status,
};
pub use sessions::{
    ListSessionsResponse, RevokeSessionsRequest, RevokeSessionsResponse, SessionSummary,
    list_auth_sessions, revoke_sessions,
};

#[cfg(test)]
mod tests {
    // Token utilities live in the parent auth.rs.
    use super::super::{AuthContext, generate_token, is_daemon_token, read_token, write_token_file};
    use crate::auth::scopes::Scope;
    use ship_substrate::events::CallerKind;

    #[test]
    fn generate_token_is_64_hex_chars() {
        let token = generate_token().unwrap();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_token_is_unique() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn write_and_read_token_roundtrips() {
        let dir = tempfile::TempDir::new().unwrap();
        let token = "abc123def456";
        write_token_file(dir.path(), token).unwrap();
        let read = read_token(dir.path()).unwrap();
        assert_eq!(read, token);
    }

    #[cfg(unix)]
    #[test]
    fn write_token_file_sets_daemon_dir_0700() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        let nested = dir.path().join("machine").join("daemon");
        write_token_file(&nested, "tok").unwrap();
        let dir_mode = std::fs::metadata(&nested).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "daemon dir must be 0700");
        let file_mode = std::fs::metadata(nested.join("token")).unwrap().permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600, "token file must be 0600");
    }

    #[test]
    fn is_daemon_token_valid() {
        let token = generate_token().unwrap();
        assert!(is_daemon_token(&token));
    }

    #[test]
    fn is_daemon_token_rejects_non_hex() {
        assert!(!is_daemon_token("ship_session_abc123"));
        assert!(!is_daemon_token("too_short"));
    }

    #[test]
    fn auth_context_to_emit_context() {
        // The daemon's space id is the sole scope. Setting it once here (the
        // only reader of this set-once global in the shipd test binary) lets us
        // assert the new behavior: `project_id` carries the SPACE id and is
        // *independent* of the legacy branch-slug `workspace_id`.
        ship_substrate::project::set_daemon_space_id("space-abc".into());
        let auth = AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("agent.test".into()),
            // A branch slug that must NOT leak into project_id anymore.
            workspace_id: Some("ws-1".into()),
            session_id: Some("sess-1".into()),
            scopes: vec![Scope::Full],
            grants: vec![],
        };
        let ctx = auth.to_emit_context();
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
        assert_eq!(
            ctx.project_id,
            Some("space-abc".into()),
            "project_id must carry the daemon space id, not the branch slug"
        );
        assert_eq!(ctx.session_id, Some("sess-1".into()));
    }

    #[test]
    fn has_scope_checks_granted() {
        let auth = AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::ReadWorkspace, Scope::LogProgress],
            grants: vec![],
        };
        assert!(auth.has_scope(Scope::ReadWorkspace));
        assert!(!auth.has_scope(Scope::CreateRuns));
    }
}
