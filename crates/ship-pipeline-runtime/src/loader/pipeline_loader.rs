//! Pipeline loading and resolution.
//!
//! Resolves a pipeline for a job from (in priority order):
//! 1. Already-set `steps` on the payload
//! 2. The descriptor's `ExtensionLibrary::pipelines` (TS declare-forms),
//!    looked up by `payload.pipeline_name` or by the `pipeline: <name>`
//!    reference in spec frontmatter
//! 3. None → single `Agent` step using the job's declared agent
//!
//! Post format-migration Wave 4 phase A: pipelines arrive through the
//! descriptor's TS-merged `Vec<compiler::PipelineDef>` (the
//! `definePipeline` declare-form in the space's `ship.config.ts`).
//! The previous `.ship/pipelines/<name>.jsonc` file walk is gone — see
//! `census-format-migration.md` Section 4 row 3.
//!
//! Lives at `crate::loader::pipeline_loader` (not under `orchestrator`)
//! so `run_core::dispatch_one` can build the dispatch envelope without
//! the `ship-code` feature. Pure spec validation; no process or session
//! types are touched here.
//!
//! ## UNMAPPED COUPLING (per spec stop-rule)
//!
//! The runtime's `Pipeline` shape (`ship_substrate::events::pipeline::Pipeline`,
//! the `step_type/agent/goal/command` per-step fields) does not map 1:1 onto
//! the descriptor's `compiler::PipelineDef` shape (`step_specs: [{agent,
//! harness, prompt}]`). A faithful converter is its own design pass —
//! it would also need to fold the descriptor's per-`step_spec` prompt bodies
//! into the runtime's `step_specs: Vec<String>` field used by
//! `pipeline_specs::write_pipeline_specs`. Until that lands, this loader
//! carries a no-op `load_pipeline_def_by_name` (the descriptor is in scope
//! of `ensure_pipeline`'s callers) and `ensure_pipeline` documents the
//! `payload.steps`-first contract. Surfaced per the Wave 4 phase A
//! stop-rule; see the job spec at
//! `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-fm-w4-noun-cut.md`
//! and the W4 spec section "STOP if any consumer turns out to read a
//! noun's files DIRECTLY".

use ship_substrate::events::pipeline::PipelineRunStartedPayload;
use ship_pipelines::PipelineMeta;

mod resolve;
use resolve::resolve_steps;

/// Enrich a job with a pipeline and validate it. Resolution order:
/// 1. Already has `steps` → use it
/// 2. The `pipelines` slice (descriptor-sourced) is consulted via
///    `resolve_steps` for `payload.pipeline_name` / spec-frontmatter
///    references — see [`resolve::resolve_steps`].
/// 3. None → single `Agent` step from `payload.agent`
///
/// Returns `Err` if the resolved pipeline fails validation — currently, any
/// agent/gate step without an `agent` declared, or any malformed gate block.
pub fn ensure_pipeline(
    mut payload: PipelineRunStartedPayload,
    pipelines: &[compiler::PipelineDef],
) -> Result<PipelineRunStartedPayload, String> {
    resolve_steps(&mut payload, pipelines);
    interpolate_pipeline(&mut payload);
    // Re-sync payload.agent with the (possibly now-interpolated) first step's agent.
    if let Some(pipeline) = &payload.steps
        && let Some(root) = pipeline.root_steps().first()
        && let Some(agent) = &root.agent
    {
        payload.agent = agent.clone();
    }
    if let Some(pipeline) = &payload.steps {
        ship_pipelines::validate::validate_pipeline(pipeline).map_err(|e| e.to_string())?;
    }
    Ok(payload)
}

fn interpolate_pipeline(payload: &mut PipelineRunStartedPayload) {
    let Some(pipeline) = payload.steps.as_mut() else { return };
    let Some(vars) = payload.vars.as_ref() else { return };
    if vars.is_empty() {
        return;
    }
    let jinja_vars: std::collections::HashMap<String, serde_json::Value> = vars
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    let render = |s: &mut Option<String>| {
        if let Some(v) = s
            && (v.contains("{{") || v.contains("{%"))
        {
            *v = resolve_template(v, &jinja_vars);
        }
    };
    for step in &mut pipeline.steps {
        render(&mut step.agent);
        render(&mut step.model);
        render(&mut step.harness);
        render(&mut step.goal);
        render(&mut step.command);
    }
    if let Some(step_specs) = payload.step_specs.as_mut() {
        for spec in step_specs.iter_mut() {
            if spec.contains("{{") || spec.contains("{%") {
                *spec = resolve_template(spec, &jinja_vars);
            }
        }
    }
}

fn resolve_template(
    content: &str,
    vars: &std::collections::HashMap<String, serde_json::Value>,
) -> String {
    use minijinja::{Environment, UndefinedBehavior};
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    match env.render_str(content, vars) {
        Ok(result) => result,
        Err(_) => content.to_string(),
    }
}

/// Look up a named pipeline from the descriptor's
/// `ExtensionLibrary::pipelines`. Returns `None` when the name isn't
/// present — the caller falls through to its single-step default.
///
/// The runtime `Pipeline` shape conversion is a separate design pass
/// (see the `UNMAPPED COUPLING` note at the top of this file); this
/// stub returns the descriptor's `PipelineMeta` projection so the
/// `merge_meta_vars` flow stays intact until the full converter lands.
pub fn load_pipeline_def_by_name(
    pipelines: &[compiler::PipelineDef],
    name: &str,
) -> Option<(PipelineMeta, String)> {
    let def = pipelines.iter().find(|p| p.name == name)?;
    let step_specs: Vec<String> = def
        .steps
        .iter()
        .flat_map(|s| s.step_specs.iter().map(|spec| spec.prompt.clone().unwrap_or_default()))
        .collect();
    let meta = PipelineMeta {
        name: Some(def.name.clone()),
        description: None,
        vars: None,
        timeout: None,
        max_retries: None,
        auto_merge: Some(def.auto_merge),
        step_specs: if step_specs.is_empty() { None } else { Some(step_specs) },
    };
    Some((meta, def.name.clone()))
}

/// Merge pipeline-level default vars into the job payload.
pub fn merge_meta_vars(payload: &mut PipelineRunStartedPayload, meta: &PipelineMeta) {
    if let Some(pipeline_vars) = &meta.vars {
        let run_vars = payload.vars.get_or_insert_with(Default::default);
        for (k, v) in pipeline_vars {
            run_vars.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    if payload.step_specs.is_none()
        && let Some(specs) = &meta.step_specs
    {
        payload.step_specs = Some(specs.clone());
    }
}
