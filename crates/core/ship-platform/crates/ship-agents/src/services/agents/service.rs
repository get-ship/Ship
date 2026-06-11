//! `AgentsService` — fifth method-oriented service-layer migration per
//! modular-runtime-02. Same shape as the previous four (rules,
//! permissions, mcp, skills): trait + `*ServiceImpl` + `with_roots`
//! test ctor + `Arc<dyn …>` hand-off.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use ship_files::files::{FilesService, Scope};
use super::state;

/// Body shape returned by `AgentsService::get`.
#[derive(Debug, Clone, Serialize)]
pub struct AgentDoc {
    pub id: String,
    pub scope: state::ScopeTag,
    pub doc: serde_json::Value,
}

/// Args for `AgentsService::create`.
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub id: String,
    pub doc: serde_json::Value,
    pub scope: Option<Scope>,
}

/// Args for `AgentsService::put` — replace doc with optional scope override.
#[derive(Debug, Clone)]
pub struct PutArgs {
    pub id: String,
    pub doc: serde_json::Value,
    pub scope: Option<Scope>,
}

/// Args for `AgentsService::duplicate`. Source scope is whichever scope
/// already holds the source id (project shadows user when both exist);
/// `to_scope = None` keeps the duplicate in the same scope as the source.
#[derive(Debug, Clone)]
pub struct DuplicateArgs {
    pub source_id: String,
    pub source_scope: Option<Scope>,
    pub new_id: String,
    pub to_scope: Option<Scope>,
}

pub trait AgentsService: Send + Sync + 'static {
    fn list(&self) -> Vec<state::IndexEntry>;
    fn get(&self, id: &str, scope: Option<Scope>) -> Result<AgentDoc, state::Error>;
    fn put(&self, args: PutArgs) -> Result<Scope, state::Error>;
    fn patch(
        &self,
        id: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error>;
    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error>;
    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error>;
    fn rename(
        &self,
        id: &str,
        new_id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error>;
    fn duplicate(&self, args: DuplicateArgs) -> Result<Scope, state::Error>;
    fn move_scope(
        &self,
        id: &str,
        from: Scope,
        to: Scope,
    ) -> Result<(), state::Error>;
}

pub struct AgentsServiceImpl {
    user_root: PathBuf,
    project_root_override: Option<PathBuf>,
}

impl AgentsServiceImpl {
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

    pub fn arc(ship_dir: PathBuf) -> Arc<dyn AgentsService> {
        Arc::new(Self::new(ship_dir))
    }

    fn files(&self) -> FilesService {
        match &self.project_root_override {
            Some(p) => FilesService::new(self.user_root.clone(), Some(p.clone())),
            None => FilesService::resolve_roots(&self.user_root),
        }
    }
}

/// Validate that `doc` is schema-conformant: it must parse as
/// `compiler::AgentProfile` (which requires the canonical `agent:` block), and
/// any declared capabilities must be agent-settable (no C3/critical). Returns a
/// `Parse` error with a human-readable message on failure — the write is refused.
///
/// Called in `create` and `put` (not `patch` or `duplicate` — patch operates on
/// an already-valid doc; duplicate copies a previously validated source).
fn validate_agent_structure(doc: &serde_json::Value) -> Result<(), state::Error> {
    let profile: compiler::AgentProfile = serde_json::from_value(doc.clone())
        .map_err(|e| state::Error::Parse(format!("invalid agent schema: {e}")))?;
    agent_def::Agent::from_profile(&profile, "")
        .validate_capabilities()
        .map_err(|e| state::Error::Parse(format!("invalid capability: {e}")))?;
    Ok(())
}

/// Force the doc's `agent.id` field to match `id` so the slot can't drift
/// from the filename. Mirrors the helper in mod.rs that the HTTP handler
/// previously called inline.
fn force_agent_id(mut doc: serde_json::Value, id: &str) -> serde_json::Value {
    if let Some(obj) = doc.as_object_mut() {
        let agent_entry = obj
            .entry("agent".to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let Some(agent) = agent_entry.as_object_mut() {
            agent.insert("id".into(), serde_json::Value::String(id.into()));
            if !agent.contains_key("name") {
                agent.insert("name".into(), serde_json::Value::String(id.into()));
            }
        }
    }
    doc
}

impl AgentsService for AgentsServiceImpl {
    fn list(&self) -> Vec<state::IndexEntry> {
        state::list(&self.files())
    }

    fn get(&self, id: &str, scope: Option<Scope>) -> Result<AgentDoc, state::Error> {
        let svc = self.files();
        let (resolved, doc) = state::load_doc(&svc, id, scope)?;
        Ok(AgentDoc {
            id: id.to_string(),
            scope: state::ScopeTag::from(resolved),
            doc,
        })
    }

    fn put(&self, args: PutArgs) -> Result<Scope, state::Error> {
        let svc = self.files();
        let target = match args.scope {
            Some(s) => s,
            None => match state::find(&svc, &args.id) {
                Some((s, _)) => s,
                None => Scope::Project,
            },
        };
        validate_agent_structure(&args.doc)?;
        state::write_doc(&svc, &args.id, target, &args.doc, true)?;
        Ok(target)
    }

    fn patch(
        &self,
        id: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error> {
        state::patch_doc(&self.files(), id, scope, patch)
    }

    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error> {
        state::delete_doc(&self.files(), id, scope)
    }

    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error> {
        let target = args.scope.unwrap_or(Scope::Project);
        let doc = force_agent_id(args.doc, &args.id);
        validate_agent_structure(&doc)?;
        state::write_doc(&self.files(), &args.id, target, &doc, false)?;
        Ok(target)
    }

    fn rename(
        &self,
        id: &str,
        new_id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error> {
        state::rename(&self.files(), id, new_id, scope)
    }

    fn duplicate(&self, args: DuplicateArgs) -> Result<Scope, state::Error> {
        let svc = self.files();
        let (from_scope, doc) = state::load_doc(&svc, &args.source_id, args.source_scope)?;
        let to = args.to_scope.unwrap_or(from_scope);
        let doc = force_agent_id(doc, &args.new_id);
        state::write_doc(&svc, &args.new_id, to, &doc, false)?;
        Ok(to)
    }

    fn move_scope(
        &self,
        id: &str,
        from: Scope,
        to: Scope,
    ) -> Result<(), state::Error> {
        state::move_scope(&self.files(), id, from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh() -> (TempDir, AgentsServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(user.join("agents")).unwrap();
        std::fs::create_dir_all(project.join("agents")).unwrap();
        let svc = AgentsServiceImpl::with_roots(user, Some(project));
        (tmp, svc)
    }

    fn doc_value() -> serde_json::Value {
        serde_json::json!({
            "agent": { "id": "ignored-by-create", "name": "ignored-by-create" },
            "permission": "ship-autonomous"
        })
    }

    #[test]
    fn empty_list_when_no_agents() {
        let (_tmp, svc) = fresh();
        assert!(svc.list().is_empty());
    }

    #[test]
    fn create_forces_id_to_match_filename() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "rust-lane".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let got = svc.get("rust-lane", None).unwrap();
        assert_eq!(got.doc["agent"]["id"], "rust-lane");
        assert_eq!(got.doc["agent"]["name"], "ignored-by-create");
    }

    #[test]
    fn duplicate_keeps_source_scope_when_to_scope_unset() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "src".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let scope = svc
            .duplicate(DuplicateArgs {
                source_id: "src".into(),
                source_scope: None,
                new_id: "copy".into(),
                to_scope: None,
            })
            .unwrap();
        assert_eq!(scope, Scope::Project);
        let got = svc.get("copy", None).unwrap();
        assert_eq!(got.doc["agent"]["id"], "copy");
    }

    #[test]
    fn rename_changes_id() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "old".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        svc.rename("old", "new", Some(Scope::Project)).unwrap();
        assert!(svc.get("old", None).is_err());
        assert!(svc.get("new", None).is_ok());
    }

    #[test]
    fn delete_returns_scope() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "doomed".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        assert_eq!(svc.delete("doomed", None).unwrap(), Scope::Project);
    }

    // ── Gap B: validation gate ────────────────────────────────────────────────

    #[test]
    fn create_rejects_c3_capability() {
        let (_tmp, svc) = fresh();
        let bad_cap = serde_json::json!({
            "agent": { "id": "bad", "name": "Bad" },
            "capabilities": [{ "domain": "account", "blast": "C3" }]
        });
        let err = svc
            .create(CreateArgs {
                id: "bad".into(),
                doc: bad_cap,
                scope: Some(Scope::Project),
            })
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("C3") || msg.contains("capability") || msg.contains("invalid"),
            "expected capability error, got: {msg}"
        );
    }

    #[test]
    fn put_rejects_flat_doc_without_agent_block() {
        let (_tmp, svc) = fresh();
        // First create a valid doc.
        svc.create(CreateArgs {
            id: "alpha".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        // Now PUT a flat replacement — must be rejected.
        let flat = serde_json::json!({ "id": "alpha", "name": "Flat" });
        let err = svc
            .put(PutArgs { id: "alpha".into(), doc: flat, scope: None })
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("parse") || msg.contains("invalid"),
            "expected parse error on PUT, got: {msg}"
        );
    }
}
