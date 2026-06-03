//! Build the spawn environment for a daemon-spawned agent via the domain-free
//! [`SpawnPolicy`] port.
//!
//! The agent dispatch names the principal it acts as
//! ([`Authority::Agent`]) and the target `slug`; the policy impl resolves
//! `SHIP_ACTOR_ID` plus, when the host issues them, a scoped `SHIP_TOKEN` +
//! `SHIP_DAEMON_URL`. The role/token resolution lives entirely inside the impl
//! (`ConfigSpawnPolicy` in L1) — this crate never names config / auth /
//! agent-role types, only the principal and the port.

use std::collections::HashMap;

use ship_substrate::services::dispatch_ports::SpawnPolicy;
use ship_types::Authority;

/// Resolve the env to inject into the harness spawned for `agent`, targeting
/// `slug`, through the [`SpawnPolicy`] port.
pub async fn build_spawn_env(
    slug: &str,
    agent: &str,
    spawn_policy: &dyn SpawnPolicy,
) -> HashMap<String, String> {
    let principal = Authority::Agent {
        id: agent.to_string(),
    };
    spawn_policy.spawn_env(&principal, slug).await
}
