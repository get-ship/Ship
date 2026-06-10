//! `pipelines.summaries` — data for the pipelines list view.
//! Relocated from `shipd::services::pipelines::summaries` (L2 → L1).

use serde::Serialize;
use std::collections::BTreeSet;
use std::path::Path;

use crate::projections::pipeline_run::PipelineRunStatus;

use super::state::{
    Filter, PipelineDef, aggregate_pipeline_stats, list_pipeline_defs, list_pipeline_runs,
    partition_runs, status_token,
};

use super::shape::{
    RunView, Stats, StatusPill, current_project_workspace_id, harnesses_on_path, to_run_view,
};

#[derive(Serialize)]
pub struct SummariesCtx {
    pub stats: Stats,
    pub stats_subtitle: String,
    pub agents: Vec<String>,
    pub defs: Vec<PipelineDef>,
    pub active_runs: Vec<RunView>,
    pub history_runs: Vec<RunView>,
    pub status_pills: Vec<StatusPill>,
    pub harnesses: Vec<String>,
    pub workspace_id: Option<String>,
    pub filter: Filter,
    pub agent_filter: String,
    pub active_tab: String,
}

fn pluralize(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

fn build_subtitle(stats: &Stats) -> String {
    format!(
        "{} · {} · {} in history",
        pluralize(stats.definitions, "definition"),
        pluralize(stats.active, "active"),
        stats.history
    )
}

pub fn build(
    ship_dir: &Path,
    filter: Filter,
    active_tab: Option<String>,
) -> SummariesCtx {
    let mut defs = list_pipeline_defs(ship_dir);
    let all_runs = list_pipeline_runs(ship_dir, &Filter::default());
    aggregate_pipeline_stats(&mut defs, all_runs.iter());
    let agents: Vec<String> = all_runs
        .iter()
        .map(|j| j.agent.clone())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let runs = list_pipeline_runs(ship_dir, &filter);
    let (active, history) = partition_runs(runs);

    let history_runs: Vec<RunView> = history
        .iter()
        .take(50)
        .filter(|j| {
            filter
                .status
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| status_token(&j.status) == s)
                .unwrap_or(true)
        })
        .map(|j| to_run_view(j, false))
        .collect();
    let active_runs: Vec<RunView> = active.iter().map(|j| to_run_view(j, true)).collect();

    let completed = history
        .iter()
        .filter(|j| matches!(j.status, PipelineRunStatus::Completed | PipelineRunStatus::Merged))
        .count();
    let failed = history
        .iter()
        .filter(|j| matches!(j.status, PipelineRunStatus::Failed))
        .count();

    let stats = Stats {
        definitions: defs.len(),
        active: active.len(),
        completed,
        failed,
        history: history.len(),
    };
    let stats_subtitle = build_subtitle(&stats);

    let all_by_status = |want: PipelineRunStatus| -> usize {
        active
            .iter()
            .chain(history.iter())
            .filter(|j| std::mem::discriminant(&j.status) == std::mem::discriminant(&want))
            .count()
    };
    let status_pills = vec![
        StatusPill { label: "Pending",  tone: "muted",       count: all_by_status(PipelineRunStatus::Pending) },
        StatusPill { label: "Running",  tone: "info",        count: all_by_status(PipelineRunStatus::Dispatched) },
        StatusPill { label: "Gate",     tone: "warning",     count: all_by_status(PipelineRunStatus::GatePending) },
        StatusPill { label: "Done",     tone: "success",     count: all_by_status(PipelineRunStatus::Completed) },
        StatusPill { label: "Merged",   tone: "accent",      count: all_by_status(PipelineRunStatus::Merged) },
        StatusPill { label: "Blocked",  tone: "warning",     count: all_by_status(PipelineRunStatus::Blocked) },
        StatusPill { label: "Failed",   tone: "destructive", count: all_by_status(PipelineRunStatus::Failed) },
    ];

    let agent_filter = filter.agent.clone().unwrap_or_default();
    let active_tab = active_tab
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "active".to_string());

    SummariesCtx {
        stats,
        stats_subtitle,
        agents,
        defs,
        active_runs,
        history_runs,
        status_pills,
        harnesses: harnesses_on_path(),
        workspace_id: current_project_workspace_id(),
        filter,
        agent_filter,
        active_tab,
    }
}
