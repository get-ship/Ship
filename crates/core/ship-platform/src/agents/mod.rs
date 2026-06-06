//! Agent-domain — re-exported from the `ship-agents` crate (C8 keystone).
//! All logic moved to `ship_agents::agent_domain`. This shim keeps the
//! `crate::agents::*` paths working for the handful of callers that still
//! reference `ship_platform::agents::*`.

pub use ship_agents::agent_domain::config;
pub use ship_agents::agent_domain::harnesses;
pub use ship_agents::agent_domain::import;
pub use ship_agents::agent_domain::permissions;
pub use ship_agents::agent_domain::rule;
pub use ship_agents::agent_domain::skill;
pub use ship_agents::agent_domain::skill_paths;
pub use ship_agents::agent_domain::skill_vars;
