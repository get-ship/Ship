//! `PermissionsService` — second method-oriented service-layer pilot
//! per modular-runtime-02. Mirrors the rules domain's shape — same
//! `with_roots` test ctor, same `Arc<dyn …>` hand-off — so consumers
//! can swap implementations without standing up an HTTP server.
//!
//! Permission preset CRUD plus the `tool-patterns` autocomplete
//! catalog. The tool-patterns method composes the permissions list
//! with the project / user MCP server list — cross-domain reads
//! happen inside the impl so transports stay marshalling-only.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use super::super::files::{FilesService, Scope};
use super::super::mcp::service::McpService;
use super::state;

/// Body shape returned by `PermissionsService::get`.
#[derive(Debug, Clone, Serialize)]
pub struct PermissionEntry {
    pub id: String,
    pub scope: state::ScopeTag,
    pub preset: serde_json::Value,
}

/// Args for `PermissionsService::create`.
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub id: String,
    pub preset: serde_json::Value,
    /// `None` defaults to project scope.
    pub scope: Option<Scope>,
}

/// Args for `PermissionsService::put`.
#[derive(Debug, Clone)]
pub struct PutArgs {
    pub id: String,
    pub preset: serde_json::Value,
    /// `None` writes to whichever scope already holds the preset, defaulting
    /// to project when no existing entry is found.
    pub scope: Option<Scope>,
}

/// One entry in the autocomplete catalog returned by
/// `PermissionsService::tool_patterns`.
#[derive(Debug, Clone, Serialize)]
pub struct ToolPattern {
    pub pattern: String,
    pub label: String,
    /// `"provider"` | `"bash"` | `"mcp-<server>"`.
    pub source: String,
}

const PROVIDER_BUILTINS: &[&str] = &[
    "Read", "Write", "Edit", "Glob", "Grep", "Bash",
    "NotebookEdit", "WebFetch", "WebSearch", "Task",
];

const BASH_SHAPES: &[(&str, &str)] = &[
    ("Bash(git *)", "Bash git — all git subcommands"),
    ("Bash(git reset*)", "Bash git — reset (destructive)"),
    ("Bash(git push --force*)", "Bash git — force push (destructive)"),
    ("Bash(npm *)", "Bash npm — all npm subcommands"),
    ("Bash(pnpm *)", "Bash pnpm"),
    ("Bash(cargo *)", "Bash cargo"),
    ("Bash(ship *)", "Bash ship — Ship CLI"),
    ("Bash(just *)", "Bash just"),
    ("Bash(rm -rf *)", "Bash rm — recursive delete (destructive)"),
];

pub trait PermissionsService: Send + Sync + 'static {
    fn list(&self) -> Vec<state::IndexEntry>;
    fn get(
        &self,
        id: &str,
        scope: Option<Scope>,
    ) -> Result<PermissionEntry, state::Error>;
    fn put(&self, args: PutArgs) -> Result<Scope, state::Error>;
    fn patch(
        &self,
        id: &str,
        scope: Option<Scope>,
        patch: &json_patch::Patch,
    ) -> Result<Scope, state::Error>;
    fn delete(
        &self,
        id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error>;
    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error>;

    /// Autocomplete catalog: provider builtins + curated Bash shapes +
    /// wildcard patterns for every MCP server known to the daemon.
    /// Cross-domain reads (the MCP listing) live inside the impl so the
    /// HTTP / MCP / view-registry transports stay marshalling-only.
    fn tool_patterns(&self) -> Vec<ToolPattern>;
}

pub struct PermissionsServiceImpl {
    user_root: PathBuf,
    project_root_override: Option<PathBuf>,
    /// Sibling MCP service — `tool_patterns` reads from it through the
    /// trait so cross-domain composition stays in trait-land instead
    /// of reaching into the mcp `state::*` free functions.
    mcp: Arc<dyn McpService>,
}

impl PermissionsServiceImpl {
    /// Production constructor. Wires `mcp` from the same ship_dir so
    /// project / user composition works out of the box.
    pub fn new(ship_dir: PathBuf) -> Self {
        let mcp = super::super::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        Self {
            user_root: ship_dir,
            project_root_override: None,
            mcp,
        }
    }

    /// Test constructor — pins both roots and accepts an injected
    /// `Arc<dyn McpService>` so `tool_patterns` can be exercised
    /// against a hermetic catalog.
    pub fn with_roots(
        user_root: PathBuf,
        project_root: Option<PathBuf>,
        mcp: Arc<dyn McpService>,
    ) -> Self {
        Self {
            user_root,
            project_root_override: project_root,
            mcp,
        }
    }

    pub fn arc(ship_dir: PathBuf) -> Arc<dyn PermissionsService> {
        Arc::new(Self::new(ship_dir))
    }

    fn files(&self) -> FilesService {
        match &self.project_root_override {
            Some(p) => FilesService::new(self.user_root.clone(), Some(p.clone())),
            None => FilesService::resolve_roots(&self.user_root),
        }
    }
}

impl PermissionsService for PermissionsServiceImpl {
    fn list(&self) -> Vec<state::IndexEntry> {
        state::list(&self.files())
    }

    fn get(
        &self,
        id: &str,
        scope: Option<Scope>,
    ) -> Result<PermissionEntry, state::Error> {
        let svc = self.files();
        let (resolved, preset) = state::load_entry(&svc, id, scope)?;
        Ok(PermissionEntry {
            id: id.to_string(),
            scope: state::ScopeTag::from(resolved),
            preset,
        })
    }

    fn put(&self, args: PutArgs) -> Result<Scope, state::Error> {
        let svc = self.files();
        let target = match args.scope {
            Some(s) => s,
            None => state::find(&svc, &args.id).unwrap_or(Scope::Project),
        };
        state::write_entry(&svc, &args.id, target, &args.preset, true)?;
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

    fn delete(
        &self,
        id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error> {
        state::delete_entry(&self.files(), id, scope)
    }

    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error> {
        let target = args.scope.unwrap_or(Scope::Project);
        state::write_entry(&self.files(), &args.id, target, &args.preset, false)?;
        Ok(target)
    }

    fn tool_patterns(&self) -> Vec<ToolPattern> {
        let mut out: Vec<ToolPattern> = Vec::with_capacity(
            PROVIDER_BUILTINS.len() + BASH_SHAPES.len(),
        );
        for t in PROVIDER_BUILTINS {
            out.push(ToolPattern {
                pattern: (*t).into(),
                label: (*t).into(),
                source: "provider".into(),
            });
        }
        for (p, l) in BASH_SHAPES {
            out.push(ToolPattern {
                pattern: (*p).into(),
                label: (*l).into(),
                source: "bash".into(),
            });
        }
        // Cross-domain composition through the `McpService` trait —
        // not the mcp `state::*` free functions. The cloud worker
        // and browser embed will swap in different `McpService`
        // impls and tool_patterns picks up their catalog without
        // changing this code.
        for entry in self.mcp.list() {
            let pattern = format!("mcp__{}__*", entry.id);
            out.push(ToolPattern {
                pattern,
                label: format!("mcp · {} · all tools", entry.id),
                source: format!("mcp-{}", entry.id),
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh() -> (TempDir, PermissionsServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        // Build an isolated McpService against the same hermetic
        // roots so tool_patterns picks up no host MCP servers.
        let mcp = std::sync::Arc::new(
            super::super::super::mcp::service::McpServiceImpl::with_roots(
                user.clone(),
                Some(project.clone()),
            ),
        );
        let svc = PermissionsServiceImpl::with_roots(user, Some(project), mcp);
        (tmp, svc)
    }

    fn preset_value() -> serde_json::Value {
        serde_json::json!({
            "default_mode": "ship-autonomous",
            "rules": []
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
            id: "default".into(),
            preset: preset_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();

        let listed = svc.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "default");

        let entry = svc.get("default", None).unwrap();
        assert_eq!(entry.preset["default_mode"], "ship-autonomous");
    }

    #[test]
    fn create_rejects_duplicate_id() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "p".into(),
            preset: preset_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let err = svc
            .create(CreateArgs {
                id: "p".into(),
                preset: preset_value(),
                scope: Some(Scope::Project),
            })
            .unwrap_err();
        assert!(matches!(err, state::Error::AlreadyExists(_)));
    }

    #[test]
    fn put_overwrites_existing() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "p".into(),
            preset: preset_value(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let new_preset = serde_json::json!({ "default_mode": "ship-standard" });
        svc.put(PutArgs {
            id: "p".into(),
            preset: new_preset.clone(),
            scope: None,
        })
        .unwrap();
        let got = svc.get("p", None).unwrap();
        assert_eq!(got.preset["default_mode"], "ship-standard");
    }

    #[test]
    fn delete_returns_scope_and_removes() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "doomed".into(),
            preset: preset_value(),
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

    #[test]
    fn tool_patterns_includes_builtins_and_bash() {
        let (_tmp, svc) = fresh();
        let patterns = svc.tool_patterns();
        assert!(patterns.iter().any(|p| p.pattern == "Read" && p.source == "provider"));
        assert!(patterns.iter().any(|p| p.pattern == "Bash(git *)" && p.source == "bash"));
    }
}
