//! Codex CLI adapter.

use std::collections::HashMap;
use std::path::Path;

use ship_substrate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use ship_runner::harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, ResumeError,
};

pub struct CodexAdapter;

#[async_trait::async_trait]
impl HarnessAdapter for CodexAdapter {
    fn id(&self) -> &str { "codex" }

    /// Codex ships an app-server (`codex app-server`) we hold as a persistent
    /// session. Spawn it; on failure return None so routing falls back to the
    /// one-shot `codex exec` path.
    async fn spawn_server_session(
        &self,
    ) -> Option<std::sync::Arc<dyn ship_substrate::services::server_session::ServerSession>> {
        match crate::harnesses::codex_session::CodexSession::spawn().await {
            Ok(session) => Some(std::sync::Arc::new(session)),
            Err(err) => {
                tracing::warn!(%err, "codex: server-session spawn failed; falling back to one-shot exec");
                None
            }
        }
    }

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
            push_mechanism: ship_runner::harness_adapter::PushMechanism::Stdin,
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
    // Flags match the codex 0.131+ CLI:
    //   --sandbox workspace-write  write-allowed-in-workspace sandbox.
    //                              Replaces the legacy --full-auto flag,
    //                              which is deprecated as of 0.131 and
    //                              prints a warning on every run.
    //   --skip-git-repo-check      override git repo requirement (we
    //                              dispatch into worktrees that may not
    //                              look like a canonical git root).
    //   --json                     JSON Lines stream of all events to
    //                              stdout (thread.started, turn.completed,
    //                              item.*, turn.failed) — lets the reader
    //                              thread parse structured failures
    //                              (usage limits, API errors) instead of
    //                              guessing from plain text.
    //   --ephemeral                skip persisting session rollout files;
    //                              we don't resume these jobs.
    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = vec![
            "exec".to_string(),
            "--sandbox".to_string(),
            "workspace-write".to_string(),
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
        // implies a persisted rollout to anchor on. --sandbox matches
        // the headless flag set above (replaces deprecated --full-auto).
        let mut args = vec![
            "exec".to_string(),
            "resume".to_string(),
            session_id.to_string(),
            "--sandbox".to_string(),
            "workspace-write".to_string(),
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

    /// Codex `--json` stream emits these events relevant to observability:
    ///   thread.started — early in run; carries `thread_id` (the resume key).
    ///   turn.completed — end of turn; carries `usage` block with token
    ///                    counts. Shape (per codex-rs/exec/src/exec_events.rs):
    ///                    `{"type":"turn.completed","usage":{
    ///                       "input_tokens":i64, "cached_input_tokens":i64,
    ///                       "output_tokens":i64, "reasoning_output_tokens":i64 }}`
    ///                    No cost field; no stop_reason; no model echo.
    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ty = v.get("type")?.as_str()?;
        match ty {
            "thread.started" => {
                let tid = v.get("thread_id").and_then(|s| s.as_str())?;
                Some(HarnessEvent::SessionStarted {
                    session_id: tid.to_string(),
                })
            }
            "turn.completed" => {
                let usage = v.get("usage")?;
                let input_tokens =
                    usage.get("input_tokens").and_then(|n| n.as_i64()).unwrap_or(0) as u64;
                let output_tokens =
                    usage.get("output_tokens").and_then(|n| n.as_i64()).unwrap_or(0) as u64;
                let reasoning_tokens = usage
                    .get("reasoning_output_tokens")
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as u64;
                let cache_read_tokens = usage
                    .get("cached_input_tokens")
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as u64;
                Some(HarnessEvent::TurnUsage {
                    model: None,
                    input_tokens,
                    output_tokens,
                    reasoning_tokens,
                    cache_creation_tokens: 0,
                    cache_read_tokens,
                    stop_reason: None,
                    harness_reported_cost_usd: None,
                })
            }
            _ => None,
        }
    }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_codex.rs"]
mod tests;
