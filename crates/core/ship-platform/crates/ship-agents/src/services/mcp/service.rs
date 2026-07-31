//! `McpService` — third method-oriented service-layer pilot per
//! modular-runtime-02. Same shape as Rules and Permissions:
//! trait + concrete impl + `Arc<dyn …>` hand-off + hermetic
//! `with_roots` test ctor. McpService is referenced cross-domain by
//! `PermissionsService::tool_patterns`; that composition can now
//! migrate to a trait-to-trait call instead of reaching into state.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use super::state;
use ship_files::files::{FilesService, Scope};

/// Body shape returned by `McpService::get`.
#[derive(Debug, Clone, Serialize)]
pub struct McpEntry {
    pub id: String,
    pub scope: state::ScopeTag,
    pub server: serde_json::Value,
}

/// Args for `McpService::create`.
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub id: String,
    pub server: serde_json::Value,
    /// `None` defaults to project scope.
    pub scope: Option<Scope>,
}

/// Args for `McpService::put`.
#[derive(Debug, Clone)]
pub struct PutArgs {
    pub id: String,
    pub server: serde_json::Value,
    /// `None` writes to whichever scope already holds the entry,
    /// defaulting to project when no existing entry is found.
    pub scope: Option<Scope>,
}

pub trait McpService: Send + Sync + 'static {
    fn list(&self) -> Vec<state::IndexEntry>;
    fn get(&self, id: &str, scope: Option<Scope>) -> Result<McpEntry, state::Error>;
    fn put(&self, args: PutArgs) -> Result<Scope, state::Error>;
    fn patch(
        &self,
        id: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error>;
    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error>;
    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error>;
}

pub struct McpServiceImpl {
    user_root: PathBuf,
    project_root_override: Option<PathBuf>,
}

impl McpServiceImpl {
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

    pub fn arc(ship_dir: PathBuf) -> Arc<dyn McpService> {
        Arc::new(Self::new(ship_dir))
    }

    fn files(&self) -> FilesService {
        match &self.project_root_override {
            Some(p) => FilesService::new(self.user_root.clone(), Some(p.clone())),
            None => FilesService::resolve_roots(&self.user_root),
        }
    }
}

impl McpService for McpServiceImpl {
    fn list(&self) -> Vec<state::IndexEntry> {
        state::list(&self.files())
    }

    fn get(&self, id: &str, scope: Option<Scope>) -> Result<McpEntry, state::Error> {
        let svc = self.files();
        let (resolved, server) = state::load_entry(&svc, id, scope)?;
        Ok(McpEntry {
            id: id.to_string(),
            scope: state::ScopeTag::from(resolved),
            server,
        })
    }

    fn put(&self, args: PutArgs) -> Result<Scope, state::Error> {
        let svc = self.files();
        let target = match args.scope {
            Some(s) => s,
            None => state::find(&svc, &args.id).unwrap_or(Scope::Project),
        };
        state::write_entry(&svc, &args.id, target, &args.server, true)?;
        Ok(target)
    }

    fn patch(
        &self,
        id: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error> {
        state::patch_entry(&self.files(), id, scope, patch)
    }

    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error> {
        state::delete_entry(&self.files(), id, scope)
    }

    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error> {
        let target = args.scope.unwrap_or(Scope::Project);
        state::write_entry(&self.files(), &args.id, target, &args.server, false)?;
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh() -> (TempDir, McpServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        let svc = McpServiceImpl::with_roots(user, Some(project));
        (tmp, svc)
    }

    fn server_value() -> serde_json::Value {
        serde_json::json!({
            "command": "ship",
            "args": ["run", "server"]
        })
    }

    #[test]
    fn empty_list_when_no_entries() {
        let (_tmp, svc) = fresh();
        assert!(svc.list().is_empty());
    }

    #[test]
    fn create_then_list_then_get_round_trips() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "ship".into(),
            server: server_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();

        assert_eq!(svc.list().len(), 1);
        let got = svc.get("ship", None).unwrap();
        assert_eq!(got.server["command"], "ship");
    }

    #[test]
    fn create_rejects_duplicate_id() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "x".into(),
            server: server_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let err = svc
            .create(CreateArgs {
                id: "x".into(),
                server: server_value(),
                scope: Some(Scope::Project),
            })
            .unwrap_err();
        assert!(matches!(err, state::Error::AlreadyExists(_)));
    }

    #[test]
    fn delete_returns_scope_and_removes() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "doomed".into(),
            server: server_value(),
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
