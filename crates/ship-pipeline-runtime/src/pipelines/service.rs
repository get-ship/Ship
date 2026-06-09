//! `PipelinesService` — sixth mr-02 migration.
//!
//! Covers the pipeline-definition CRUD only. Dispatch / cancel / run
//! detail still go through `runtime_api` because they're event-emitting
//! flows that need the kernel handle and the auth context, both of
//! which are HTTP-handler-shaped concerns; folding them into the trait
//! at v1 would force the trait to know about axum extractors.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use ship_files::files::{FilesService, Scope};
use super::state;

/// Body shape returned by `PipelinesService::get`. `format` is the
/// on-disk file extension (`yaml` / `jsonc`) so the round-trip path is
/// stable when callers re-write through `put`.
#[derive(Debug, Clone, Serialize)]
pub struct PipelineDoc {
    pub name: String,
    pub scope: state::ScopeTag,
    pub format: String,
    pub doc: serde_json::Value,
}

/// Args for `PipelinesService::put`.
#[derive(Debug, Clone)]
pub struct PutArgs {
    pub name: String,
    pub doc: serde_json::Value,
    pub scope: Option<Scope>,
}

/// Write-time validation for a pipeline document.
///
/// LENIENT-READ stays lenient: this only runs on `put`. A doc that declares
/// `steps` must be a well-formed [`PipelineDefinition`] and pass every platform
/// invariant (agent/produces/consumes/fanout). The legacy `command` wire key,
/// the positional `step_specs` array, and a flat `provider` are all tolerated
/// by the typed deserialize. The legacy short-form `agents:` authoring sugar is
/// normalized through the runtime parser before its DAG is validated. Anything
/// that declares neither `steps` nor `agents` is rejected as malformed.
fn validate_put_doc(doc: &serde_json::Value) -> Result<(), state::Error> {
    use ship_pipelines::PipelineDefinition;

    if doc.get("steps").is_some() {
        let def: PipelineDefinition = serde_json::from_value(doc.clone())
            .map_err(|e| state::Error::Invalid(e.to_string()))?;
        return def.validate().map_err(state::Error::Invalid);
    }

    if doc.get("agents").is_some() {
        // Short-form sugar: normalize to the `{ steps: [...] }` shape, then
        // validate the resulting DAG via the same invariant checks.
        let text = doc.to_string();
        let (normalized, _meta) =
            ship_pipelines::normalize_pipeline(&text).map_err(state::Error::Invalid)?;
        let pipeline: ship_pipelines::Pipeline = serde_json::from_value(normalized)
            .map_err(|e| state::Error::Invalid(e.to_string()))?;
        return ship_pipelines::validate_pipeline(&pipeline).map_err(state::Error::Invalid);
    }

    Err(state::Error::Invalid(
        "pipeline must declare `steps` or `agents`".to_string(),
    ))
}

pub trait PipelinesService: Send + Sync + 'static {
    fn list(&self) -> Vec<state::IndexEntry>;
    fn get(
        &self,
        name: &str,
        scope: Option<Scope>,
    ) -> Result<PipelineDoc, state::Error>;
    fn put(&self, args: PutArgs) -> Result<Scope, state::Error>;
    fn patch(
        &self,
        name: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error>;
    fn delete(&self, name: &str, scope: Option<Scope>) -> Result<Scope, state::Error>;
}

pub struct PipelinesServiceImpl {
    user_root: PathBuf,
    project_root_override: Option<PathBuf>,
}

impl PipelinesServiceImpl {
    pub fn new(ship_dir: PathBuf) -> Self {
        Self {
            user_root: ship_dir,
            project_root_override: None,
        }
    }

    pub fn with_roots(user_root: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self {
            user_root,
            project_root_override: project_root,
        }
    }

    pub fn arc(ship_dir: PathBuf) -> Arc<dyn PipelinesService> {
        Arc::new(Self::new(ship_dir))
    }

    fn files(&self) -> FilesService {
        match &self.project_root_override {
            Some(p) => FilesService::new(self.user_root.clone(), Some(p.clone())),
            None => FilesService::resolve_roots(&self.user_root),
        }
    }
}

impl PipelinesService for PipelinesServiceImpl {
    fn list(&self) -> Vec<state::IndexEntry> {
        state::list(&self.files())
    }

    fn get(
        &self,
        name: &str,
        scope: Option<Scope>,
    ) -> Result<PipelineDoc, state::Error> {
        let svc = self.files();
        let (resolved, format, doc) = state::load_doc(&svc, name, scope)?;
        Ok(PipelineDoc {
            name: name.to_string(),
            scope: state::ScopeTag::from(resolved),
            format,
            doc,
        })
    }

    fn put(&self, args: PutArgs) -> Result<Scope, state::Error> {
        // STRICT-WRITE: validate before persisting. Lenient-read elsewhere
        // (`load_doc` accepts any JSONC) keeps on-disk docs and the legacy
        // `command` fixture parsing; `put` is the one place we reject malformed
        // input rather than silently writing it.
        validate_put_doc(&args.doc)?;
        let svc = self.files();
        let target = match args.scope {
            Some(s) => s,
            None => match state::find(&svc, &args.name) {
                Some((s, _)) => s,
                None => Scope::Project,
            },
        };
        state::write_doc(&svc, &args.name, target, &args.doc, true)?;
        Ok(target)
    }

    fn patch(
        &self,
        name: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error> {
        state::patch_doc(&self.files(), name, scope, patch)
    }

    fn delete(&self, name: &str, scope: Option<Scope>) -> Result<Scope, state::Error> {
        state::delete_doc(&self.files(), name, scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh() -> (TempDir, PipelinesServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(user.join("pipelines")).unwrap();
        std::fs::create_dir_all(project.join("pipelines")).unwrap();
        let svc = PipelinesServiceImpl::with_roots(user, Some(project));
        (tmp, svc)
    }

    fn doc_value() -> serde_json::Value {
        serde_json::json!({
            "steps": [
                { "id": "build", "type": "run", "command": "cargo build" }
            ]
        })
    }

    #[test]
    fn empty_list_when_no_pipelines() {
        let (_tmp, svc) = fresh();
        assert!(svc.list().is_empty());
    }

    #[test]
    fn put_then_get_round_trips() {
        let (_tmp, svc) = fresh();
        svc.put(PutArgs {
            name: "build-only".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();

        assert_eq!(svc.list().len(), 1);
        let got = svc.get("build-only", None).unwrap();
        assert_eq!(got.doc["steps"][0]["id"], "build");
    }

    #[test]
    fn put_accepts_valid_definition() {
        let (_tmp, svc) = fresh();
        let valid = serde_json::json!({
            "name": "ok",
            "steps": [
                { "id": "impl", "agent": "rust-lane", "provider": "claude" },
                { "id": "verify", "run": "cargo check", "after": ["impl"] }
            ]
        });
        svc.put(PutArgs { name: "ok".into(), doc: valid, scope: Some(Scope::Project) })
            .expect("valid definition must be accepted");
        assert_eq!(svc.list().len(), 1);
    }

    #[test]
    fn put_rejects_agent_step_without_agent() {
        let (_tmp, svc) = fresh();
        let invalid = serde_json::json!({
            "steps": [ { "id": "impl", "type": "agent" } ]
        });
        let err = svc
            .put(PutArgs { name: "bad".into(), doc: invalid, scope: Some(Scope::Project) })
            .expect_err("agent step without agent must be rejected");
        assert!(matches!(err, state::Error::Invalid(_)), "got {err:?}");
        assert!(err.to_string().contains("agent"), "error: {err}");
        // Nothing was written.
        assert!(svc.list().is_empty());
    }

    #[test]
    fn put_rejects_doc_without_steps_or_agents() {
        let (_tmp, svc) = fresh();
        let invalid = serde_json::json!({ "name": "empty" });
        let err = svc
            .put(PutArgs { name: "empty".into(), doc: invalid, scope: Some(Scope::Project) })
            .expect_err("doc with neither steps nor agents must be rejected");
        assert!(matches!(err, state::Error::Invalid(_)), "got {err:?}");
    }

    #[test]
    fn put_accepts_legacy_command_fixture() {
        // LENIENT-READ: the legacy `command` wire key still parses on write.
        let (_tmp, svc) = fresh();
        svc.put(PutArgs {
            name: "legacy".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .expect("legacy command fixture must still be accepted");
    }

    #[test]
    fn delete_returns_scope_and_removes() {
        let (_tmp, svc) = fresh();
        svc.put(PutArgs {
            name: "doomed".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let scope = svc.delete("doomed", None).unwrap();
        assert_eq!(scope, Scope::Project);
        assert!(matches!(
            svc.get("doomed", None).unwrap_err(),
            state::Error::NotFound(_)
        ));
    }
}
