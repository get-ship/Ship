// Re-export hook/permission/layer types from ship-config::project.
// The canonical definitions moved there so ship-agents (C8) can use them
// without depending on ship-platform.
pub use ship_config::project::{
    AgentLayerConfig, HookConfig, HookTrigger, PermissionConfig,
};

pub use ship_substrate::project::PRIMARY_CONFIG_FILE;
