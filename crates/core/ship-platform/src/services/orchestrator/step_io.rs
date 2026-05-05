//! Step I/O — consumes resolution, required-output checking, and promotes.
//!
//! Implements the three operations that connect pipeline steps via artifacts:
//!
//! 1. **Resolve consumes** — before a step dispatches, look up the artifacts
//!    produced by upstream steps, copy them to `.artifacts/input/<name>`, and
//!    return their metadata for inclusion in the dispatch prompt.
//!
//! 2. **Check required outputs** — when `step.done_declared` arrives, verify
//!    that every `produces` entry with `required: true` appears in the
//!    agent's `outputs` map. Missing required outputs fail the step.
//!
//! 3. **Run promotes** — after a step advances, promote declared outputs to
//!    their target scope. Failures are non-fatal (warn, continue pipeline).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::{ArtifactRecord, ArtifactScope, ArtifactService};
use ship_substrate::events::job::{OutputScope, PipelineStep, StepOutput};
use ship_substrate::events::{EventFilter, EventStore, SqliteEventStore};

/// An artifact resolved from a prior step's `produces` declaration.
pub struct ResolvedInput {
    pub name: String,
    pub artifact_id: String,
    /// Path relative to the worktree root (`.artifacts/input/<name>`).
    pub worktree_path: String,
    pub size: u64,
    pub mime: String,
    /// For lane-scoped `.failed` markers: a short human-readable reason
    /// extracted from the sentinel artifact. `None` for regular outputs.
    pub failure_reason: Option<String>,
}

/// Resolve a step's `consumes` list to artifact records.
///
/// For each consumed name this function:
/// 1. Locates the producing step's `step.done_declared` event.
/// 2. Extracts the artifact_id from its `outputs` map.
/// 3. Fetches the artifact record from the artifact service.
/// 4. Writes the blob to `<worktree>/.artifacts/input/<name>` (hardlink or copy).
///
/// Returns `Err` with a human-readable message when any input cannot be resolved.
pub fn resolve_consumes(
    step: &PipelineStep,
    pipeline: &ship_substrate::events::job::Pipeline,
    job_id: &str,
    worktree_path: &Path,
) -> Result<Vec<ResolvedInput>, String> {
    if step.consumes.is_empty() {
        return Ok(vec![]);
    }

    let ship_dir = ship_substrate::project::get_global_dir()
        .map_err(|e| format!("resolve_consumes: cannot get ship dir: {e}"))?;
    let mut svc = ship_substrate::data::DataService::open(&ship_dir)
        .map_err(|e| format!("resolve_consumes: cannot open data service: {e}"))?;
    let art_svc = ArtifactService::new(&ship_dir);

    let mut resolved = Vec::new();

    for consumed_name in &step.consumes {
        // Lane-scoped references like "lane-claude.draft" resolve via the lane's
        // step.done_declared event (step_id = "<fanout_step_id>.<lane_id>").
        let (producer_step_id, output_key) = if let Some((lane_id, output_name)) = parse_lane_scoped_ref(consumed_name) {
            let fanout_step = find_fanout_step_for_lane(pipeline, lane_id)
                .ok_or_else(|| format!(
                    "step '{}' consumes '{consumed_name}': no fanout step found for lane '{lane_id}'",
                    step.id
                ))?;
            (format!("{}.{}", fanout_step.id, lane_id), output_name.to_string())
        } else {
            // Normal path: find the step that produces this name.
            let producer = pipeline
                .steps
                .iter()
                .find(|s| s.produces.iter().any(|o| o.name == *consumed_name))
                .ok_or_else(|| format!(
                    "step '{}' consumes '{consumed_name}' but no producer found in pipeline",
                    step.id
                ))?;
            (producer.id.clone(), consumed_name.clone())
        };

        // Look up the artifact_id from the producer's step.done_declared event.
        let artifact_id =
            find_artifact_id_in_events(job_id, &producer_step_id, &output_key).ok_or_else(|| {
                format!(
                    "artifact '{consumed_name}' not found in step '{producer_step_id}' outputs — \
                     the step may not have uploaded it or omitted it from step_done.outputs",
                )
            })?;

        // Fetch the artifact record.
        let record = art_svc
            .get_record(&mut svc, &artifact_id)
            .map_err(|e| {
                format!("resolve_consumes: artifact lookup failed for '{consumed_name}': {e}")
            })?
            .ok_or_else(|| {
                format!(
                    "artifact '{consumed_name}' (id={artifact_id}) not found in artifact store"
                )
            })?;

        // Write to .artifacts/input/<name>.
        write_input_file(worktree_path, consumed_name, &record, &ship_dir)
            .map_err(|e| format!("resolve_consumes: failed writing input '{consumed_name}': {e}"))?;

        // Extract a failure reason from `<lane>.failed` sentinel artifacts.
        let failure_reason = if output_key == "failed" {
            let p = worktree_path.join(format!(".artifacts/input/{consumed_name}"));
            read_failure_reason(&p)
        } else {
            None
        };

        resolved.push(ResolvedInput {
            name: consumed_name.clone(),
            artifact_id,
            worktree_path: format!(".artifacts/input/{consumed_name}"),
            size: record.size,
            mime: record.mime,
            failure_reason,
        });
    }

    Ok(resolved)
}

/// Check that all `required: true` outputs were reported in `step_done.outputs`.
///
/// Returns a failure message string when any required output is missing,
/// or `None` when the step satisfied its contract.
pub fn check_required_outputs(
    produces: &[StepOutput],
    outputs: &BTreeMap<String, String>,
) -> Option<String> {
    for output in produces {
        if output.required && !outputs.contains_key(&output.name) {
            return Some(format!(
                "required output '{}' was not produced — \
                 upload it via artifact_upload and include its id in step_done.outputs",
                output.name
            ));
        }
    }
    None
}

/// Promote declared outputs to wider scopes after a step succeeds.
///
/// Failures are non-fatal: each failure emits a warning and the next
/// promote entry is attempted. The pipeline advances regardless.
pub async fn run_step_promotes(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    job_id: &str,
    step: &PipelineStep,
) {
    if step.promotes.is_empty() {
        return;
    }

    let ship_dir = match ship_substrate::project::get_global_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(job_id, step_id = step.id, "promotes: cannot get ship dir: {e}");
            return;
        }
    };
    let mut svc = match ship_substrate::data::DataService::open(&ship_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(job_id, step_id = step.id, "promotes: cannot open data service: {e}");
            return;
        }
    };
    let art_svc = ArtifactService::new(&ship_dir);
    let project_id = ship_substrate::project::project_slug_from_ship_dir(&ship_dir);

    // This step's own outputs (for promotes of self-produced artifacts).
    let own_outputs = get_step_outputs_from_events(job_id, &step.id);

    // Load pipeline to resolve consumed artifacts from upstream steps.
    let pipeline = load_pipeline_for_job(job_id);

    for promote in &step.promotes {
        // Find the artifact ID: check own outputs first, then look up the
        // producer step if this is a consumed (pass-through) promote.
        let artifact_id = if let Some(id) = own_outputs.get(&promote.name) {
            id.clone()
        } else if let Some(id) =
            find_artifact_for_promote(&promote.name, job_id, step, pipeline.as_ref())
        {
            id
        } else {
            tracing::warn!(
                job_id, step_id = step.id, name = promote.name,
                "promotes: artifact not found in step outputs or upstream, skipping"
            );
            continue;
        };

        let target = match output_scope_to_artifact_scope(&promote.target_scope, &project_id) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    job_id, step_id = step.id, name = promote.name,
                    "promotes: invalid target scope: {e}"
                );
                continue;
            }
        };

        match art_svc.promote(&mut svc, kernel, &artifact_id, target).await {
            Ok(r) => tracing::info!(
                job_id, step_id = step.id, name = promote.name,
                new_id = r.id, "promotes: artifact promoted"
            ),
            Err(e) => tracing::warn!(
                job_id, step_id = step.id, name = promote.name,
                "promotes: promotion failed (non-fatal): {e}"
            ),
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Scan the event store for a `step.done_declared` event from the given step
/// and return the artifact_id mapped to `output_name` in its `outputs` field.
pub fn find_artifact_id_in_events(
    job_id: &str,
    step_id: &str,
    output_name: &str,
) -> Option<String> {
    let store = SqliteEventStore::new().ok()?;
    let envelopes = store
        .query(&EventFilter {
            entity_id: Some(job_id.to_string()),
            event_type: Some("step.done_declared".to_string()),
            ..Default::default()
        })
        .ok()?;
    for env in envelopes {
        let payload: serde_json::Value =
            serde_json::from_str(&env.payload_json).ok()?;
        if payload.get("step_id").and_then(|v| v.as_str()) == Some(step_id) {
            let id = payload
                .get("outputs")
                .and_then(|o| o.get(output_name))
                .and_then(|v| v.as_str())
                .map(str::to_string)?;
            return Some(id);
        }
    }
    None
}

/// Re-export of `ship_pipelines::validate::parse_lane_scoped_ref`. Kept
/// here under the previous name so existing callers
/// (`pipeline_specs.rs::parse_lane_scoped_ref`) keep compiling without
/// change.
pub use ship_pipelines::validate::parse_lane_scoped_ref;

/// Find the fanout step that owns a given lane_id.
pub fn find_fanout_step_for_lane<'a>(
    pipeline: &'a ship_substrate::events::job::Pipeline,
    lane_id: &str,
) -> Option<&'a ship_substrate::events::job::PipelineStep> {
    pipeline.steps.iter().find(|s| {
        s.fanout.as_ref().map_or(false, |f| f.lanes.iter().any(|l| l.id == lane_id))
    })
}

/// Retrieve the full `outputs` map from a step's `step.done_declared` event.
pub fn get_step_outputs_from_events(job_id: &str, step_id: &str) -> BTreeMap<String, String> {
    let Ok(store) = SqliteEventStore::new() else {
        return BTreeMap::new();
    };
    let Ok(envelopes) = store.query(&EventFilter {
        entity_id: Some(job_id.to_string()),
        event_type: Some("step.done_declared".to_string()),
        ..Default::default()
    }) else {
        return BTreeMap::new();
    };
    for env in envelopes {
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(&env.payload_json) else {
            continue;
        };
        if payload.get("step_id").and_then(|v| v.as_str()) != Some(step_id) {
            continue;
        }
        if let Some(outputs) = payload.get("outputs").and_then(|v| v.as_object()) {
            return outputs
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect();
        }
    }
    BTreeMap::new()
}

/// Read the `reason:` line from a `<lane>.failed` sentinel artifact.
/// The marker file is small plain text; we cap reads at 4 KiB defensively.
fn read_failure_reason(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let text = std::str::from_utf8(&bytes).ok()?;
    let trimmed = if text.len() > 4096 { &text[..4096] } else { text };
    for line in trimmed.lines() {
        if let Some(rest) = line.strip_prefix("reason:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Write an artifact blob to `<worktree>/.artifacts/input/<name>`.
/// Tries hardlink first (fast, same device); falls back to byte copy.
///
/// Defense-in-depth: even though `pipeline_loader::validate_output_name`
/// rejects traversal at load time, we re-verify here that `dst` resolves
/// within `input_dir`. A loader bypass (e.g. payload injected via an
/// unvalidated code path) cannot escape the worktree.
fn write_input_file(
    worktree_path: &Path,
    name: &str,
    record: &ArtifactRecord,
    ship_dir: &Path,
) -> std::io::Result<()> {
    let input_dir = worktree_path.join(".artifacts/input");
    std::fs::create_dir_all(&input_dir)?;
    let dst = input_dir.join(name);
    ensure_within(&input_dir, &dst)?;
    if dst.exists() {
        return Ok(());
    }
    let src = ship_dir.join(&record.path);
    if std::fs::hard_link(&src, &dst).is_err() {
        std::fs::copy(&src, &dst)?;
    }
    Ok(())
}

/// Verify that `candidate` resolves under `base`. We intentionally do NOT
/// call `canonicalize` on the candidate — the file may not exist yet, and
/// canonicalize follows symlinks which could itself be an attack vector.
/// Instead we canonicalize `base` (which must exist) and walk the candidate
/// component-by-component, rejecting `..` / absolute paths / root prefixes.
pub(crate) fn ensure_within(base: &Path, candidate: &Path) -> std::io::Result<()> {
    use std::path::Component;
    let base = base.canonicalize()?;
    // Strip the base prefix if present, otherwise walk relative components.
    let relative = candidate.strip_prefix(&base).unwrap_or(candidate);
    for comp in relative.components() {
        match comp {
            Component::Normal(_) => continue,
            Component::CurDir => continue,
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "path traversal rejected: '{}' escapes '{}'",
                        candidate.display(),
                        base.display()
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Load the pipeline for a job from the projection (for promote resolution).
fn load_pipeline_for_job(job_id: &str) -> Option<ship_substrate::events::job::Pipeline> {
    ship_substrate::projections::job::load_jobs()
        .ok()?
        .into_iter()
        .find(|(_, j)| j.job_id == job_id)
        .and_then(|(_, j)| j.steps)
}

/// Resolve the artifact ID for a promoted name that is consumed (not produced) by
/// the promoting step. Finds the upstream producer step and reads its outputs map.
fn find_artifact_for_promote(
    name: &str,
    job_id: &str,
    step: &PipelineStep,
    pipeline: Option<&ship_substrate::events::job::Pipeline>,
) -> Option<String> {
    // Only relevant when the name is in consumes (not in produces).
    if !step.consumes.contains(&name.to_string()) {
        return None;
    }
    let pipeline = pipeline?;
    let producer = pipeline.steps.iter().find(|s| s.produces.iter().any(|o| o.name == name))?;
    find_artifact_id_in_events(job_id, &producer.id, name)
}

/// Convert a pipeline `OutputScope` to an `ArtifactScope` for service calls.
/// User scope promotes are not supported by the orchestrator (no user_id
/// available in this context) and return an error.
fn output_scope_to_artifact_scope(
    scope: &OutputScope,
    project_id: &str,
) -> Result<ArtifactScope, String> {
    match scope {
        OutputScope::Project => Ok(ArtifactScope::Project {
            project_id: project_id.to_string(),
        }),
        OutputScope::User => Err(
            "User-scope promotes require an explicit user_id; not available in the orchestrator \
             context. Use Project scope instead."
                .to_string(),
        ),
        OutputScope::Run => Err(
            "Cannot promote to Run scope — run artifacts are the default source, not the target."
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::events::job::{OutputScope, StepOutput, StepPromote};

    #[test]
    fn check_required_outputs_passes_when_present() {
        let produces = vec![StepOutput {
            name: "patch".into(),
            scope: OutputScope::Run,
            required: true,
        }];
        let mut outputs = BTreeMap::new();
        outputs.insert("patch".into(), "art-01".into());
        assert!(check_required_outputs(&produces, &outputs).is_none());
    }

    #[test]
    fn check_required_outputs_fails_when_missing() {
        let produces = vec![StepOutput {
            name: "patch".into(),
            scope: OutputScope::Run,
            required: true,
        }];
        let outputs = BTreeMap::new();
        let err = check_required_outputs(&produces, &outputs);
        assert!(err.is_some());
        assert!(err.unwrap().contains("patch"));
    }

    #[test]
    fn check_required_outputs_optional_missing_is_ok() {
        let produces = vec![StepOutput {
            name: "extra".into(),
            scope: OutputScope::Run,
            required: false,
        }];
        let outputs = BTreeMap::new();
        assert!(check_required_outputs(&produces, &outputs).is_none());
    }

    #[test]
    fn output_scope_to_artifact_scope_project() {
        let scope = output_scope_to_artifact_scope(&OutputScope::Project, "my-proj").unwrap();
        assert!(matches!(scope, ArtifactScope::Project { .. }));
    }

    #[test]
    fn output_scope_to_artifact_scope_user_errors() {
        let err = output_scope_to_artifact_scope(&OutputScope::User, "my-proj");
        assert!(err.is_err());
    }

    #[test]
    fn output_scope_to_artifact_scope_run_errors() {
        let err = output_scope_to_artifact_scope(&OutputScope::Run, "my-proj");
        assert!(err.is_err());
    }

    #[test]
    fn ensure_within_rejects_parent_dir_traversal() {
        // Defense-in-depth: a loader bypass that planted `"../../../etc/evil"`
        // as a consumes name must not escape the input dir at write time.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let base = tmp.path().join("input");
        std::fs::create_dir_all(&base).unwrap();
        let malicious = base.join("../../../etc/evil");
        let err = ensure_within(&base, &malicious).expect_err("traversal must be rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        let msg = err.to_string();
        assert!(msg.contains("path traversal"), "message must flag traversal: {msg}");
    }

    #[test]
    fn ensure_within_accepts_plain_filename() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let base = tmp.path().join("input");
        std::fs::create_dir_all(&base).unwrap();
        let safe = base.join("patch.diff");
        ensure_within(&base, &safe).expect("plain filename must be allowed");
    }

    #[test]
    fn ensure_within_rejects_absolute_escape() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let base = tmp.path().join("input");
        std::fs::create_dir_all(&base).unwrap();
        // An attacker-supplied absolute path that doesn't share base's prefix.
        let escaped = std::path::PathBuf::from("/etc/passwd");
        let err = ensure_within(&base, &escaped).expect_err("absolute escape must be rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }
}
