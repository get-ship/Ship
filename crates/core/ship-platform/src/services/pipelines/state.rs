//! PipelineService — state layer. Pure domain logic over the file tree.
//!
//! Pipelines live at `<scope>/pipelines/<name>.jsonc`. JSONC is the
//! only supported format — legacy YAML files at the same name are
//! ignored by the loader.
//!
//! Scope model: project wins by default, exactly as AgentsService.

use crate::services::files::{Error as FileError, FilesService, Scope};
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
            Self::NotFound(s) => write!(f, "pipeline `{s}` not found"),
            Self::AlreadyExists(s) => write!(f, "pipeline `{s}` already exists"),
            Self::BadId(s) => write!(f, "bad name `{s}` — must match ^[a-z0-9-]+$"),
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
#[cfg_attr(feature = "specta", specta(rename = "PipelineScopeTag"))]
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
#[cfg_attr(feature = "specta", specta(rename = "PipelineIndexEntry"))]
pub struct IndexEntry {
    pub name: String,
    pub scope: ScopeTag,
    /// On-disk extension — `"jsonc" | "json" | "yaml" | "yml"`.
    pub format: String,
}

// ── Id validation ───────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || name.starts_with('-')
        || name.ends_with('-')
    {
        return Err(Error::BadId(name.to_string()));
    }
    Ok(())
}

// ── Path resolution ─────────────────────────────────────────────────

/// Canonical pipeline extension. JSONC is the only format.
const PIPELINE_EXT: &str = "jsonc";

fn rel_for(name: &str) -> PathBuf {
    PathBuf::from(format!("pipelines/{name}.{PIPELINE_EXT}"))
}

/// Locate a pipeline across both scopes (project wins). Returns the scope
/// and relative path.
pub fn find(files: &FilesService, name: &str) -> Option<(Scope, PathBuf)> {
    for scope in [Scope::Project, Scope::User] {
        let rel = rel_for(name);
        if files.exists(scope, &rel) {
            return Some((scope, rel));
        }
    }
    None
}

fn find_in_scope(files: &FilesService, scope: Scope, name: &str) -> Option<PathBuf> {
    let rel = rel_for(name);
    if files.exists(scope, &rel) { Some(rel) } else { None }
}

// ── Read ───────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Ok(entries) = files.list_dir(scope, Path::new("pipelines")) else { continue };
        for p in entries {
            let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { continue };
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext != PIPELINE_EXT { continue }
            if !seen.insert(stem.to_string()) { continue } // project wins
            out.push(IndexEntry {
                name: stem.to_string(),
                scope: scope.into(),
                format: PIPELINE_EXT.to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a pipeline definition as raw JSON. Pipelines are JSONC; the
/// returned value is always `serde_json::Value`.
pub fn load_doc(
    files: &FilesService,
    name: &str,
    explicit_scope: Option<Scope>,
) -> Result<(Scope, String, serde_json::Value), Error> {
    validate_name(name)?;
    let (scope, rel) = match explicit_scope {
        Some(s) => (
            s,
            find_in_scope(files, s, name).ok_or_else(|| Error::NotFound(name.into()))?,
        ),
        None => find(files, name).ok_or_else(|| Error::NotFound(name.into()))?,
    };
    let raw = files.read_string(scope, &rel)?;
    let json = ship_substrate::fs_util::strip_jsonc_comments(&raw);
    let doc = serde_json::from_str(&json).map_err(|e| Error::Parse(e.to_string()))?;
    Ok((scope, PIPELINE_EXT.to_string(), doc))
}

// ── Write ──────────────────────────────────────────────────────────

/// Write a pipeline doc. Always writes JSONC (`.jsonc`).
pub fn write_doc(
    files: &FilesService,
    name: &str,
    scope: Scope,
    doc: &serde_json::Value,
    allow_overwrite: bool,
) -> Result<(), Error> {
    validate_name(name)?;
    let target = rel_for(name);
    if find_in_scope(files, scope, name).is_some() && !allow_overwrite {
        return Err(Error::AlreadyExists(name.into()));
    }
    files.write_jsonc(scope, &target, doc)?;
    Ok(())
}

pub fn patch_doc(
    files: &FilesService,
    name: &str,
    explicit_scope: Option<Scope>,
    patch: &json_patch::Patch,
) -> Result<Scope, Error> {
    validate_name(name)?;
    let (scope, rel) = match explicit_scope {
        Some(s) => (
            s,
            find_in_scope(files, s, name).ok_or_else(|| Error::NotFound(name.into()))?,
        ),
        None => find(files, name).ok_or_else(|| Error::NotFound(name.into()))?,
    };
    let raw = files.read_string(scope, &rel)?;
    let json = ship_substrate::fs_util::strip_jsonc_comments(&raw);
    let mut doc: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| Error::Parse(e.to_string()))?;
    crate::services::path_tolerant::ensure_paths_for_adds(&mut doc, patch);
    json_patch::patch(&mut doc, patch).map_err(|e| Error::Patch(e.to_string()))?;
    files.write_jsonc(scope, &rel, &doc)?;
    Ok(scope)
}

pub fn delete_doc(
    files: &FilesService,
    name: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_name(name)?;
    let (scope, rel) = match explicit_scope {
        Some(s) => (
            s,
            find_in_scope(files, s, name).ok_or_else(|| Error::NotFound(name.into()))?,
        ),
        None => find(files, name).ok_or_else(|| Error::NotFound(name.into()))?,
    };
    files.trash(scope, &rel)?;
    Ok(scope)
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
