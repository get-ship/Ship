// Re-export project-config types from ship-config::project.
// The canonical definitions moved there so ship-agents (C8) can use them
// without depending on ship-platform.
pub use ship_config::project::{
    AgentProfile, HarnessProjectOverrides, McpConfig, McpSection, McpServerConfig, McpServerType,
    ProjectConfig, ProjectDiscovery,
};
