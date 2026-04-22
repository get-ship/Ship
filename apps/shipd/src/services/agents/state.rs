//! AgentService — state layer. Pure domain logic over the file tree.
//! HTTP handlers in `mod.rs` are axum glue on top.
//!
//! Scope model: project wins by default. `?scope=user` forces user scope.
//! List returns both with a `scope` annotation per entry. Writes go to
//! the scope supplied by the caller; absent caller scope, project when
//! available else user.

use super::super::files::{Error as FileError, FilesService, Scope};
use serde::Serialize;
use std::path::{Path, PathBuf};

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

// ── Path resolution ─────────────────────────────────────────────────

fn rel_for(id: &str) -> PathBuf { PathBuf::from(format!("agents/{id}.jsonc")) }

/// Locate an agent file across the two scopes (project wins). Returns
/// the scope that held it and the relative path used to reach it.
pub fn find(files: &FilesService, id: &str) -> Option<(Scope, PathBuf)> {
    let rel = rel_for(id);
    for scope in [Scope::Project, Scope::User] {
        if files.exists(scope, &rel) {
            return Some((scope, rel.clone()));
        }
        // Tolerate .json (schema permits it).
        let alt = PathBuf::from(format!("agents/{id}.json"));
        if files.exists(scope, &alt) {
            return Some((scope, alt));
        }
    }
    None
}

fn find_in_scope(files: &FilesService, scope: Scope, id: &str) -> Option<PathBuf> {
    let rel = rel_for(id);
    if files.exists(scope, &rel) { return Some(rel) }
    let alt = PathBuf::from(format!("agents/{id}.json"));
    if files.exists(scope, &alt) { return Some(alt) }
    None
}

// ── Read ───────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Ok(entries) = files.list_dir(scope, Path::new("agents")) else { continue };
        for p in entries {
            let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { continue };
            let ext = p.extension().and_then(|s| s.to_str());
            if !matches!(ext, Some("jsonc" | "json")) { continue }
            if !seen.insert(stem.to_string()) { continue }  // project wins
            out.push(IndexEntry { id: stem.to_string(), scope: scope.into() });
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
    let (scope, rel) = match explicit_scope {
        Some(s) => {
            let rel = find_in_scope(files, s, id).ok_or_else(|| Error::NotFound(id.into()))?;
            (s, rel)
        }
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    let doc: serde_json::Value = files.read_jsonc(scope, &rel)?;
    Ok((scope, doc))
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
    let rel = rel_for(id);
    if !allow_overwrite && files.exists(scope, &rel) {
        return Err(Error::AlreadyExists(id.into()));
    }
    files.write_jsonc(scope, &rel, doc)?;
    Ok(())
}

pub fn patch_doc(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
    patch: &json_patch::Patch,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let (scope, rel) = match explicit_scope {
        Some(s) => {
            let rel = find_in_scope(files, s, id).ok_or_else(|| Error::NotFound(id.into()))?;
            (s, rel)
        }
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    // We load → ensure parent paths exist for `add` ops → apply → write.
    // The path-tolerant wrapper keeps templates' one-liners honest: an
    // `add /skills/refs/-` op succeeds even if `/skills/refs` wasn't
    // there before. Strictly this bends RFC 6902 (which fails when
    // ancestors are missing), but it matches user expectations and
    // avoids every client having to do a GET-then-PUT dance.
    let mut doc: serde_json::Value = files.read_jsonc(scope, &rel)?;
    ensure_paths_for_adds(&mut doc, patch);
    json_patch::patch(&mut doc, patch).map_err(|e| Error::Patch(e.to_string()))?;
    files.write_jsonc(scope, &rel, &doc)?;
    Ok(scope)
}

/// Walk every `add` op's path and create missing ancestors as empty
/// objects / arrays. No-ops for paths that already exist and for the
/// strictly-defined ops (`test`, `remove`, `replace`, `move`, `copy`) —
/// those follow RFC 6902 exactly.
fn ensure_paths_for_adds(doc: &mut serde_json::Value, patch: &json_patch::Patch) {
    use json_patch::PatchOperation as Op;
    for op in patch.0.iter() {
        let Op::Add(add) = op else { continue };
        let path = add.path.to_string();
        if path.is_empty() { continue }
        // Strip trailing `-` (RFC 6902 array-append) so we think about
        // the container, not the slot.
        let container_path = if path.ends_with("/-") {
            &path[..path.len() - 2]
        } else {
            path.as_str()
        };
        // Walk segment by segment. Segments are `/`-separated after
        // the leading `/`. RFC 6901 escapes are rare in our schema;
        // we don't bother unescaping here.
        let segments: Vec<&str> = container_path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        if segments.is_empty() { continue }
        // Descend into `doc`, creating empty objects along the way.
        // Reborrow each step so the loop doesn't move the outer mutable
        // reference. Escape the loop on non-object cursors — those are
        // RFC-6902 violations the patch call itself will surface.
        let last_idx = segments.len().saturating_sub(1);
        let ends_with_append = path.ends_with("/-");
        let mut cursor: &mut serde_json::Value = doc;
        for (i, seg) in segments.iter().enumerate() {
            let is_last = i == last_idx;
            if !cursor.is_object() { break }
            let obj = cursor.as_object_mut().unwrap();
            if !obj.contains_key(*seg) {
                let placeholder = if is_last && ends_with_append {
                    serde_json::Value::Array(Vec::new())
                } else {
                    serde_json::Value::Object(serde_json::Map::new())
                };
                obj.insert((*seg).to_string(), placeholder);
            }
            cursor = obj.get_mut(*seg).unwrap();
        }
        // `cursor` is dropped at end of iteration; `doc` reborrows next pass.
        let _ = cursor;
    }
}

pub fn delete_doc(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let (scope, rel) = match explicit_scope {
        Some(s) => {
            let rel = find_in_scope(files, s, id).ok_or_else(|| Error::NotFound(id.into()))?;
            (s, rel)
        }
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    files.trash(scope, &rel)?;
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

    let (scope, _rel) = match explicit_scope {
        Some(s) => {
            let rel = find_in_scope(files, s, id).ok_or_else(|| Error::NotFound(id.into()))?;
            (s, rel)
        }
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    // Refuse if the new id collides within the same scope.
    if find_in_scope(files, scope, new_id).is_some() {
        return Err(Error::AlreadyExists(new_id.into()));
    }

    let (_, doc) = load_doc(files, id, Some(scope))?;
    // Also update the agent.id field inside the doc when present so the
    // persisted representation stays consistent with the filename stem.
    let doc = with_agent_id(doc, new_id);
    write_doc(files, new_id, scope, &doc, false)?;
    // Remove the old file atomically via trash — safer than deleting
    // outright in case the caller wants to recover.
    delete_doc(files, id, Some(scope))?;
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
    if find_in_scope(files, scope, new_id).is_some() {
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
    if find_in_scope(files, to, id).is_some() {
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
    if let Some(obj) = doc.as_object_mut() {
        if let Some(agent) = obj.get_mut("agent").and_then(|v| v.as_object_mut()) {
            agent.insert("id".into(), serde_json::Value::String(new_id.into()));
        }
    }
    doc
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
