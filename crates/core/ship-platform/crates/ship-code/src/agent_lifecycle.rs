//! Agent lifecycle helpers — graceful agent exit for pipeline step transitions.

use ship_harnesses::harness_adapter::adapter_for_or_default;

/// Send the harness's exit command to the agent in a tmux session and wait for clean exit.
/// Best-effort — skips if tmux isn't available (tests).
pub async fn graceful_agent_exit(slug: &str, harness: Option<&str>) {
    let tmux_session = format!("job-{}", slug);
    // Best-effort: unknown harness → no exit command, force-kill path handles it.
    let exit_cmd = adapter_for_or_default(harness)
        .map(|a| a.exit_command().to_string())
        .unwrap_or_default();

    let has_session = ship_substrate::spawn_bounded::Cmd::new("tmux")
        .args(["has-session", "-t", &tmux_session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_session {
        tracing::debug!(slug, "pipeline: no tmux session, skipping graceful exit");
        return;
    }

    tracing::info!(slug, session = %tmux_session, "pipeline: sending exit command to current agent");
    let _ = ship_substrate::spawn_bounded::Cmd::new("tmux")
        .args(["send-keys", "-t", &tmux_session, &exit_cmd, "Enter"])
        .status();

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let output = ship_substrate::spawn_bounded::Cmd::new("tmux")
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
