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
    /// Explicit space root for the composed roll-up read (`state::list` /
    /// `state::load_doc` — the C0 Get/List surface). `None` = walk up from
    /// cwd via `view_state::resolve_space_root` (helmd's production path);
    /// `Some(root)` = tests injecting a tempdir as a composed space.
    space_root_override: Option<PathBuf>,
}

impl AgentsServiceImpl {
    pub fn new(ship_dir: PathBuf) -> Self {
        Self::with_roots_and_space(ship_dir, None, None)
    }

    pub fn with_roots(user_root: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self::with_roots_and_space(user_root, project_root, None)
    }

    /// Full-fidelity test/injection ctor — pins the jsonc door roots AND
    /// the composed-read space root. Production callers (helmd, shipd)
    /// use [`new`] / [`with_roots`] (composed-read = cwd-walked).
    pub fn with_roots_and_space(
        user_root: PathBuf,
        project_root: Option<PathBuf>,
        space_root: Option<PathBuf>,
    ) -> Self {
        Self {
            user_root,
            project_root_override: project_root,
            space_root_override: space_root,
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

    fn space_root(&self) -> Option<&std::path::Path> {
        self.space_root_override.as_deref()
    }
}

/// Validate that `doc` is schema-conformant: it must parse as
/// `stack_model::AgentProfile` (which requires the canonical `agent:` block), and
/// any declared capabilities must be agent-settable (no C3/critical). Returns a
/// `Parse` error with a human-readable message on failure — the write is refused.
///
/// Called in `create` and `put` (not `patch` or `duplicate` — patch operates on
/// an already-valid doc; duplicate copies a previously validated source).
fn validate_agent_structure(doc: &serde_json::Value) -> Result<(), state::Error> {
    let profile: stack_model::AgentProfile = serde_json::from_value(doc.clone())
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
        state::list(self.space_root())
    }

    fn get(&self, id: &str, scope: Option<Scope>) -> Result<AgentDoc, state::Error> {
        let (resolved, doc) = state::load_doc(self.space_root(), id, scope)?;
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
        // Phase-6-fenced: the source read stays on the jsonc door — the
        // roll-up does not reflect writes, so cloning a TS-declared agent
        // the operator just mutated would fail. Reads that serve writes
        // ride `load_doc_jsonc`, not the composed `state::load_doc`.
        let (from_scope, doc) =
            state::load_doc_jsonc(&svc, &args.source_id, args.source_scope)?;
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

    /// Build a tempdir-shaped space with a stack.config.ts the composed
    /// roll-up will pick up. Returns the dir + a `with_roots_and_space`
    /// service that pins BOTH the jsonc door roots AND the composed
    /// space root to the same tempdir. Used by the C0 Get/List
    /// regression tests below — see `composed_get_and_list_resolve_ts_declared_agent`.
    fn fresh_ts_space(ts_body: &str) -> (TempDir, AgentsServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        std::fs::write(tmp.path().join("stack.config.ts"), ts_body).unwrap();
        let svc = AgentsServiceImpl::with_roots_and_space(
            user,
            Some(project),
            Some(tmp.path().to_path_buf()),
        );
        (tmp, svc)
    }

    fn doc_value() -> serde_json::Value {
        serde_json::json!({
            "agent": { "id": "ignored-by-create", "name": "ignored-by-create" },
            "permission": "ship-autonomous"
        })
    }

    /// C0 Get/List regression (job-spec-helmd-reads-composed.md
    /// acceptance): a `with_roots` (no-index, no `AgentDefIndex`)
    /// service must serve `get` AND `list` for an agent declared only
    /// via `defineAgent` in `stack.config.ts` — there is no
    /// `.ship/agents/` directory anywhere on disk. Mirrors the W5b
    /// regression in `view_state_tests` (W5b covered `summaries` /
    /// `detail` / `agent_id_exists`; this commit extends coverage to
    /// `list` / `get` — the last two jsonc-era reads the helmd posture
    /// mounted). The composed roll-up is the only source the wire
    /// touches; pre-migration these reads went through
    /// `state::load_doc`/`state::list` and hit the `.ship/agents/*.jsonc`
    /// directory scan, returning `not_found` for TS-declared agents.
    #[test]
    fn composed_get_and_list_resolve_ts_declared_agent() {
        let (_tmp, svc) = fresh_ts_space(
            r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "rust-lane", name: "Rust Lane" },
  skills: { refs: ["ship-agents"] },
  mcp: { servers: ["ship"] },
  rules: { refs: ["discipline"] },
  permissions: { preset: "ship-standard" },
});
"#,
        );
        assert!(
            !_tmp.path().join(".ship").join("agents").exists(),
            "fixture must have no .ship/agents/ dir — the whole point of the regression"
        );

        // List returns the TS-declared agent.
        let items = svc.list();
        assert_eq!(items.len(), 1, "expected one composed agent, got: {items:#?}");
        assert_eq!(items[0].id, "rust-lane");

        // Get returns the same agent with its doc and resolved scope.
        let got = svc.get("rust-lane", None).expect("get rust-lane");
        assert_eq!(got.id, "rust-lane");
        assert_eq!(got.doc["agent"]["id"], "rust-lane");
        assert_eq!(got.doc["agent"]["name"], "Rust Lane");
        let skills = got.doc["skills"]["refs"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        assert_eq!(skills, vec!["ship-agents"]);

        // Get with a forced scope that does NOT match returns the
        // canonical "agent `id`" NotFound — the bare-`id` symptom that
        // motivated this migration is gone.
        let err = svc.get("rust-lane", Some(Scope::User)).unwrap_err();
        assert!(
            err.to_string().contains("agent `rust-lane`"),
            r#"expected `agent `rust-lane`` NotFound, got: {err}"#
        );
    }

    #[test]
    fn empty_list_when_no_agents() {
        // Empty composed space: the tempdir IS a space (`.ship/` +
        // `stack.config.ts`) so the cwd-walked `resolve_space_root` would
        // otherwise find THIS test's enclosing repo and pick up the
        // actual repo's agents. With the override pinned to the empty
        // tempdir, the composed roll-up has no agents.
        let (_tmp, svc) = fresh_ts_space(
            r#"
import { defineAgent } from "ship";
// No defineAgent calls — empty space.
"#,
        );
        assert!(svc.list().is_empty());
    }

    #[test]
    fn create_forces_id_to_match_filename() {
        // The create-side contract: `create` overwrites the doc's
        // `agent.id` with the file id and writes to the jsonc door.
        // Reads are composed (post-migration), so the post-create
        // assertion uses `load_doc_jsonc` to verify the file on disk
        // (the jsonc write door's content) — the same door `create`
        // wrote to.
        let (tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "rust-lane".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let files = FilesService::resolve_roots(&tmp.path().join("project"));
        let (_scope, doc) = state::load_doc_jsonc(&files, "rust-lane", None).unwrap();
        assert_eq!(doc["agent"]["id"], "rust-lane");
        assert_eq!(doc["agent"]["name"], "ignored-by-create");
    }

    #[test]
    fn duplicate_keeps_source_scope_when_to_scope_unset() {
        let (tmp, svc) = fresh();
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
        // The duplicate wrote a second file on disk — read it back via
        // the jsonc door (composed reads do not reflect writes until
        // Phase-6; the duplicate's read in `service::duplicate` stays
        // on the jsonc door for the same reason).
        let files = FilesService::resolve_roots(&tmp.path().join("project"));
        let (_scope, doc) = state::load_doc_jsonc(&files, "copy", None).unwrap();
        assert_eq!(doc["agent"]["id"], "copy");
    }

    #[test]
    fn rename_changes_id() {
        let (tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "old".into(),
            doc: doc_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        svc.rename("old", "new", Some(Scope::Project)).unwrap();
        // Read back via the jsonc door (writes are jsonc) — the old id
        // is gone (renamed/trashed), the new id holds the content.
        let files = FilesService::resolve_roots(&tmp.path().join("project"));
        assert!(state::load_doc_jsonc(&files, "old", None).is_err());
        let (_, doc) = state::load_doc_jsonc(&files, "new", None).unwrap();
        assert_eq!(doc["agent"]["id"], "new");
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
