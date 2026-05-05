use ship_substrate::db::session::{get_active_session_db, get_session_db};
use ship_substrate::db::session_events::{
    insert_session_attached_event, insert_session_with_started_event,
    update_session_with_ended_event,
};
use ship_substrate::events::types::{SessionAttached, SessionEnded, SessionStarted};
use ship_substrate::git;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;

use super::compile::{compile_session_context, resolve_session_providers};
use super::helpers::*;
use super::session::{hydrate_session, persist_session_artifact};
use super::types_session::*;
use ship_substrate::project::sanitize_file_name;

// ---- Start / end -----------------------------------------------------------

/// Start a session on the given branch. Idempotent — returns existing
/// active session.
pub fn start_session(
    ship_dir: &Path,
    branch: &str,
    goal: Option<String>,
    agent_id: Option<String>,
    primary_provider: Option<String>,
) -> Result<Session> {
    let branch = ensure_branch_key(branch)?;
    let workspace_id = sanitize_file_name(branch);

    // Idempotent: return existing active session.
    if let Some(active) = get_active_session_db(&workspace_id)? {
        let existing = hydrate_session(active);
        let payload = SessionAttached {
            existing_session_id: existing.id.clone(),
            workspace_id: workspace_id.clone(),
            branch: branch.to_string(),
        };
        insert_session_attached_event(&existing.id, &workspace_id, &payload)?;
        if let Err(e) = persist_session_artifact(ship_dir, &existing, "attach") {
            tracing::warn!("failed to persist attached session artifact: {e}");
        }
        return Ok(existing);
    }

    // Resolve providers and compile context.
    let agent_id = agent_id.and_then(|v| normalize_optional_text(Some(v)));
    let providers = resolve_session_providers(ship_dir, branch, agent_id.as_deref())?;
    let primary_provider = resolve_primary_provider(primary_provider, &providers, branch)?;
    let _providers = compile_session_context(ship_dir, branch, None, agent_id.as_deref())?;

    // Look up project_id from ship_dir.parent (the project root). When the
    // project isn't registered, leaves project_id as None — registration is
    // explicit, never implicit.
    let project_lookup_path = ship_dir.parent().unwrap_or(ship_dir).to_path_buf();
    let project_id = ship_substrate::projects::find_by_path(&project_lookup_path)
        .ok()
        .flatten()
        .map(|p| p.id);

    let session_id = ship_substrate::gen_nanoid();
    let started_payload = SessionStarted {
        goal: normalize_optional_text(goal),
        workspace_id: workspace_id.clone(),
        branch: branch.to_string(),
        agent_id,
        primary_provider: Some(primary_provider),
        project_id,
        kind: Some("direct_attach".to_string()),
        pipeline_run_id: None,
        chat_thread_id: None,
    };
    let _envelope =
        insert_session_with_started_event(&session_id, &workspace_id, &started_payload)?;
    let created = get_session_db(&session_id)?
        .ok_or_else(|| anyhow!("Failed to load created session"))?;
    let started = hydrate_session(created);

    if let Err(e) = persist_session_artifact(ship_dir, &started, "start") {
        tracing::warn!("failed to persist session artifact on start: {e}");
    }

    Ok(started)
}

fn resolve_primary_provider(
    requested: Option<String>,
    providers: &[String],
    branch: &str,
) -> Result<String> {
    if let Some(req) = requested {
        let normalized = normalize_provider_ref(&req)
            .ok_or_else(|| anyhow!("Session provider cannot be empty"))?;
        if !providers.iter().any(|p| p == &normalized) {
            return Err(anyhow!(
                "Provider '{}' not allowed for branch '{}' (allowed: {})",
                normalized, branch, providers.join(", ")
            ));
        }
        Ok(normalized)
    } else {
        providers.first().cloned()
            .ok_or_else(|| anyhow!("No providers resolved for branch '{}'", branch))
    }
}

pub fn end_session(
    ship_dir: &Path,
    branch: &str,
    request: EndSessionRequest,
) -> Result<Session> {
    let branch = ensure_branch_key(branch)?;
    let workspace_id = sanitize_file_name(branch);

    let mut active = get_active_session_db(&workspace_id)?
        .ok_or_else(|| anyhow!("No active session on branch '{}'", branch))?;

    let now = Utc::now().to_rfc3339();
    active.status = SessionStatus::Ended.to_string();
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
    let project_dir = ship_dir.parent().unwrap_or(ship_dir);
    let git_metrics = git::collect_metrics(project_dir);
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

    let ended = get_session_db(&active.id)?
        .ok_or_else(|| anyhow!("Failed to load ended workspace session"))?;
    let ended = hydrate_session(ended);

    // Telemetry already landed on sessions via apply_ended; no
    // separate session.recorded event needed (slice 04 of workspace-retirement).

    if let Err(e) = persist_session_artifact(ship_dir, &ended, "end") {
        tracing::warn!("failed to persist session artifact on end: {e}");
    }

    Ok(ended)
}
