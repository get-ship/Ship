//! Typed run-level read cores (P0.3) — the DOMAIN projection read contract.
//!
//! Build the typed Connect responses for `Summaries` / `Run` / `History` /
//! `ListRuns` straight off the normalized projections `pipeline_runs` (run-level)
//! and `pipeline_step_runs` (step-level). TOPOLOGY-INVARIANT: reading the
//! projections rather than the orchestrator's live view-model is what lets this
//! contract survive the R13 orchestrator dissolution unchanged.
//!
//! The GUI view-model tokens (tone / segments / status_text / English timeline
//! copy) and `process_state` are DELIBERATELY absent — they are not the domain
//! contract; they stay on the dual REST surface + the `*Live` JsonResponse
//! streams (still served by the shipd view helpers).

use std::path::Path;

use ship_ops_rpc::{
    ListPipelineRunsResponse, PipelineRunDetailResponse, PipelineRunHistoryResponse, PipelineRunRow,
    PipelineRunSummariesResponse, PipelineStepRunRow,
};

use crate::projections::pipeline_run::{
    PipelineRun, PipelineRunStatus, load_pipeline_run_by_id_at, load_pipeline_run_by_slug_at,
    load_pipeline_runs_at,
};
use crate::projections::pipeline_step_run::{
    PipelineStepRun, StepRunStatus, load_step_runs_for_run_at,
};

// ── status mapping (snake_case DOMAIN tokens — never the GUI kebab/tone) ──────

fn run_status_str(s: &PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::Pending => "pending",
        PipelineRunStatus::Dispatched => "dispatched",
        PipelineRunStatus::GatePending => "gate_pending",
        PipelineRunStatus::Blocked => "blocked",
        PipelineRunStatus::Completed => "completed",
        PipelineRunStatus::Merged => "merged",
        PipelineRunStatus::Failed => "failed",
        PipelineRunStatus::Cancelled => "cancelled",
    }
}

/// A run is terminal once it can no longer advance — the point at which
/// `ended_at` / `outcome` become meaningful. `Blocked` is paused-pending-human,
/// NOT terminal (it can still resume), so it carries neither.
fn is_terminal(s: &PipelineRunStatus) -> bool {
    matches!(
        s,
        PipelineRunStatus::Completed
            | PipelineRunStatus::Merged
            | PipelineRunStatus::Failed
            | PipelineRunStatus::Cancelled
    )
}

fn step_status_str(s: &StepRunStatus) -> &'static str {
    match s {
        StepRunStatus::Pending => "pending",
        StepRunStatus::Running => "running",
        StepRunStatus::Completed => "completed",
        StepRunStatus::Failed => "failed",
    }
}

// ── row mappers (domain projection record → typed wire row) ──────────────────

/// Project one `pipeline_runs` record to the typed run row. `ended_at` /
/// `outcome` are present only once the run reached a terminal status.
pub fn run_row_of(run: &PipelineRun) -> PipelineRunRow {
    let terminal = is_terminal(&run.status);
    PipelineRunRow {
        pipeline_run_id: run.pipeline_run_id.clone(),
        slug: run.slug.clone(),
        status: run_status_str(&run.status).to_string(),
        started_at: run.created_at.to_rfc3339(),
        ended_at: terminal.then(|| run.updated_at.to_rfc3339()),
        outcome: terminal.then(|| run_status_str(&run.status).to_string()),
        ..Default::default()
    }
}

/// Project one `pipeline_step_runs` record to the typed step row.
pub fn step_row_of(step: &PipelineStepRun) -> PipelineStepRunRow {
    PipelineStepRunRow {
        step_id: step.step_id.clone(),
        status: step_status_str(&step.status).to_string(),
        started_at: step.started_at.map(|t| t.to_rfc3339()),
        completed_at: step.completed_at.map(|t| t.to_rfc3339()),
        depends_on: step.depends_on.clone().unwrap_or_default(),
        step_type: step.step_type.clone(),
        attempt: step.attempt,
        ..Default::default()
    }
}

// ── pure folds (event slice → typed message; topology-invariant) ─────────────
//
// These fold an ordered event slice straight to the typed wire messages, with no
// DB. They are the always-fresh form of the projection-backed verbs above — same
// mappers, same shapes — and back the reactive Watch snapshot path, where each
// re-resolve must reflect the latest emitted event without a live projection.

/// `Run` detail by id, folded directly from an ordered event slice. `None` when
/// no `pipeline.run.started` in the slice carries `run_id`.
pub fn run_detail_from_events(
    events: &[ship_substrate::events::EventEnvelope],
    run_id: &str,
) -> Option<PipelineRunDetailResponse> {
    use crate::projections::pipeline_run::project as project_runs;
    use crate::projections::pipeline_step_run::project as project_steps;

    let runs = project_runs(events);
    let run = runs.get(run_id)?;
    let step_map = project_steps(events);
    let mut steps: Vec<PipelineStepRunRow> = step_map
        .iter()
        .filter(|((r, _), _)| r == run_id)
        .map(|(_, s)| step_row_of(s))
        .collect();
    steps.sort_by(|a, b| a.step_id.cmp(&b.step_id));
    Some(PipelineRunDetailResponse { run: run_row_of(run).into(), steps, ..Default::default() })
}

/// `Summaries` (run-level rows, newest first) folded directly from an ordered
/// event slice.
pub fn summaries_from_events(
    events: &[ship_substrate::events::EventEnvelope],
) -> PipelineRunSummariesResponse {
    use crate::projections::pipeline_run::project as project_runs;

    let mut runs: Vec<PipelineRun> = project_runs(events).into_values().collect();
    runs.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.pipeline_run_id.cmp(&b.pipeline_run_id))
    });
    PipelineRunSummariesResponse {
        runs: runs.iter().map(run_row_of).collect(),
        ..Default::default()
    }
}

// ── shared loaders ───────────────────────────────────────────────────────────

/// Every run, newest first by creation, stable tiebreak on id.
fn sorted_runs(ship_dir: &Path) -> Vec<PipelineRun> {
    let mut runs: Vec<PipelineRun> = load_pipeline_runs_at(ship_dir)
        .unwrap_or_default()
        .into_values()
        .collect();
    runs.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.pipeline_run_id.cmp(&b.pipeline_run_id))
    });
    runs
}

/// The typed step rows for one run, ordered by step id for determinism.
fn step_rows_for(ship_dir: &Path, run_id: &str) -> Vec<PipelineStepRunRow> {
    let mut steps = load_step_runs_for_run_at(ship_dir, run_id).unwrap_or_default();
    steps.sort_by(|a, b| a.step_id.cmp(&b.step_id));
    steps.iter().map(step_row_of).collect()
}

// ── verbs ────────────────────────────────────────────────────────────────────

/// `Summaries` — run-level rows for every run, newest first.
pub fn summaries(ship_dir: &Path) -> PipelineRunSummariesResponse {
    PipelineRunSummariesResponse {
        runs: sorted_runs(ship_dir).iter().map(run_row_of).collect(),
        ..Default::default()
    }
}

/// `Run` — one run's detail: the run row + its step rows. `None` when no run
/// carries `slug`.
pub fn run_detail(ship_dir: &Path, slug: &str) -> Option<PipelineRunDetailResponse> {
    let run = load_pipeline_run_by_slug_at(ship_dir, slug).ok().flatten()?;
    let steps = step_rows_for(ship_dir, &run.pipeline_run_id);
    Some(PipelineRunDetailResponse {
        run: run_row_of(&run).into(),
        steps,
        ..Default::default()
    })
}

/// `Run` keyed by `pipeline_run_id` — the same run detail as [`run_detail`] but
/// looked up by id rather than slug. The Watch surface (P0.4) keys on the
/// pipeline_run_id (events carry it; slug is not on every event), so the typed
/// snapshot is built off the id. `None` when no run carries `run_id`.
pub fn run_detail_by_id(ship_dir: &Path, run_id: &str) -> Option<PipelineRunDetailResponse> {
    let run = load_pipeline_run_by_id_at(ship_dir, run_id).ok().flatten()?;
    let steps = step_rows_for(ship_dir, &run.pipeline_run_id);
    Some(PipelineRunDetailResponse {
        run: run_row_of(&run).into(),
        steps,
        ..Default::default()
    })
}

/// `History` — paginated run-level history rows (terminal-category runs),
/// filtered by `pipeline` (exact pipeline name) and `q` (slug/agent substring).
pub fn history(
    ship_dir: &Path,
    q: Option<String>,
    pipeline: Option<String>,
    offset: usize,
    limit: usize,
) -> PipelineRunHistoryResponse {
    use crate::view::state::{Filter, list_pipeline_runs, partition_runs};

    let limit = if limit == 0 { 50 } else { limit.clamp(1, 200) };
    let filter = Filter {
        pipeline: pipeline.filter(|s| !s.is_empty()),
        ..Filter::default()
    };
    let all = list_pipeline_runs(ship_dir, &filter); // sorted updated_at desc
    let (_active, history_all) = partition_runs(all);

    let q_lower = q.unwrap_or_default().to_lowercase();
    let filtered: Vec<PipelineRun> = history_all
        .into_iter()
        .filter(|j| {
            q_lower.is_empty()
                || j.slug.to_lowercase().contains(&q_lower)
                || j.agent.to_lowercase().contains(&q_lower)
        })
        .collect();
    let total = filtered.len();
    let offset = offset.min(total);
    let runs: Vec<PipelineRunRow> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|r| run_row_of(&r))
        .collect();

    PipelineRunHistoryResponse {
        runs,
        total: total as u64,
        offset: offset as u64,
        limit: limit as u64,
        ..Default::default()
    }
}

/// `ListRuns` — every run with its step rows (newest first).
pub fn list_runs(ship_dir: &Path) -> ListPipelineRunsResponse {
    let runs = sorted_runs(ship_dir)
        .iter()
        .map(|run| PipelineRunDetailResponse {
            run: run_row_of(run).into(),
            steps: step_rows_for(ship_dir, &run.pipeline_run_id),
            ..Default::default()
        })
        .collect();
    ListPipelineRunsResponse { runs, ..Default::default() }
}

#[cfg(test)]
#[path = "tests_read_core.rs"]
mod tests;
