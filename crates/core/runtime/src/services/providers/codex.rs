//! Codex CLI adapter.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::provider_adapter::{ProviderAdapter, ProviderEvent};

pub struct CodexAdapter;

impl ProviderAdapter for CodexAdapter {
    fn id(&self) -> &str { "codex" }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = Vec::new();
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_MESH_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "codex".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        self.build_interactive(slug, profile)
    }

    fn ready_pattern(&self) -> &str { "$ " }
    fn autostart_input(&self) -> &str { "go" }
    fn exit_command(&self) -> &str { "q" }

    fn parse_output(&self, _line: &str) -> Option<ProviderEvent> { None }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_codex.rs"]
mod tests;
