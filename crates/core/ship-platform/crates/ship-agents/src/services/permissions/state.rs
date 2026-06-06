//! PermissionService — state layer.
//!
//! Permission presets live as top-level keys in `<scope>/permissions.jsonc`.
//! Keys starting with `_` or `$` (e.g. `_base_deny`, `$schema`) are treated
//! as private and filtered from `list()` + rejected as ids. Non-object
//! values (metadata like `default_mode`) are also filtered.

use ship_files::files::{Error as FileError, FilesService, Scope};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    AlreadyExists(String),
    BadId(String),
    Parse(String),
    Patch(String),
    Io(String),
    ScopeUnavailable(Scope),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "permission preset `{s}` not found"),
            Self::AlreadyExists(s) => write!(f, "permission preset `{s}` already exists"),
            Self::BadId(s) => write!(f, "bad id `{s}`"),
            Self::Parse(s) => write!(f, "parse: {s}"),
            Self::Patch(s) => write!(f, "patch: {s}"),
            Self::Io(s) => write!(f, "io: {s}"),
            Self::ScopeUnavailable(s) => write!(f, "scope {s:?} not configured"),
        }
    }
}

impl From<FileError> for Error {
    fn from(e: FileError) -> Self {
        match e {
            FileError::NotFound(p) => Self::NotFound(p.display().to_string()),
            FileError::AlreadyExists(p) => Self::AlreadyExists(p.display().to_string()),
            FileError::ScopeUnavailable(s) => Self::ScopeUnavailable(s),
            FileError::Json(s) => Self::Parse(s),
            FileError::Patch(s) => Self::Patch(s),
            FileError::Io(e) => Self::Io(e.to_string()),
            FileError::PathEscape(p) => Self::BadId(p.display().to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "specta", specta(rename = "PermissionScopeTag"))]
#[serde(rename_all = "lowercase")]
pub enum ScopeTag { Project, User }

impl From<Scope> for ScopeTag {
    fn from(s: Scope) -> Self {
        match s { Scope::Project => Self::Project, Scope::User => Self::User }
    }
}
impl From<ScopeTag> for Scope {
    fn from(s: ScopeTag) -> Self {
        match s { ScopeTag::Project => Self::Project, ScopeTag::User => Self::User }
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "specta", specta(rename = "PermissionIndexEntry"))]
pub struct IndexEntry {
    pub id: String,
    pub scope: ScopeTag,
}

fn is_private_key(k: &str) -> bool {
    k.is_empty() || k.starts_with('_') || k.starts_with('$')
}

fn validate_id(id: &str) -> Result<(), Error> {
    if is_private_key(id) {
        return Err(Error::BadId(id.to_string()));
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(Error::BadId(id.to_string()));
    }
    Ok(())
}

fn file_rel() -> PathBuf { PathBuf::from("permissions.jsonc") }

// ── Whole-file load/save ───────────────────────────────────────────

fn load_root(files: &FilesService, scope: Scope) -> Result<serde_json::Value, Error> {
    let rel = file_rel();
    if !files.exists(scope, &rel) {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }
    Ok(files.read_jsonc(scope, &rel)?)
}

fn save_root(files: &FilesService, scope: Scope, doc: &serde_json::Value) -> Result<(), Error> {
    files.write_jsonc(scope, &file_rel(), doc).map_err(Into::into)
}

fn root_mut(root: &mut serde_json::Value) -> &mut serde_json::Map<String, serde_json::Value> {
    if !root.is_object() {
        *root = serde_json::Value::Object(serde_json::Map::new());
    }
    root.as_object_mut().unwrap()
}

// ── Read ───────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Ok(root) = load_root(files, scope) else { continue };
        let Some(obj) = root.as_object() else { continue };
        for (k, v) in obj {
            if is_private_key(k) { continue }
            if !v.is_object() { continue }
            if !seen.insert(k.clone()) { continue }
            out.push(IndexEntry { id: k.clone(), scope: scope.into() });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn find(files: &FilesService, id: &str) -> Option<Scope> {
    for scope in [Scope::Project, Scope::User] {
        let Ok(root) = load_root(files, scope) else { continue };
        if root.get(id).and_then(|v| if v.is_object() { Some(()) } else { None }).is_some() {
            return Some(scope);
        }
    }
    None
}

fn exists_in_scope(files: &FilesService, scope: Scope, id: &str) -> bool {
    load_root(files, scope).ok()
        .and_then(|r| r.get(id).cloned())
        .map(|v| v.is_object())
        .unwrap_or(false)
}

pub fn load_entry(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<(Scope, serde_json::Value), Error> {
    validate_id(id)?;
    let scope = match explicit_scope {
        Some(s) if exists_in_scope(files, s, id) => s,
        Some(_) => return Err(Error::NotFound(id.into())),
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    let root = load_root(files, scope)?;
    let entry = root.get(id).cloned().ok_or_else(|| Error::NotFound(id.into()))?;
    Ok((scope, entry))
}

// ── Write ──────────────────────────────────────────────────────────

pub fn write_entry(
    files: &FilesService,
    id: &str,
    scope: Scope,
    entry: &serde_json::Value,
    allow_overwrite: bool,
) -> Result<(), Error> {
    validate_id(id)?;
    if !entry.is_object() {
        return Err(Error::Parse("preset entry must be an object".into()));
    }
    let mut root = load_root(files, scope)?;
    let obj = root_mut(&mut root);
    if !allow_overwrite && obj.contains_key(id) {
        return Err(Error::AlreadyExists(id.into()));
    }
    obj.insert(id.to_string(), entry.clone());
    save_root(files, scope, &root)?;
    Ok(())
}

pub fn patch_entry(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
    patch: &json_patch::Patch,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = match explicit_scope {
        Some(s) if exists_in_scope(files, s, id) => s,
        Some(_) => return Err(Error::NotFound(id.into())),
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    let mut root = load_root(files, scope)?;
    let mut entry = root.get(id).cloned().ok_or_else(|| Error::NotFound(id.into()))?;
    ship_files::path_tolerant::ensure_paths_for_adds(&mut entry, patch);
    json_patch::patch(&mut entry, patch).map_err(|e| Error::Patch(e.to_string()))?;
    let obj = root_mut(&mut root);
    obj.insert(id.to_string(), entry);
    save_root(files, scope, &root)?;
    Ok(scope)
}

pub fn delete_entry(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = match explicit_scope {
        Some(s) if exists_in_scope(files, s, id) => s,
        Some(_) => return Err(Error::NotFound(id.into())),
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    let mut root = load_root(files, scope)?;
    let obj = root_mut(&mut root);
    if obj.remove(id).is_none() {
        return Err(Error::NotFound(id.into()));
    }
    save_root(files, scope, &root)?;
    Ok(scope)
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
