//! Gemini CLI adapter — `gemini` binary with `-p` headless flag.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::provider_adapter::{ProviderAdapter, ProviderEvent};

pub struct GeminiAdapter;

impl ProviderAdapter for GeminiAdapter {
    fn id(&self) -> &str { "gemini" }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = Vec::new();
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_MESH_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "gemini".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = vec!["--yolo".to_string()];
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        args.push("-p".to_string());
        let prompt = profile
            .autostart_prompt
            .clone()
            .unwrap_or_else(|| "Read .artifacts/job-spec.md and execute it.".to_string());
        args.push(prompt);

        let mut env = HashMap::new();
        env.insert("SHIP_MESH_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "gemini".to_string(), args, env }
    }

    fn ready_pattern(&self) -> &str { "$ " }
    fn autostart_input(&self) -> &str { "go" }
    fn exit_command(&self) -> &str { "" }

    fn parse_output(&self, _line: &str) -> Option<ProviderEvent> { None }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_gemini.rs"]
mod tests;
