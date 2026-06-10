//! AuthContext — the credential the agent presents at the one door.
//!
//! Faithful to shipd's `AuthContext` (apps/shipd/src/auth/mod.rs). The gate
//! consumes this and does not care how it was produced. The agent constructs it
//! in-process at dispatch: no socket, no PID attestation. `operator_cred` is
//! hard-wired false, so the agent can NEVER clear C3 — by construction.
//!
//! `grants` = the spec's Capability facet (granted service domains). This is
//! what the gate checks for C2; it replaces the dying role-derivation.

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub agent_id: String,
    /// Granted service domains (elevated_services). Mirrors spec.capability.grants.
    pub grants: Vec<String>,
    /// Always false in-process. Only a kernel-attested operator cred sets true.
    pub operator_cred: bool,
}

impl AuthContext {
    pub fn in_process(agent_id: impl Into<String>, grants: Vec<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            grants,
            operator_cred: false,
        }
    }
}
