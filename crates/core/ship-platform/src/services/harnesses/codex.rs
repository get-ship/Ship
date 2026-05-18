//! Codex CLI adapter.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, ResumeError,
};

pub struct CodexAdapter;

impl HarnessAdapter for CodexAdapter {
    fn id(&self) -> &str { "codex" }

    fn caps(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            headless: true,
            interactive: true,
            server_mode: true,
            json_stream: true,
            mcp: true,
            hooks: true,
            session_resume: true,
            persistent_session: true,
            push_mechanism: super::super::harness_adapter::PushMechanism::Stdin,
        }
    }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = Vec::new();
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "codex".to_string(), args, env }
    }

    // `codex exec` is non-TUI: it runs the prompt and exits naturally, so
    // ready_pattern / exit_command only matter for the interactive path.
    //
    // Flags match the OpenAI docs (developers.openai.com/codex/noninteractive):
    //   --full-auto           allow edits without approval
    //   --skip-git-repo-check override git repo requirement (we worktree)
    //   --json                JSON Lines stream of all events to stdout
    //                         (thread.started, turn.completed, item.*,
    //                         turn.failed) — lets the reader thread parse
    //                         structured failures (usage limits, API errors)
    //                         instead of guessing from plain text.
    //   --ephemeral           skip persisting session rollout files; we
    //                         don't resume these jobs.
    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = vec![
            "exec".to_string(),
            "--full-auto".to_string(),
            "--skip-git-repo-check".to_string(),
            "--json".to_string(),
            "--ephemeral".to_string(),
        ];
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let prompt = profile
            .autostart_prompt
            .clone()
            .unwrap_or_else(|| "Read .artifacts/job-spec.md and execute it.".to_string());
        args.push(prompt);

        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "codex".to_string(), args, env }
    }

    fn ready_pattern(&self) -> &str { "$ " }
    fn autostart_input(&self) -> &str { "go" }
    fn exit_command(&self) -> &str { "q" }

    fn build_resume_args(
        &self,
        slug: &str,
        session_id: &str,
        profile: &LaunchProfile,
    ) -> Result<LaunchCommand, ResumeError> {
        if session_id.is_empty() {
            return Err(ResumeError::BadSession {
                harness: self.id().to_string(),
                reason: "empty session id".to_string(),
            });
        }
        // `codex exec resume <SESSION_ID> [prompt]` — the documented CLI
        // shape per OpenAI's codex docs. Drop `--ephemeral`: a resume
        // implies a persisted rollout to anchor on.
        let mut args = vec![
            "exec".to_string(),
            "resume".to_string(),
            session_id.to_string(),
            "--full-auto".to_string(),
            "--skip-git-repo-check".to_string(),
            "--json".to_string(),
        ];
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let prompt = profile
            .autostart_prompt
            .clone()
            .unwrap_or_else(|| "Continue.".to_string());
        args.push(prompt);

        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        Ok(LaunchCommand { bin: "codex".to_string(), args, env })
    }

    /// Codex `--json` stream emits `thread.started` early in a run with
    /// the thread id we resume against later. Surface that as
    /// `SessionStarted` so the reader middleware can persist
    /// `.ship/session.json`.
    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ty = v.get("type")?.as_str()?;
        if ty == "thread.started"
            && let Some(tid) = v.get("thread_id").and_then(|s| s.as_str())
        {
            return Some(HarnessEvent::SessionStarted {
                session_id: tid.to_string(),
            });
        }
        None
    }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_codex.rs"]
mod tests;
