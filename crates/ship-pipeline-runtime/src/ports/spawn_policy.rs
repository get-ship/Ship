//! L1 impl of the domain-free [`SpawnPolicy`] port.
//!
//! Resolves the spawn environment for an op-invocation by a principal: the
//! scoped daemon-session token mint (via [`AgentPlatform`]). The runner hands a
//! domain-free [`Authority`] principal and never names any of these.
//!
//! A later slice relocates the runner to L0; it will compile against
//! `SpawnPolicy` alone while this impl stays beside the L1 config/auth it needs.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use agent_driver::dispatch_ports::{AgentPlatform, Authority, SpawnPolicy};

/// The current behavior, behind the port: mint a scoped session token for the
/// actor via the platform. An agent's capabilities are its provisioned grant
/// baseline, resolved by the platform at mint time.
pub struct ConfigSpawnPolicy {
    platform: Arc<dyn AgentPlatform>,
}

impl ConfigSpawnPolicy {
    /// Build a policy that mints tokens via `platform`. No filesystem access
    /// occurs here.
    pub fn new(platform: Arc<dyn AgentPlatform>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl SpawnPolicy for ConfigSpawnPolicy {
    async fn spawn_env(&self, _principal: &Authority, actor_id: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), actor_id.to_string());
        // Mint the scoped token for the actor. Its capabilities are its
        // provisioned grant baseline (resolved by the platform at mint).
        if let Some(token) = self.platform.mint_session_token(actor_id).await {
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
mod tests {
    use super::*;

    /// `ConfigSpawnPolicy` is wired without a `get_config` call — it carries
    /// only the platform, no config reader.
    #[test]
    fn config_spawn_policy_carries_only_platform() {
        let platform: Arc<dyn AgentPlatform> = Arc::new(NoopPlatform);
        let _policy: Arc<dyn SpawnPolicy> = Arc::new(ConfigSpawnPolicy::new(platform));
    }

    #[tokio::test]
    async fn noop_policy_emits_actor_id_only() {
        let env = NoopSpawnPolicy.spawn_env(&Authority::Sdk, "slug-1").await;
        assert_eq!(env.get("SHIP_ACTOR_ID").map(String::as_str), Some("slug-1"));
        assert!(!env.contains_key("SHIP_TOKEN"));
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
