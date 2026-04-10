//! Provider-specific CLI launch command builders.
//!
//! Each provider (Claude, Codex, etc.) knows how to translate a `LaunchProfile`
//! into the right binary + flags. Executors call these instead of hardcoding
//! CLI arguments.

use super::dispatch_ports::{LaunchCommand, LaunchProfile, ProviderLauncher};
use std::collections::HashMap;

/// Claude Code CLI launcher.
pub struct ClaudeLauncher;

impl ClaudeLauncher {
    /// Common args shared between interactive and headless modes.
    fn base_args(slug: &str, profile: &LaunchProfile) -> (Vec<String>, HashMap<String, String>) {
        let mut args = vec![
            "--dangerously-skip-permissions".to_string(),
            "--dangerously-load-development-channels".to_string(),
            "server:ship".to_string(),
        ];

        // Session identity — deterministic name for this job.
        if let Some(ref name) = profile.session_name {
            args.push("--name".to_string());
            args.push(name.clone());
        } else {
            args.push("--name".to_string());
            args.push(format!("job-{slug}"));
        }

        // Resume previous session.
        if let Some(ref session_id) = profile.resume_session {
            args.push("--resume".to_string());
            args.push(session_id.clone());
        }

        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        if let Some(ref effort) = profile.effort {
            args.push("--effort".to_string());
            args.push(effort.clone());
        }

        if let Some(budget) = profile.max_budget_usd {
            args.push("--max-budget-usd".to_string());
            args.push(format!("{budget}"));
        }

        if let Some(ref mode) = profile.permission_mode {
            args.push("--permission-mode".to_string());
            args.push(mode.clone());
        }

        if !profile.persist_session {
            args.push("--no-session-persistence".to_string());
        }

        let mut env = HashMap::new();
        env.insert("SHIP_MESH_ID".to_string(), slug.to_string());

        (args, env)
    }
}

impl ProviderLauncher for ClaudeLauncher {
    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let (args, env) = Self::base_args(slug, profile);
        LaunchCommand {
            bin: "claude".to_string(),
            args,
            env,
        }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let (mut args, env) = Self::base_args(slug, profile);

        args.push("--output-format".to_string());
        if profile.structured_output {
            args.push("stream-json".to_string());
        } else {
            args.push("text".to_string());
        }
        args.push("--print".to_string());
        args.push("-p".to_string());
        args.push("go".to_string());

        LaunchCommand {
            bin: "claude".to_string(),
            args,
            env,
        }
    }
}

/// Codex CLI launcher.
pub struct CodexLauncher;

impl ProviderLauncher for CodexLauncher {
    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = Vec::new();

        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        let mut env = HashMap::new();
        env.insert("SHIP_MESH_ID".to_string(), slug.to_string());

        LaunchCommand {
            bin: "codex".to_string(),
            args,
            env,
        }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        self.build_interactive(slug, profile)
    }
}

/// Resolve a provider name to the appropriate launcher.
pub fn launcher_for(provider: Option<&str>) -> Box<dyn ProviderLauncher> {
    match provider {
        Some("codex") => Box::new(CodexLauncher),
        _ => Box::new(ClaudeLauncher),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile() -> LaunchProfile {
        LaunchProfile {
            persist_session: true,
            ..Default::default()
        }
    }

    #[test]
    fn claude_interactive_basic_flags() {
        let launcher = ClaudeLauncher;
        let cmd = launcher.build_interactive("test-job", &test_profile());
        assert_eq!(cmd.bin, "claude");
        assert!(cmd.args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(cmd.args.contains(&"--dangerously-load-development-channels".to_string()));
        assert!(cmd.args.contains(&"server:ship".to_string()));
        assert!(cmd.args.contains(&"--name".to_string()));
        assert!(cmd.args.contains(&"job-test-job".to_string()));
        assert!(!cmd.args.contains(&"--print".to_string()));
        assert_eq!(cmd.env.get("SHIP_MESH_ID").unwrap(), "test-job");
    }

    #[test]
    fn claude_headless_includes_print_and_output_format() {
        let launcher = ClaudeLauncher;
        let cmd = launcher.build_headless("test-job", &test_profile());
        assert!(cmd.args.contains(&"--print".to_string()));
        assert!(cmd.args.contains(&"-p".to_string()));
        assert!(cmd.args.contains(&"--output-format".to_string()));
        assert!(cmd.args.contains(&"text".to_string()));
    }

    #[test]
    fn claude_headless_stream_json_when_structured() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.structured_output = true;
        let cmd = launcher.build_headless("test-job", &profile);
        assert!(cmd.args.contains(&"stream-json".to_string()));
    }

    #[test]
    fn claude_resume_passes_session_id() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.resume_session = Some("session-abc-123".to_string());
        let cmd = launcher.build_interactive("test-job", &profile);
        let idx = cmd.args.iter().position(|a| a == "--resume").unwrap();
        assert_eq!(cmd.args[idx + 1], "session-abc-123");
    }

    #[test]
    fn claude_model_flag() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.model = Some("opus".to_string());
        let cmd = launcher.build_interactive("test-job", &profile);
        let idx = cmd.args.iter().position(|a| a == "--model").unwrap();
        assert_eq!(cmd.args[idx + 1], "opus");
    }

    #[test]
    fn claude_effort_flag() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.effort = Some("high".to_string());
        let cmd = launcher.build_interactive("test-job", &profile);
        let idx = cmd.args.iter().position(|a| a == "--effort").unwrap();
        assert_eq!(cmd.args[idx + 1], "high");
    }

    #[test]
    fn claude_budget_flag() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.max_budget_usd = Some(5.0);
        let cmd = launcher.build_interactive("test-job", &profile);
        let idx = cmd.args.iter().position(|a| a == "--max-budget-usd").unwrap();
        assert_eq!(cmd.args[idx + 1], "5");
    }

    #[test]
    fn claude_ephemeral_session() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.persist_session = false;
        let cmd = launcher.build_interactive("test-job", &profile);
        assert!(cmd.args.contains(&"--no-session-persistence".to_string()));
    }

    #[test]
    fn claude_permission_mode() {
        let launcher = ClaudeLauncher;
        let mut profile = test_profile();
        profile.permission_mode = Some("plan".to_string());
        let cmd = launcher.build_interactive("test-job", &profile);
        let idx = cmd.args.iter().position(|a| a == "--permission-mode").unwrap();
        assert_eq!(cmd.args[idx + 1], "plan");
    }

    #[test]
    fn codex_basic_flags() {
        let launcher = CodexLauncher;
        let cmd = launcher.build_interactive("test-job", &test_profile());
        assert_eq!(cmd.bin, "codex");
        assert!(!cmd.args.contains(&"--dangerously-skip-permissions".to_string()));
    }

    #[test]
    fn launcher_for_resolves_correctly() {
        let ctx = test_profile();
        assert_eq!(launcher_for(None).build_interactive("s", &ctx).bin, "claude");
        assert_eq!(launcher_for(Some("claude")).build_interactive("s", &ctx).bin, "claude");
        assert_eq!(launcher_for(Some("codex")).build_interactive("s", &ctx).bin, "codex");
    }

    #[test]
    fn launch_command_to_shell_string() {
        let launcher = ClaudeLauncher;
        let cmd = launcher.build_interactive("test-job", &test_profile());
        let shell = cmd.to_shell_string();
        assert!(shell.contains("SHIP_MESH_ID=test-job"));
        assert!(shell.contains("claude"));
        assert!(shell.contains("--dangerously-skip-permissions"));
    }
}
