//! `pipelines.history(limit?, offset?, q?, pipeline?)` — paginated
//! history rows for the dedicated history view.

use ship_pipeline_runtime::projections::pipeline_run::PipelineRun;
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::Path;

use crate::views::state::{
    Filter, list_pipeline_runs, partition_runs, status_tone, pipeline_name_for_job_public,
};

use super::shape::{format_duration_between, run_state_token};

#[derive(Serialize, Clone)]
pub struct HistoryRow {
    pub slug: String,
    pub pipeline: String,
    pub agent: String,
    pub state: &'static str,
    pub tone: &'static str,
    pub duration: String,
    pub updated_at: String,
    pub branch: String,
}

#[derive(Serialize)]
pub struct HistoryCtx {
    pub q: String,
    pub pipeline_filter: String,
    pub rows: Vec<HistoryRow>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_prev: bool,
    pub has_next: bool,
    pub pipelines: Vec<String>,
}

pub fn build(
    ship_dir: &Path,
    q: Option<String>,
    pipeline: Option<String>,
    offset: usize,
    limit: usize,
) -> HistoryCtx {
    let limit = limit.clamp(1, 200);

    let filter = Filter {
        pipeline: pipeline.clone().filter(|s| !s.is_empty()),
        ..Filter::default()
    };
    let all = list_pipeline_runs(ship_dir, &filter);
    let (_active, history_all) = partition_runs(all);

    let q_str = q.clone().unwrap_or_default();
    let q_lower = q_str.to_lowercase();
    let filtered: Vec<PipelineRun> = history_all
        .into_iter()
        .filter(|j| {
            if q_lower.is_empty() {
                return true;
            }
            let pname = pipeline_name_for_job_public(j).unwrap_or_default();
            j.slug.to_lowercase().contains(&q_lower)
                || j.agent.to_lowercase().contains(&q_lower)
                || pname.to_lowercase().contains(&q_lower)
        })
        .collect();
    let total = filtered.len();

    let offset = offset.min(total);
    let window: Vec<PipelineRun> = filtered.into_iter().skip(offset).take(limit).collect();

    let pipelines: Vec<String> = {
        let unfiltered = list_pipeline_runs(ship_dir, &Filter::default());
        let (_a, h) = partition_runs(unfiltered);
        let mut set = BTreeSet::new();
        for j in &h {
            if let Some(p) = pipeline_name_for_job_public(j) {
                set.insert(p);
            }
        }
        set.into_iter().collect()
    };

    let rows: Vec<HistoryRow> = window
        .into_iter()
        .map(|j| HistoryRow {
            slug: j.slug.clone(),
            pipeline: pipeline_name_for_job_public(&j).unwrap_or_default(),
            agent: j.agent.clone(),
            state: run_state_token(&j.status),
            tone: status_tone(&j.status),
            duration: format_duration_between(&j.created_at, &j.updated_at),
            updated_at: j.updated_at.to_rfc3339(),
            branch: j.branch.clone(),
        })
        .collect();

    let has_prev = offset > 0;
    let has_next = offset + limit < total;

    HistoryCtx {
        q: q_str,
        pipeline_filter: pipeline.unwrap_or_default(),
        rows,
        total,
        offset,
        limit,
        has_prev,
        has_next,
        pipelines,
    }
}
