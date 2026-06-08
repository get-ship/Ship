//! `pipelines.focus(name)` — per-pipeline detail page data.
//!
//! Joins the parsed pipeline doc (steps, auto-merge) with the `PipelineDef`
//! aggregate stats from `views::state` and partitions the `PipelineRun`
//! records that reference this pipeline into the active + history streams
//! the studio focus view renders. Returns `None` when no pipeline file
//! resolves to `name` (the live resolver surfaces that as
//! `ConnectError::not_found`; the controller renders the error state).
//!
//! Pure / sync — no I/O beyond reading the pipeline file off disk. The
//! `live_resolvers` registry re-runs this on every kernel `config` event,
//! so per-render cost matters.

use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ship_pipeline_runtime::projections::pipeline_run::PipelineRun;

use crate::views::state::{
    Filter, PipelineDef, aggregate_pipeline_stats, list_pipeline_defs, list_pipeline_runs,
    partition_runs, pipeline_name_for_job_public,
};

use super::shape::{
    RunView, current_project_workspace_id, find_pipeline_file, harnesses_on_path, to_run_view,
};

/// A single step in the parsed pipeline doc — the JSONC shape the studio
/// focus view renders (id, agent, run, gate, after, model). Matches the
/// `StepDef` the controller declares.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FocusStep {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate: Option<FocusGate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FocusGate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_changed: Option<FocusGateRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbid_changed: Option<FocusGateRule>,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FocusGateRule {
    pub match_: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,
}

impl FocusGateRule {
    /// Parse a gate rule. Pipeline files store `require_changed` /
    /// `forbid_changed` as either:
    ///
    ///   - a single object: `{ "match": ["**/*.rs"], "min": 1 }` (the
    ///     form the loader normalizes to)
    ///   - an array of such objects (legacy / unnormalized form some
    ///     hand-authored files still use)
    ///
    /// Always picks the FIRST rule in either form — multi-rule
    /// combinations aren't rendered by the studio view today, and
    /// the runtime loader collapses them the same way.
    fn from_value(v: &serde_json::Value) -> Option<Self> {
        let first = if let Some(arr) = v.as_array() {
            arr.first()?
        } else {
            v
        };
        let arr = first.get("match").and_then(|x| x.as_array())?;
        let match_: Vec<String> = arr
            .iter()
            .filter_map(|s| s.as_str().map(str::to_string))
            .collect();
        let min = first.get("min").and_then(|x| x.as_u64()).map(|n| n as u32);
        Some(FocusGateRule { match_, min })
    }
}

/// Document fragment the studio focus view edits — the file's structured
/// body. `raw` is the on-disk text (preserves comments for the JSONC
/// editor) and `format` names the parser that produced `doc` (jsonc/json/
/// yaml — the loader walks extensions in that order).
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FocusDoc {
    pub steps: Vec<FocusStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_merge: Option<bool>,
}

/// Per-pipeline detail payload — the studio `studio/pipelines/focus/[name]`
/// view binds to this. Layout mirrors `SummariesCtx` but scoped to a single
/// pipeline: def with stats, the parsed doc, the file text, recent runs,
/// active runs for this pipeline, providers detected on PATH, and the
/// current project workspace id (for edit/delete permission checks).
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "specta", specta(rename = "PipelineFocusCtx"))]
pub struct FocusCtx {
    pub def: PipelineDef,
    pub doc: FocusDoc,
    pub raw: String,
    pub format: String,
    pub workspace_id: Option<String>,
    pub runs: Vec<RunView>,
    /// Active runs (dispatched / pending / gate-pending) for THIS pipeline.
    /// The view renders the "Ops · active now" rail from this set.
    pub active_for_pipeline: Vec<RunView>,
    /// Non-active runs for THIS pipeline (history). Currently unused by the
    /// template — `runs` is the rendered history stream — kept for the ops
    /// view's future partitioned rendering.
    pub other_for_pipeline: Vec<RunView>,
    /// `true` when the agent has flagged this project as a live-ops seat
    /// (only the live-ops shell uses the "active now" rail). Always `false`
    /// for the standard studio view — the shell sets it from a per-user
    /// preference and is wired separately; the live resolver doesn't need
    /// to know about it.
    pub ops_mode: bool,
    /// Provider ids detected on PATH — powers the dispatch dialog's
    /// provider-override `<select>`. Re-derived every resolve so a freshly
    /// installed harness shows up without a daemon restart.
    pub providers: Vec<String>,
}

/// Build the focus payload for `name`. Returns `None` when no `.ship/
/// pipelines/<name>.{yaml,yml,jsonc,json}` file is found — the live
/// resolver surfaces that as `connectrpc::ConnectError::not_found`.
pub fn build(name: &str, ship_dir: &Path) -> Option<FocusCtx> {
    // Prefer the daemon's known ship_dir so tests / multi-project
    // installs don't depend on CWD; fall through to the CWD walk if
    // the daemon's dir has no pipelines/ subtree (legacy / global
    // install).
    let path = find_pipeline_file(name, Some(ship_dir))?;
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let format = path
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "jsonc".to_string());

    let parsed: serde_json::Value = match format.as_str() {
        "yaml" | "yml" => serde_yaml::from_str(&raw).unwrap_or(serde_json::Value::Null),
        // jsonc/json: defer to the canonical substrate stripper (handles
        // // line + /* */ block comments + trailing commas), then parse.
        // Anything that still fails yields an empty steps list rather than
        // a 500 — the editor surfaces the unparsed file via `raw` for the
        // user to fix.
        _ => {
            let cleaned = ship_substrate::fs_util::strip_jsonc_comments(&raw);
            serde_json::from_str(&cleaned).unwrap_or(serde_json::Value::Null)
        }
    };
    let doc = parse_doc(&parsed);

    // Aggregate run stats for the single pipeline we care about: build a
    // one-element defs slice, fold in jobs whose slug-name prefix matches.
    let mut defs = list_pipeline_defs(ship_dir);
    let all_runs = list_pipeline_runs(ship_dir, &Filter::default());
    aggregate_pipeline_stats(&mut defs, all_runs.iter());
    let def = defs
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| PipelineDef {
            name: name.to_string(),
            description: String::new(),
            step_count: doc.steps.len(),
            active_runs: 0,
            total_runs: 0,
            success_runs: 0,
            failed_runs: 0,
            success_rate: 0.0,
            last_run_at: None,
            last_run_status: None,
            last_run_tone: None,
            last_run_slug: None,
            recent_outcomes: Vec::new(),
        });

    let runs: Vec<PipelineRun> = all_runs
        .into_iter()
        .filter(|j| pipeline_name_for_job_public(j).as_deref() == Some(name))
        .collect();
    let (active, history) = partition_runs(runs);

    let active_for_pipeline: Vec<RunView> =
        active.iter().map(|j| to_run_view(j, true)).collect();
    let other_for_pipeline: Vec<RunView> =
        history.iter().map(|j| to_run_view(j, false)).collect();
    let runs_view: Vec<RunView> = history
        .iter()
        .take(50)
        .map(|j| to_run_view(j, false))
        .collect();

    // Providers = harnesses on PATH (binary-detected) ∪ provider ids the
    // doc declares on its steps. Dedup via BTreeSet, ordered for stable
    // render.
    let mut providers: BTreeSet<String> = harnesses_on_path().into_iter().collect();
    for step in &doc.steps {
        if let Some(p) = &step.provider {
            providers.insert(p.clone());
        }
    }

    Some(FocusCtx {
        def,
        doc,
        raw,
        format,
        workspace_id: current_project_workspace_id(),
        runs: runs_view,
        active_for_pipeline,
        other_for_pipeline,
        ops_mode: false,
        providers: providers.into_iter().collect(),
    })
}

/// Parse the pipeline doc body. Tolerates malformed input — yields an
/// empty step list rather than failing the whole op.
fn parse_doc(value: &serde_json::Value) -> FocusDoc {
    let steps = value
        .get("steps")
        .and_then(|s| s.as_array())
        .map(|arr| arr.iter().filter_map(parse_step).collect())
        .unwrap_or_default();
    let auto_merge = value.get("auto_merge").and_then(|v| v.as_bool());
    FocusDoc { steps, auto_merge }
}

fn parse_step(v: &serde_json::Value) -> Option<FocusStep> {
    let id = v
        .get("id")
        .and_then(|s| s.as_str())
        .map(str::to_string)
        .unwrap_or_default();
    let agent = v.get("agent").and_then(|s| s.as_str()).map(str::to_string);
    let provider = v.get("provider").and_then(|s| s.as_str()).map(str::to_string);
    let model = v.get("model").and_then(|s| s.as_str()).map(str::to_string);
    let run = v.get("run").and_then(|s| s.as_str()).map(str::to_string);
    let gate = v.get("gate").map(|g| {
        let require_changed = g.get("require_changed").and_then(FocusGateRule::from_value);
        let forbid_changed = g.get("forbid_changed").and_then(FocusGateRule::from_value);
        FocusGate { require_changed, forbid_changed }
    });
    let after = v.get("after").and_then(|s| s.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    });
    // Drop stub entries (e.g. `{}`) so the template's `{#for step of doc.steps}`
    // never has to render blanks. Real steps always have an id or an action.
    if id.is_empty() && agent.is_none() && run.is_none() {
        return None;
    }
    Some(FocusStep { id, agent, provider, model, run, gate, after })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_step_skips_garbage() {
        let v = serde_json::json!({});
        assert!(parse_step(&v).is_none());
        let v = serde_json::json!({ "id": "build" });
        assert_eq!(parse_step(&v).unwrap().id, "build");
    }

    #[test]
    fn parse_doc_reads_steps_and_auto_merge() {
        let v = serde_json::json!({
            "auto_merge": true,
            "steps": [
                { "id": "build", "run": "echo hi" },
                { "id": "review", "agent": "gatekeeper", "gate": { "require_changed": [{ "match": ["**/*.rs"], "min": 1 }] } },
            ],
        });
        let d = parse_doc(&v);
        assert_eq!(d.steps.len(), 2);
        assert_eq!(d.auto_merge, Some(true));
        let gate = d.steps[1].gate.as_ref().and_then(|g| g.require_changed.as_ref()).unwrap();
        assert_eq!(gate.match_, vec!["**/*.rs".to_string()]);
        assert_eq!(gate.min, Some(1));
    }
}
