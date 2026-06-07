//! L1 impl of the domain-free [`SpawnPolicy`] port.
//!
//! Resolves the spawn environment for an op-invocation by a principal. This is
//! where the agent-domain reach-up the runner used to do inline now lives: the
//! config lookup (`get_config`), the agent → role mapping, and the scoped
//! daemon-session token mint (via [`AgentPlatform`]). The runner hands a
//! domain-free [`Authority`] principal and never names any of these.
//!
//! A later slice relocates the runner to L0; it will compile against
//! `SpawnPolicy` alone while this impl stays beside the L1 config/auth it needs.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use ship_platform::config::ProjectConfig;
use ship_substrate::services::dispatch_ports::{AgentPlatform, Authority, SpawnPolicy};

/// The current behavior, behind the port: resolve the agent's declared role
/// from project config and mint a role-scoped session token via the platform.
pub struct ConfigSpawnPolicy {
    platform: Arc<dyn AgentPlatform>,
}

impl ConfigSpawnPolicy {
    pub fn new(platform: Arc<dyn AgentPlatform>) -> Self {
        Self { platform }
    }
}

/// The declared role of `agent` in the project config, if any. Drives the
/// scoped token's capabilities at spawn. Pure (takes the loaded config) so the
/// mapping is unit-testable without touching the process-global cwd.
fn agent_role(cfg: &ProjectConfig, agent: &str) -> Option<String> {
    cfg.agents
        .iter()
        .find(|a| a.id == agent)
        .and_then(|a| a.role.clone())
}

/// The principal's identity for role resolution. Today only the agent
/// principal carries a config-declared role; other authorities mint with the
/// read-only default (`None` role) — fail closed, never silently elevated.
fn role_for(principal: &Authority, actor_id: &str) -> Option<String> {
    match principal {
        Authority::Agent { id } => {
            let cfg = ship_platform::config::get_config(None).ok()?;
            agent_role(&cfg, id)
        }
        // A non-agent principal (operator console, SDK token, cloud sync,
        // space-bound caller) carries no agent profile; resolve against
        // `actor_id` opportunistically so a future pipeline-token that names
        // an actor still picks up its declared role.
        _ => {
            let cfg = ship_platform::config::get_config(None).ok()?;
            agent_role(&cfg, actor_id)
        }
    }
}

#[async_trait]
impl SpawnPolicy for ConfigSpawnPolicy {
    async fn spawn_env(&self, principal: &Authority, actor_id: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), actor_id.to_string());
        // Mint the scoped token with the principal's resolved role so its
        // capabilities match what it is — designer → SuiAuthoring,
        // contributor → Full, etc. A missing/unknown role resolves to `None`
        // → `scopes_for_role`'s read-only default: fail closed.
        let role = role_for(principal, actor_id);
        if let Some(token) = self
            .platform
            .mint_session_token(actor_id, role.as_deref())
            .await
        {
            env.insert("SHIP_TOKEN".to_string(), token);
        }
        if let Some(url) = self.platform.daemon_url() {
            env.insert("SHIP_DAEMON_URL".to_string(), url);
        }
        env
    }
}

/// Test double: emits only `SHIP_ACTOR_ID`, never mints a token. Used by
/// orchestrator harness tests that exercise the runner mechanism without a
/// real config/auth backend. Public (not `#[cfg(test)]`) so test code in
/// downstream crates can build it — mirrors `MockPlatform`.
pub struct NoopSpawnPolicy;

#[async_trait]
impl SpawnPolicy for NoopSpawnPolicy {
    async fn spawn_env(&self, _principal: &Authority, actor_id: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), actor_id.to_string());
        env
    }
}

#[cfg(test)]
mod role_tests {
    use super::*;
    use ship_auth::scopes::{scopes_for_role, Scope};
    use ship_platform::config::AgentProfile;

    fn cfg_with(id: &str, role: Option<&str>) -> ProjectConfig {
        ProjectConfig {
            agents: vec![AgentProfile {
                id: id.to_string(),
                role: role.map(str::to_string),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// A designer agent's declared role resolves and mints SuiAuthoring — the
    /// scope the role-blind `None` silently denied.
    #[test]
    fn designer_role_resolves_and_grants_sui_authoring() {
        let cfg = cfg_with("dz", Some("designer"));
        let role = agent_role(&cfg, "dz");
        assert_eq!(role.as_deref(), Some("designer"));
        assert!(Scope::SuiAuthoring.is_granted_by(&scopes_for_role(role.as_deref())));
    }

    /// Unknown agent → no role → read-only default, never SuiAuthoring.
    #[test]
    fn unknown_agent_fails_closed() {
        let cfg = cfg_with("dz", Some("designer"));
        let role = agent_role(&cfg, "missing");
        assert_eq!(role, None);
        assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(role.as_deref())));
    }

    /// An agent with no declared role fails closed too — not silently elevated.
    #[test]
    fn roleless_agent_fails_closed() {
        let cfg = cfg_with("plain", None);
        let role = agent_role(&cfg, "plain");
        assert_eq!(role, None);
        assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(role.as_deref())));
    }
}
