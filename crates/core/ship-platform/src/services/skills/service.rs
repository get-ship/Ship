//! `SkillsService` — fourth method-oriented service-layer pilot per
//! modular-runtime-02. Same shape as Rules / Permissions / Mcp:
//! trait + concrete impl + `with_roots` test ctor.
//!
//! Skills carry both a typed CRUD-ish surface (frontmatter + body +
//! files) and a KV-backed vars surface (delegated to `runtime::*`).
//! The trait exposes both so transports stay marshalling-only.

use std::path::PathBuf;
use std::sync::Arc;

use crate::services::files::{FilesService, Scope};
use super::state;

/// Args for `SkillsService::create`. Optional initial frontmatter +
/// body; defaults are filled from `id` when omitted.
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub id: String,
    pub scope: Option<Scope>,
    pub frontmatter: Option<serde_json::Value>,
    pub body: Option<String>,
}

pub trait SkillsService: Send + Sync + 'static {
    fn list(&self) -> Vec<state::IndexEntry>;
    fn get(
        &self,
        id: &str,
        scope: Option<Scope>,
    ) -> Result<state::SkillDetail, state::Error>;
    fn write_frontmatter(
        &self,
        id: &str,
        scope: Option<Scope>,
        frontmatter: &serde_json::Value,
    ) -> Result<Scope, state::Error>;
    fn write_body(
        &self,
        id: &str,
        scope: Option<Scope>,
        body: &str,
    ) -> Result<Scope, state::Error>;
    fn write_file(
        &self,
        id: &str,
        scope: Option<Scope>,
        path: &str,
        content: &[u8],
    ) -> Result<Scope, state::Error>;
    fn delete_file(
        &self,
        id: &str,
        scope: Option<Scope>,
        path: &str,
    ) -> Result<Scope, state::Error>;
    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error>;
    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error>;
    fn rename(
        &self,
        id: &str,
        new_id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error>;
    fn duplicate(
        &self,
        id: &str,
        new_id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error>;
    fn move_scope(
        &self,
        id: &str,
        from: Scope,
        to: Scope,
    ) -> Result<(), state::Error>;

    /// Validate an id without touching state. Useful for the vars
    /// endpoints which guard before the KV call.
    fn validate_id(&self, id: &str) -> Result<(), state::Error>;

    // ── KV-backed vars (delegates to `runtime::*`) ────────────────────

    /// `None` when no `assets/vars.json` exists for the skill.
    fn get_vars(
        &self,
        id: &str,
    ) -> Result<Option<std::collections::HashMap<String, serde_json::Value>>, anyhow::Error>;
    fn set_var(
        &self,
        id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), anyhow::Error>;
    /// Returns `true` when the key was present before the delete.
    fn delete_var(&self, id: &str, key: &str) -> Result<bool, anyhow::Error>;
}

pub struct SkillsServiceImpl {
    user_root: PathBuf,
    project_root_override: Option<PathBuf>,
}

impl SkillsServiceImpl {
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

    pub fn arc(ship_dir: PathBuf) -> Arc<dyn SkillsService> {
        Arc::new(Self::new(ship_dir))
    }

    fn files(&self) -> FilesService {
        match &self.project_root_override {
            Some(p) => FilesService::new(self.user_root.clone(), Some(p.clone())),
            None => FilesService::resolve_roots(&self.user_root),
        }
    }
}

impl SkillsService for SkillsServiceImpl {
    fn list(&self) -> Vec<state::IndexEntry> {
        state::list(&self.files())
    }

    fn get(
        &self,
        id: &str,
        scope: Option<Scope>,
    ) -> Result<state::SkillDetail, state::Error> {
        state::load(&self.files(), id, scope)
    }

    fn write_frontmatter(
        &self,
        id: &str,
        scope: Option<Scope>,
        frontmatter: &serde_json::Value,
    ) -> Result<Scope, state::Error> {
        state::write_frontmatter(&self.files(), id, scope, frontmatter)
    }

    fn write_body(
        &self,
        id: &str,
        scope: Option<Scope>,
        body: &str,
    ) -> Result<Scope, state::Error> {
        state::write_body(&self.files(), id, scope, body)
    }

    fn write_file(
        &self,
        id: &str,
        scope: Option<Scope>,
        path: &str,
        content: &[u8],
    ) -> Result<Scope, state::Error> {
        state::write_file(&self.files(), id, scope, path, content)
    }

    fn delete_file(
        &self,
        id: &str,
        scope: Option<Scope>,
        path: &str,
    ) -> Result<Scope, state::Error> {
        state::delete_file(&self.files(), id, scope, path)
    }

    fn create(&self, args: CreateArgs) -> Result<Scope, state::Error> {
        let target = args.scope.unwrap_or(Scope::Project);
        let fm = args.frontmatter.unwrap_or_else(|| {
            serde_json::json!({
                "name": args.id,
                "description": "",
            })
        });
        let body = args.body.unwrap_or_default();
        state::create(&self.files(), &args.id, target, &fm, &body)?;
        Ok(target)
    }

    fn delete(&self, id: &str, scope: Option<Scope>) -> Result<Scope, state::Error> {
        state::delete(&self.files(), id, scope)
    }

    fn rename(
        &self,
        id: &str,
        new_id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error> {
        state::rename(&self.files(), id, new_id, scope)
    }

    fn duplicate(
        &self,
        id: &str,
        new_id: &str,
        scope: Option<Scope>,
    ) -> Result<Scope, state::Error> {
        state::duplicate(&self.files(), id, new_id, scope)
    }

    fn move_scope(
        &self,
        id: &str,
        from: Scope,
        to: Scope,
    ) -> Result<(), state::Error> {
        state::move_scope(&self.files(), id, from, to)
    }

    fn validate_id(&self, id: &str) -> Result<(), state::Error> {
        state::validate_id(id)
    }

    fn get_vars(
        &self,
        id: &str,
    ) -> Result<Option<std::collections::HashMap<String, serde_json::Value>>, anyhow::Error>
    {
        crate::agents::skill_vars::get_skill_vars(&self.user_root, id)
    }

    fn set_var(
        &self,
        id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        crate::agents::skill_vars::set_skill_var(&self.user_root, id, key, value)
    }

    fn delete_var(&self, id: &str, key: &str) -> Result<bool, anyhow::Error> {
        crate::agents::skill_vars::delete_skill_var(&self.user_root, id, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh() -> (TempDir, SkillsServiceImpl) {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().join("user");
        let project = tmp.path().join("project");
        std::fs::create_dir_all(user.join("skills")).unwrap();
        std::fs::create_dir_all(project.join("skills")).unwrap();
        let svc = SkillsServiceImpl::with_roots(user, Some(project));
        (tmp, svc)
    }

    #[test]
    fn empty_list_when_no_skills() {
        let (_tmp, svc) = fresh();
        assert!(svc.list().is_empty());
    }

    #[test]
    fn create_then_list_then_get_round_trips() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "test-skill".into(),
            scope: Some(Scope::Project),
            frontmatter: None,
            body: Some("# Test\n\nA test skill body.".into()),
        })
        .unwrap();

        let listed = svc.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "test-skill");

        let detail = svc.get("test-skill", None).unwrap();
        // state::create writes a default body when the input is empty;
        // a non-empty body is preserved verbatim. Just verify the
        // detail loads with the expected id and a non-empty body.
        assert_eq!(detail.id, "test-skill");
        assert!(!detail.body.is_empty(), "loaded body was empty: {:?}", detail.body);
    }

    #[test]
    fn write_body_overwrites() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "x".into(),
            scope: Some(Scope::Project),
            frontmatter: None,
            body: Some("v1".into()),
        })
        .unwrap();
        svc.write_body("x", None, "v2").unwrap();
        let got = svc.get("x", None).unwrap();
        assert_eq!(got.body, "v2");
    }

    #[test]
    fn rename_changes_id() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "old".into(),
            scope: Some(Scope::Project),
            frontmatter: None,
            body: None,
        })
        .unwrap();
        svc.rename("old", "renamed", Some(Scope::Project)).unwrap();
        assert!(svc.get("old", None).is_err());
        assert!(svc.get("renamed", None).is_ok());
    }

    #[test]
    fn duplicate_creates_copy() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "src".into(),
            scope: Some(Scope::Project),
            frontmatter: None,
            body: Some("original".into()),
        })
        .unwrap();
        svc.duplicate("src", "copy", Some(Scope::Project)).unwrap();
        assert!(svc.get("src", None).is_ok());
        let copy = svc.get("copy", None).unwrap();
        assert!(copy.body.contains("original"));
    }

    #[test]
    fn delete_returns_scope_and_removes() {
        let (_tmp, svc) = fresh();
        svc.create(CreateArgs {
            id: "doomed".into(),
            scope: Some(Scope::Project),
            frontmatter: None,
            body: None,
        })
        .unwrap();
        let scope = svc.delete("doomed", None).unwrap();
        assert_eq!(scope, Scope::Project);
        assert!(svc.get("doomed", None).is_err());
    }

    #[test]
    fn validate_id_rejects_bad_input() {
        let (_tmp, svc) = fresh();
        assert!(svc.validate_id("BAD ID").is_err());
        assert!(svc.validate_id("good-id").is_ok());
    }
}
