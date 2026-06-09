//! Build the spawn environment for a daemon-spawned agent.
//! Moved from ship-runner (L0) to ship-code (L1).

use std::collections::HashMap;

use crate::dispatch_ports::SpawnPolicy;
use ship_types::Authority;

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
