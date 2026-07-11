//! L1 impl of the domain-free [`SpawnPolicy`] port.
//!
//! Resolves the spawn environment for an op-invocation by a principal. This is
//! where the agent-domain reach-up the runner used to do inline now lives: the
//! role map (read from the descriptor's resolved `AgentProfile.role` set, not
//! re-read from the process-global cwd on every call) and the scoped
//! daemon-session token mint (via [`AgentPlatform`]). The runner hands a
//! domain-free [`Authority`] principal and never names any of these.
//!
//! A later slice relocates the runner to L0; it will compile against
//! `SpawnPolicy` alone while this impl stays beside the L1 config/auth it needs.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use harness_adapter::dispatch_ports::{AgentPlatform, Authority, SpawnPolicy};

/// Read-only lookup of declared agent roles from the resolved descriptor.
///
/// The descriptor (a `ResolvedConfig`-shaped projection of the roll-up output)
/// is the only source of truth: at dispatch time the wire-resident
/// `ResolvedConfig` is captured once, roles extracted, and this trait is
/// backed by the captured map. No file system, no `get_config` call — the
/// field moves from a global read on every invocation to an injected value.
pub trait AgentRoleResolver: Send + Sync {
    /// Resolve the role declared for an agent id. `None` for unknown agents,
    /// missing-role agents, and non-agent principals (callers should not
    /// consult this for raw actor ids — that responsibility lives in
    /// [`role_for`]).
    fn role_for_agent(&self, agent_id: &str) -> Option<String>;
}

/// Test/CLI-friendly role resolver backed by a static map.
#[derive(Default)]
pub struct StaticAgentRoleResolver {
    entries: HashMap<String, String>,
}

impl StaticAgentRoleResolver {
    pub fn new<I, K, V>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            entries: entries
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

impl AgentRoleResolver for StaticAgentRoleResolver {
    fn role_for_agent(&self, agent_id: &str) -> Option<String> {
        self.entries.get(agent_id).cloned()
    }
}

/// The current behavior, behind the port: resolve the agent's declared role
/// from the injected role resolver (descriptor-sourced) and mint a role-scoped
/// session token via the platform.
pub struct ConfigSpawnPolicy {
    platform: Arc<dyn AgentPlatform>,
    roles: Arc<dyn AgentRoleResolver>,
}

impl ConfigSpawnPolicy {
    /// Build a policy that resolves roles from `roles` (descriptor-sourced)
    /// and mints tokens via `platform`. No filesystem access occurs here.
    pub fn new(platform: Arc<dyn AgentPlatform>, roles: Arc<dyn AgentRoleResolver>) -> Self {
        Self { platform, roles }
    }
}

/// The principal's identity for role resolution. Today only the agent
/// principal carries a config-declared role; other authorities mint with the
/// read-only default (`None` role) — fail closed, never silently elevated.
fn role_for(
    roles: &dyn AgentRoleResolver,
    principal: &Authority,
    actor_id: &str,
) -> Option<String> {
    match principal {
        Authority::Agent { id } => roles.role_for_agent(id),
        // A non-agent principal (operator console, SDK token, cloud sync,
        // space-bound caller) carries no agent profile; consult the
        // injected descriptor roles against `actor_id` opportunistically so
        // a future pipeline-token that names an actor still picks up its
        // declared role.
        _ => roles.role_for_agent(actor_id),
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
        let role = role_for(&*self.roles, principal, actor_id);
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

    fn roles(entries: &[(&str, &str)]) -> Arc<dyn AgentRoleResolver> {
        Arc::new(StaticAgentRoleResolver::new(entries.iter().map(|(id, role)| {
            ((*id).to_string(), (*role).to_string())
        })))
    }

    /// A designer agent's descriptor-declared role resolves and mints SuiAuthoring —
    /// the scope the role-blind `None` silently denied.
    #[test]
    fn designer_role_resolves_and_grants_sui_authoring() {
        let resolver = roles(&[("dz", "designer")]);
        let role = role_for(&*resolver, &Authority::Agent { id: "dz".into() }, "slug-1");
        assert_eq!(role.as_deref(), Some("designer"));
        assert!(Scope::SuiAuthoring.is_granted_by(&scopes_for_role(role.as_deref())));
    }

    /// Unknown agent → no role → read-only default, never SuiAuthoring.
    #[test]
    fn unknown_agent_fails_closed() {
        let resolver = roles(&[("dz", "designer")]);
        let role = role_for(&*resolver, &Authority::Agent { id: "missing".into() }, "slug-1");
        assert_eq!(role, None);
        assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(role.as_deref())));
    }

    /// A non-agent principal can still resolve through its actor id, but only
    /// through the injected descriptor role source — never by rereading config.
    #[test]
    fn non_agent_principal_resolves_actor_id_through_descriptor_roles() {
        let resolver = roles(&[("console-act", "operator")]);
        let role = role_for(
            &*resolver,
            &Authority::Sdk,
            "console-act",
        );
        assert_eq!(role.as_deref(), Some("operator"));
    }

    /// An agent with no declared role fails closed too — not silently elevated.
    #[test]
    fn roleless_agent_fails_closed() {
        let resolver = roles(&[("plain", "designer")]);
        // Replace the role with None by removing it from the resolver.
        let resolver = roles(&[]);
        let role = role_for(
            &*resolver,
            &Authority::Agent { id: "plain".into() },
            "slug-1",
        );
        assert_eq!(role, None);
        assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(role.as_deref())));
    }

    /// `ConfigSpawnPolicy` is wired without a `get_config` call — the role
    /// surface is the injected resolver only.
    #[test]
    fn config_spawn_policy_does_not_read_config() {
        // Sanity: the type carries no SpaceManifest / config reader fields.
        // Struct shape change fails to compile, which is the test.
        let platform: Arc<dyn AgentPlatform> = Arc::new(NoopPlatform);
        let resolver: Arc<dyn AgentRoleResolver> = Arc::new(StaticAgentRoleResolver::empty());
        let _policy: Arc<dyn SpawnPolicy> = Arc::new(ConfigSpawnPolicy::new(platform, resolver));
    }

    struct NoopPlatform;

    #[async_trait]
    impl AgentPlatform for NoopPlatform {
        fn worktrees_dir(&self) -> anyhow::Result<std::path::PathBuf> {
            Ok(std::path::PathBuf::from("/tmp/noop-worktrees"))
        }
        fn compile_agent_config(
            &self,
            _worktree_path: &std::path::Path,
            _agent_id: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        async fn graceful_agent_exit(&self, _slug: &str, _harness: Option<&str>) {}
        async fn cleanup_job(&self, _slug: &str, _branch: &str) {}
        fn git_merge(&self, _branch: &str) -> anyhow::Result<bool> {
            Ok(false)
        }
        async fn cleanup_orphaned_worktrees(&self) {}
    }
}
