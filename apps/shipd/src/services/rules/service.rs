//! `RulesService` — method-oriented domain trait per modular-runtime-02.
//!
//! Pilot for the broader spec-02 pattern: business logic moves behind a
//! `trait *Service`; transports (HTTP today, MCP / view registry / cloud
//! worker tomorrow) become thin marshallers that call the same trait.
//! No transport (`axum`, `mcp::*`) imports here — the trait is testable
//! without standing up an HTTP server.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use ship_platform::services::files::{FilesService, Scope};
use super::state;

/// Body shape returned by `RulesService::get`.
#[derive(Debug, Clone, Serialize)]
pub struct RuleContent {
    pub id: String,
    pub scope: state::ScopeTag,
    pub content: String,
}

/// Args for `RulesService::create` — full body create with explicit scope.
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub id: String,
    pub content: String,
    /// `None` defaults to project scope.
    pub scope: Option<Scope>,
}

/// Args for `RulesService::put` — replace body, with optional scope override.
#[derive(Debug, Clone)]
pub struct PutArgs {
    pub id: String,
    pub content: String,
    /// `None` writes to whichever scope already holds the rule, defaulting
    /// to project when no existing rule is found.
    pub scope: Option<Scope>,
}

/// Method-oriented domain trait. `Send + Sync + 'static` so transports can
/// share it via `Arc<dyn RulesService>`. Methods are sync because the
/// underlying file ops are sync; async will be introduced when a method
/// genuinely needs it (e.g. cross-process locks).
pub trait RulesService: Send + Sync + 'static {
    /// List all rules across project + user scope. Project shadows user
    /// on id collision; entries are sorted by id.
    fn list(&self) -> Vec<state::IndexEntry>;

    /// Read a rule's body. `scope = None` walks Project → User to find it.
    fn get(&self, id: &str, scope: Option<Scope>) -> Result<RuleContent, state::Error>;

    /// Replace a rule's body. Caller must use `create` for net-new rules.
    fn put(&self, args: PutArgs) -> Result<Scope, state::Error>;

    /// Trash a rule. `scope = None` walks Project → User.
    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error>;

    /// Create a fresh rule. Errors with `AlreadyExists` if the id is taken
    /// in the target scope.
    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error>;
}

/// Concrete impl backed by the on-disk `<scope>/rules/*.md` tree.
///
/// Stores a user-scope root and an optional project-scope override.
/// Production callers go through `new(ship_dir)` which lets
/// `FilesService::resolve_roots` discover the project from cwd at call
/// time; tests use `with_roots(user, project)` to pin both and stay
/// hermetic against the test process's real `~/.ship/`.
pub struct RulesServiceImpl {
    user_root: PathBuf,
    project_root_override: Option<PathBuf>,
}

impl RulesServiceImpl {
    /// Production constructor — `FilesService::resolve_roots` semantics.
    pub fn new(ship_dir: PathBuf) -> Self {
        Self {
            user_root: ship_dir,
            project_root_override: None,
        }
    }

    /// Test constructor — pins both roots so the service ignores the
    /// host environment.
    pub fn with_roots(user_root: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self {
            user_root,
            project_root_override: project_root,
        }
    }

    /// Cheap wrapper for callers that want an `Arc<dyn RulesService>`
    /// without naming the concrete type. Production semantics.
    pub fn arc(ship_dir: PathBuf) -> Arc<dyn RulesService> {
        Arc::new(Self::new(ship_dir))
    }

    fn files(&self) -> FilesService {
        match &self.project_root_override {
            Some(p) => FilesService::new(self.user_root.clone(), Some(p.clone())),
            None => FilesService::resolve_roots(&self.user_root),
        }
    }
}

impl RulesService for RulesServiceImpl {
    fn list(&self) -> Vec<state::IndexEntry> {
        state::list(&self.files())
    }

    fn get(&self, id: &str, scope: Option<Scope>) -> Result<RuleContent, state::Error> {
        let svc = self.files();
        let (resolved, body) = state::load_body(&svc, id, scope)?;
        Ok(RuleContent {
            id: id.to_string(),
            scope: state::ScopeTag::from(resolved),
            content: body,
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
        state::write_body(&svc, &args.id, target, &args.content, true)?;
        Ok(target)
    }

    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error> {
        let svc = self.files();
        state::delete_body(&svc, id, scope)
    }

    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error> {
        let target = args.scope.unwrap_or(Scope::Project);
        let svc = self.files();
        state::write_body(&svc, &args.id, target, &args.content, false)?;
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Build a hermetic `RulesServiceImpl` with both user and project
    /// roots pinned at fresh tempdirs. Skips `FilesService::resolve_roots`
    /// so tests don't pick up the dev's actual `~/.ship/` or the cwd's
    /// real project.
    fn fresh() -> (TempDir, RulesServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(user.join("rules")).unwrap();
        std::fs::create_dir_all(project.join("rules")).unwrap();
        let svc = RulesServiceImpl::with_roots(user, Some(project));
        (tmp, svc)
    }

    #[test]
    fn empty_list_when_no_rules() {
        let (_tmp, svc) = fresh();
        assert!(svc.list().is_empty());
    }

    #[test]
    fn create_then_list_then_get_round_trips() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "no-foo-bar".into(),
            content: "do not foo the bar".into(),
            scope: Some(Scope::Project),
        })
        .unwrap();

        let listed = svc.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "no-foo-bar");

        let got = svc.get("no-foo-bar", None).unwrap();
        assert_eq!(got.content, "do not foo the bar");
    }

    #[test]
    fn create_rejects_duplicate_id() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "rule-1".into(),
            content: "first".into(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let err = svc
            .create(CreateArgs {
                id: "rule-1".into(),
                content: "second".into(),
                scope: Some(Scope::Project),
            })
            .unwrap_err();
        assert!(matches!(err, state::Error::AlreadyExists(_)));
    }

    #[test]
    fn put_overwrites_existing() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "x".into(),
            content: "v1".into(),
            scope: Some(Scope::Project),
        })
        .unwrap();
        let scope = svc
            .put(PutArgs {
                id: "x".into(),
                content: "v2".into(),
                scope: None,
            })
            .unwrap();
        assert_eq!(scope, Scope::Project);
        assert_eq!(svc.get("x", None).unwrap().content, "v2");
    }

    #[test]
    fn delete_returns_scope_and_removes() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "doomed".into(),
            content: "bye".into(),
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
    fn get_with_invalid_id_errors_bad_id() {
        let (_tmp, svc) = fresh();
        let err = svc.get("BAD ID with spaces", None).unwrap_err();
        assert!(matches!(err, state::Error::BadId(_)));
    }
}
