//! Agent lifecycle helpers — graceful agent exit for pipeline step transitions.

use runtime::services::provider_adapter::adapter_for_or_default;

/// Send the provider's exit command to the agent in a tmux session and wait for clean exit.
/// Best-effort — skips if tmux isn't available (tests).
pub(crate) async fn graceful_agent_exit(slug: &str, provider: Option<&str>) {
    let tmux_session = format!("job-{}", slug);
    // Best-effort: unknown provider → no exit command, force-kill path handles it.
    let exit_cmd = adapter_for_or_default(provider)
        .map(|a| a.exit_command().to_string())
        .unwrap_or_default();

    let has_session = std::process::Command::new("tmux")
        .args(["has-session", "-t", &tmux_session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_session {
        tracing::debug!(slug, "pipeline: no tmux session, skipping graceful exit");
        return;
    }

    tracing::info!(slug, session = %tmux_session, "pipeline: sending exit command to current agent");
    let _ = std::process::Command::new("tmux")
        .args(["send-keys", "-t", &tmux_session, &exit_cmd, "Enter"])
        .status();

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let output = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", &tmux_session, "-p"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.lines().rev().take(3).any(|l| l.trim_end().ends_with('$')) {
                tracing::info!(slug, "pipeline: agent exited cleanly");
                return;
            }
        }
    }
    tracing::warn!(slug, "pipeline: agent did not exit within 30s, proceeding anyway");
}
