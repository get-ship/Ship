//! AgentsService — state layer. Pure domain logic over the agent-def tree.
//! HTTP handlers in `mod.rs` are axum glue on top.
//!
//! Scope model: project wins by default. `?scope=user` forces user scope.
//! List returns both with a `scope` annotation per entry. Writes go to
//! the scope supplied by the caller; absent caller scope, project when
//! available else user.
//!
//! Disk boundary (S5b): EVERY agent-def read/write/locate/trash flows through
//! THE one FsBroker write door (`ship_config::agents_write`) + the S2 reader
//! (`ship_config::agents`). `FilesService` is used ONLY to resolve a `Scope` to
//! its `.ship` dir — zero raw `std::fs` on agent-def paths in this module.

use ship_files::files::{Error as FileError, FilesService, Scope};
use ship_config::agents_write as door;
use ship_substrate::ports::LocalFsBroker;
use serde::Serialize;

/// The gated disk door every def op rides. `LocalFsBroker` is stateless.
fn fs() -> LocalFsBroker {
    LocalFsBroker::new()
}

impl From<ship_config::error::ConfigError> for Error {
    fn from(e: ship_config::error::ConfigError) -> Self {
        use ship_config::error::ConfigError as C;
        match e {
            C::NotFound(m) => Self::NotFound(m),
            C::AlreadyExists(m) => Self::AlreadyExists(m),
            C::Validation(m) => Self::Parse(m),
            C::Fs(m) | C::Internal(m) | C::Forbidden(m) => Self::Io(m),
        }
    }
}

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
            Self::NotFound(s) => write!(f, "agent `{s}` not found"),
            Self::AlreadyExists(s) => write!(f, "agent `{s}` already exists"),
            Self::BadId(s) => write!(f, "bad id `{s}` — must match ^[a-z0-9-]+$"),
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
#[serde(rename_all = "lowercase")]
pub enum ScopeTag {
    Project,
    User,
}

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
pub struct IndexEntry {
    pub id: String,
    pub scope: ScopeTag,
}

// ── Id validation ───────────────────────────────────────────────────

fn validate_id(id: &str) -> Result<(), Error> {
    if id.is_empty()
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || id.starts_with('-')
        || id.ends_with('-')
    {
        return Err(Error::BadId(id.to_string()));
    }
    Ok(())
}

// ── Path resolution (FsBroker door) ─────────────────────────────────

/// Resolve `scope`'s `.ship` dir. `None` when project scope has no discovered
/// root (`?scope=project` with no enclosing project) — callers treat that as
/// "scope holds nothing".
fn scope_dir(files: &FilesService, scope: Scope) -> Option<&std::path::Path> {
    files.scope_root(scope).ok()
}

/// `true` when `scope` holds a def for `id` (either ext), via the door's locate.
fn present_in(files: &FilesService, scope: Scope, id: &str) -> bool {
    scope_dir(files, scope)
        .and_then(|dir| door::find_agent_def(&fs(), dir, id).ok().flatten())
        .is_some()
}

/// Locate an agent def across the two scopes (project wins). Returns the scope
/// that held it. The legacy `PathBuf` second tuple element is retained for the
/// service-layer call shape; it is the agents-relative filename.
pub fn find(files: &FilesService, id: &str) -> Option<(Scope, std::path::PathBuf)> {
    for scope in [Scope::Project, Scope::User] {
        if let Some(dir) = scope_dir(files, scope)
            && let Ok(Some(leaf)) = door::find_agent_def(&fs(), dir, id)
        {
            return Some((scope, std::path::PathBuf::from(format!("agents/{leaf}"))));
        }
    }
    None
}

// ── Read ───────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Some(dir) = scope_dir(files, scope) else { continue };
        let Ok(defs) = ship_config::agents::read_agent_defs(&fs(), dir) else { continue };
        for d in defs {
            if !seen.insert(d.id.clone()) { continue }  // project wins
            out.push(IndexEntry { id: d.id, scope: scope.into() });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Load the raw parsed document. The service is shape-agnostic at this
/// layer — schema enforcement happens (future) via `validate`.
pub fn load_doc(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<(Scope, serde_json::Value), Error> {
    validate_id(id)?;
    let scope = resolve_holder(files, id, explicit_scope)?;
    let dir = scope_dir(files, scope).ok_or_else(|| Error::NotFound(id.into()))?;
    let (_leaf, doc) = door::read_agent_def_value(&fs(), dir, id)?
        .ok_or_else(|| Error::NotFound(id.into()))?;
    Ok((scope, doc))
}

/// Resolve which scope holds `id`: the forced scope (must hold it) or the
/// project-wins default search.
fn resolve_holder(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    match explicit_scope {
        Some(s) if present_in(files, s, id) => Ok(s),
        Some(_) => Err(Error::NotFound(id.into())),
        None => find(files, id).map(|(s, _)| s).ok_or_else(|| Error::NotFound(id.into())),
    }
}

// ── Write ──────────────────────────────────────────────────────────

/// Write (atomic). Refuses when the id exists in the target scope
/// and `allow_overwrite` is false — PUT callers set true, create
/// callers set false.
pub fn write_doc(
    files: &FilesService,
    id: &str,
    scope: Scope,
    doc: &serde_json::Value,
    allow_overwrite: bool,
) -> Result<(), Error> {
    validate_id(id)?;
    let dir = files.scope_root(scope)?;
    door::write_agent_def(&fs(), dir, id, doc, allow_overwrite)?;
    Ok(())
}

pub fn patch_doc(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
    patch: &json_patch::Patch,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = resolve_holder(files, id, explicit_scope)?;
    let dir = files.scope_root(scope)?;
    // Load → ensure parent paths exist for `add` ops → apply → write, all disk
    // through the door. The path-tolerant wrapper keeps templates' one-liners
    // honest: an `add /skills/refs/-` op succeeds even if `/skills/refs` wasn't
    // there before. Strictly this bends RFC 6902 (which fails when ancestors
    // are missing), but it matches user expectations and avoids every client
    // having to do a GET-then-PUT dance.
    let (_leaf, mut doc) = door::read_agent_def_value(&fs(), dir, id)?
        .ok_or_else(|| Error::NotFound(id.into()))?;
    ship_files::path_tolerant::ensure_paths_for_adds(&mut doc, patch);
    json_patch::patch(&mut doc, patch).map_err(|e| Error::Patch(e.to_string()))?;
    door::write_agent_def(&fs(), dir, id, &doc, true)?;
    Ok(scope)
}

pub fn delete_doc(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = resolve_holder(files, id, explicit_scope)?;
    let dir = files.scope_root(scope)?;
    door::delete_agent_def(&fs(), dir, id)?;
    Ok(scope)
}

// ── Lifecycle actions ──────────────────────────────────────────────

pub fn rename(
    files: &FilesService,
    id: &str,
    new_id: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    validate_id(new_id)?;
    if id == new_id { return Ok(explicit_scope.unwrap_or(Scope::Project)) }

    let scope = resolve_holder(files, id, explicit_scope)?;
    // Refuse if the new id collides within the same scope.
    if present_in(files, scope, new_id) {
        return Err(Error::AlreadyExists(new_id.into()));
    }
    let dir = files.scope_root(scope)?;
    let (_leaf, doc) = door::read_agent_def_value(&fs(), dir, id)?
        .ok_or_else(|| Error::NotFound(id.into()))?;
    // Update the agent.id field inside the doc so the persisted representation
    // stays consistent with the filename stem. The door writes the new file and
    // trashes the old (recoverable, not a hard delete).
    let doc = with_agent_id(doc, new_id);
    door::rename_agent_def(&fs(), dir, id, new_id, &doc)?;
    Ok(scope)
}

pub fn duplicate(
    files: &FilesService,
    id: &str,
    new_id: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    validate_id(new_id)?;
    let (scope, doc) = load_doc(files, id, explicit_scope)?;
    if present_in(files, scope, new_id) {
        return Err(Error::AlreadyExists(new_id.into()));
    }
    let doc = with_agent_id(doc, new_id);
    write_doc(files, new_id, scope, &doc, false)?;
    Ok(scope)
}

pub fn move_scope(
    files: &FilesService,
    id: &str,
    from: Scope,
    to: Scope,
) -> Result<(), Error> {
    validate_id(id)?;
    if from == to { return Ok(()) }
    let (_scope, doc) = load_doc(files, id, Some(from))?;
    if present_in(files, to, id) {
        return Err(Error::AlreadyExists(id.into()));
    }
    write_doc(files, id, to, &doc, false)?;
    delete_doc(files, id, Some(from))?;
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────

/// When renaming/duplicating we also update `agent.id` inside the doc so
/// the id field tracks the filename. Non-destructive to other keys.
fn with_agent_id(mut doc: serde_json::Value, new_id: &str) -> serde_json::Value {
    if let Some(obj) = doc.as_object_mut()
        && let Some(agent) = obj.get_mut("agent").and_then(|v| v.as_object_mut()) {
            agent.insert("id".into(), serde_json::Value::String(new_id.into()));
        }
    doc
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
