use crate::db::session::{get_active_workspace_session_db, get_workspace_session_db};
use crate::db::session_events::{
    insert_session_attached_event, insert_session_recorded_event,
    insert_session_with_started_event, update_session_with_ended_event,
};
use crate::db::workspace_state::get_workspace_config_generation_db;
use crate::events::types::{SessionAttached, SessionEnded, SessionRecorded, SessionStarted};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;
use std::process::Command;

// ---- Git metrics -----------------------------------------------------------

#[derive(Debug, Default)]
struct GitMetrics {
    files_changed: Option<i64>,
    lines_added: Option<i64>,
    lines_removed: Option<i64>,
}

fn collect_git_metrics(ship_dir: &Path) -> GitMetrics {
    let project_dir = ship_dir.parent().unwrap_or(ship_dir);
    let output = Command::new("git")
        .args(["diff", "--stat", "--stat-width=200", "HEAD"])
        .current_dir(project_dir)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return GitMetrics::default(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_diff_stat(&stdout)
}

fn parse_git_diff_stat(output: &str) -> GitMetrics {
    let Some(summary) = output.lines().last() else {
        return GitMetrics::default();
    };

    let mut files = None;
    let mut added = None;
    let mut removed = None;

    for part in summary.split(',') {
        let part = part.trim();
        if part.contains("file") && part.contains("changed") {
            files = part.split_whitespace().next().and_then(|n| n.parse().ok());
        } else if part.contains("insertion") {
            added = part.split_whitespace().next().and_then(|n| n.parse().ok());
        } else if part.contains("deletion") {
            removed = part.split_whitespace().next().and_then(|n| n.parse().ok());
        }
    }

    GitMetrics { files_changed: files, lines_added: added, lines_removed: removed }
}

use super::compile::{compile_workspace_context, resolve_session_providers};
use super::crud::get_workspace;
use super::helpers::*;
use super::lifecycle::create_workspace;
use super::session::{
    annotate_session_stale_state, hydrate_workspace_session, persist_session_artifact,
};
use super::types_session::*;

// ---- Post-session hooks ----------------------------------------------------

fn run_post_session_hooks(ship_dir: &Path, session: &WorkspaceSession) -> Result<()> {
    let effective = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    let hooks: Vec<_> = effective
        .hooks
        .into_iter()
        .filter(|hook| {
            hook.trigger == crate::config::HookTrigger::SessionEnd
                || hook.trigger == crate::config::HookTrigger::Stop
        })
        .collect();

    for hook in hooks {
        let command = hook.command.trim();
        if command.is_empty() {
            continue;
        }

        let mut process = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-lc", command]);
            cmd
        };

        let output = process
            .current_dir(ship_dir)
            .env("SHIP_ACTOR_ID", &session.id)
            .env("SHIP_SESSION_BRANCH", &session.workspace_branch)
            .output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    tracing::warn!(
                        "Post-session hook '{}' failed with status {:?}",
                        hook.id,
                        out.status.code()
                    );
                }
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to execute post-session hook '{}': {}",
                    hook.id, error
                );
            }
        }
    }

    Ok(())
}

// ---- Start / end -----------------------------------------------------------

/// Start a workspace session. Idempotent — returns existing active session.
pub fn start_workspace_session(
    ship_dir: &Path,
    branch: &str,
    goal: Option<String>,
    agent_id: Option<String>,
    primary_provider: Option<String>,
) -> Result<WorkspaceSession> {
    let branch = ensure_branch_key(branch)?;

    // Get or create workspace record.
    let workspace = match get_workspace(ship_dir, branch)? {
        Some(ws) => ws,
        None => create_workspace(ship_dir, CreateWorkspaceRequest {
            branch: branch.to_string(),
            active_agent: agent_id.clone(),
            ..Default::default()
        })?,
    };

    // Idempotent: return existing active session.
    if let Some(active) = get_active_workspace_session_db(&workspace.id)? {
        let mut existing = hydrate_workspace_session(active);
        annotate_session_stale_state(&mut existing, &std::collections::HashMap::new());
        let payload = SessionAttached {
            existing_session_id: existing.id.clone(),
            workspace_id: workspace.id.clone(),
            workspace_branch: workspace.name.clone(),
        };
        insert_session_attached_event(&existing.id, &workspace.id, &payload)?;
        if let Err(e) = persist_session_artifact(ship_dir, &existing, "attach") {
            tracing::warn!("failed to persist attached session artifact: {e}");
        }
        return Ok(existing);
    }

    // Resolve providers and compile context.
    let agent_id = agent_id
        .or(workspace.active_agent.clone())
        .and_then(|v| normalize_optional_text(Some(v)));
    let providers = resolve_session_providers(ship_dir, &workspace, agent_id.as_deref())?;
    let primary_provider = resolve_primary_provider(primary_provider, &providers, &workspace)?;
    let _providers = compile_workspace_context(ship_dir, &workspace, agent_id.as_deref())?;
    let config_generation_at_start = get_workspace_config_generation_db(&workspace.id)?;

    let session_id = crate::gen_nanoid();
    let started_payload = SessionStarted {
        goal: normalize_optional_text(goal),
        workspace_id: workspace.id.clone(),
        workspace_branch: workspace.name.clone(),
        agent_id,
        primary_provider: Some(primary_provider),
        config_generation_at_start,
        compiled_at: None,
    };
    let _envelope = insert_session_with_started_event(&session_id, &workspace.id, &started_payload)?;
    let created = get_workspace_session_db(&session_id)?
        .ok_or_else(|| anyhow!("Failed to load created workspace session"))?;
    let started = hydrate_workspace_session(created);

    if let Err(e) = persist_session_artifact(ship_dir, &started, "start") {
        tracing::warn!("failed to persist session artifact on start: {e}");
    }

    Ok(started)
}

fn resolve_primary_provider(
    requested: Option<String>,
    providers: &[String],
    workspace: &super::types::Workspace,
) -> Result<String> {
    if let Some(req) = requested {
        let normalized = normalize_provider_ref(&req)
            .ok_or_else(|| anyhow!("Session provider cannot be empty"))?;
        if !providers.iter().any(|p| p == &normalized) {
            return Err(anyhow!(
                "Provider '{}' not allowed for workspace '{}' (allowed: {})",
                normalized, workspace.name, providers.join(", ")
            ));
        }
        Ok(normalized)
    } else {
        providers.first().cloned()
            .ok_or_else(|| anyhow!("No providers resolved for workspace '{}'", workspace.name))
    }
}

pub fn end_workspace_session(
    ship_dir: &Path,
    branch: &str,
    request: EndWorkspaceSessionRequest,
) -> Result<WorkspaceSession> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow!("Workspace not found for branch '{}'", branch))?;

    let mut active = get_active_workspace_session_db(&workspace.id)?
        .ok_or_else(|| anyhow!("No active workspace session for '{}'", workspace.name))?;

    let now = Utc::now().to_rfc3339();
    active.status = WorkspaceSessionStatus::Ended.to_string();
    active.ended_at = Some(now.clone());
    active.summary = normalize_optional_text(request.summary);
    active.updated_workspace_ids = request.updated_workspace_ids;
    active.updated_at = now;

    let started_at = active.started_at.parse::<chrono::DateTime<Utc>>()
        .map_err(|e| anyhow!("invalid started_at '{}': {}", active.started_at, e))?;
    let ended_ts = active.ended_at.as_deref()
        .ok_or_else(|| anyhow!("ended_at must be set before emitting session.ended"))?
        .parse::<chrono::DateTime<Utc>>()
        .map_err(|e| anyhow!("invalid ended_at: {}", e))?;
    let duration_secs_val = (ended_ts - started_at).num_seconds();

    // Collect git metrics best-effort — failure doesn't prevent session end.
    let git_metrics = collect_git_metrics(ship_dir);
    let files_changed = request.files_changed.or(git_metrics.files_changed);
    let lines_added = request.lines_added.or(git_metrics.lines_added);
    let lines_removed = request.lines_removed.or(git_metrics.lines_removed);

    let tool_calls = if active.tool_call_count > 0 {
        Some(active.tool_call_count)
    } else {
        request.tool_calls
    };

    let ended_payload = SessionEnded {
        summary: active.summary.clone(),
        duration_secs: Some(duration_secs_val as u64),
        gate_result: request.gate_result.clone(),
        updated_workspace_ids: active.updated_workspace_ids.clone(),
        compile_error: active.compile_error.clone(),
        agent_id: active.agent_id.clone(),
        provider: active.primary_provider.clone(),
        model: request.model.clone(),
        files_changed,
        lines_added,
        lines_removed,
        tool_calls,
        exit_reason: request.exit_reason.clone(),
    };
    let _envelope = update_session_with_ended_event(&active.id, &active.workspace_id, &ended_payload)?;

    let ended = get_workspace_session_db(&active.id)?
        .ok_or_else(|| anyhow!("Failed to load ended workspace session"))?;
    let ended = hydrate_workspace_session(ended);

    let duration_secs = ended.ended_at.map(|end| (end - ended.started_at).num_seconds());

    let record_id = crate::gen_nanoid();
    let recorded_payload = SessionRecorded {
        record_id: record_id.clone(),
        summary: ended.summary.clone(),
        updated_workspace_ids: ended.updated_workspace_ids.clone(),
        duration_secs,
        provider: ended.primary_provider.clone(),
        model: request.model,
        agent_id: ended.agent_id.clone(),
        files_changed,
        gate_result: request.gate_result,
        workspace_branch: ended.workspace_branch.clone(),
        lines_added,
        lines_removed,
        tool_calls,
        exit_reason: request.exit_reason,
    };
    let _envelope =
        insert_session_recorded_event(&ended.id, &ended.workspace_id, &recorded_payload)?;

    let mut ended = ended;
    ended.session_record_id = Some(record_id);

    if let Err(e) = persist_session_artifact(ship_dir, &ended, "end") {
        tracing::warn!("failed to persist session artifact on end: {e}");
    }
    let hook_ship_dir = ship_dir.to_path_buf();
    let hook_session = ended.clone();
    std::thread::spawn(move || {
        if let Err(e) = run_post_session_hooks(&hook_ship_dir, &hook_session) {
            tracing::warn!("failed to run post-session hooks: {e}");
        }
    });

    Ok(ended)
}
