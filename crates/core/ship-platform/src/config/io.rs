// Re-export config IO from ship-config::project.
// The canonical implementations moved there so ship-agents (C8) can use
// get_config/save_config without depending on ship-platform.
pub use ship_config::project::{get_config, get_effective_config, save_config};
