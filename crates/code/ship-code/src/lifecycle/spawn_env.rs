//! Build the spawn environment for a daemon-spawned agent.


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
