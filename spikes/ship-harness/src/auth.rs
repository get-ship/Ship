//! AuthContext — faithful mirror of shipd's `AuthContext`
//! (apps/shipd/src/auth/mod.rs:73). The gate (`gate::caller_from_auth` ->
//! `decide`) consumes this and does NOT care how it was produced.
//!
//! Today shipd has two producers: kernel-attested SO_PEERCRED (out-of-process
//! harnesses) and session tokens. The base Ship agent adds a THIRD provenance:
//! shipd constructs the context in-process at dispatch. Same gate, different
//! source.
//!
//! Why this matters beyond the agent: identity is no longer tied to a connecting
//! process PID. An embedded agent can live inside the Ship shell / daemon as a
//! first-class surface — AI integrated deeply, not bolted onto a terminal
//! window's PID. That's the shell-AI integration unlock.
//!
//! Invariant carried over from shipd: `is_operator()` requires a verified,
//! pid-attested operator cred. An in-process context can't have one, so the base
//! agent can NEVER clear C3 — by construction, not by policy.

#[derive(Clone, Debug)]
pub struct AuthContext {
    /// Who this is, e.g. `Some("ship-agent-<id>")`.
    pub agent_id: Option<String>,
    /// Coarse scopes (mirrors shipd `scopes`). Carried for shape fidelity.
    #[allow(dead_code)]
    pub scopes: Vec<String>,
    /// Elevated (C2) services this identity may reach, e.g. `["git"]`.
    pub grants: Vec<String>,
    /// Operator clearance. ALWAYS false for an in-process agent — only a
    /// kernel-attested operator cred sets this true (out-of-process path).
    pub operator_cred: bool,
}

impl AuthContext {
    /// The third provenance: construct identity in-process at dispatch. No
    /// socket, no PID attestation. `operator_cred` is hard-wired false.
    pub fn in_process(agent_id: impl Into<String>, grants: Vec<String>) -> Self {
        Self {
            agent_id: Some(agent_id.into()),
            scopes: Vec::new(),
            grants,
            operator_cred: false,
        }
    }

    /// Mirrors shipd's gate check. Always false in-process — the invariant.
    #[allow(dead_code)]
    pub fn is_operator(&self) -> bool {
        self.operator_cred
    }
}
