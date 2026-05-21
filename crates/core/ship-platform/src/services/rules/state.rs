//! RuleService — state layer. Pure domain logic over the file tree.
//!
//! Rules live as markdown files at `<scope>/rules/<id>.md`. Body is plain
//! markdown — there's no structured payload, so PATCH is not supported at
//! this layer (the HTTP handler rejects PATCH with 405).
//!
//! Scope model: project wins by default. `?scope=user` forces user scope.
//! List returns both with a `scope` annotation per entry; project entries
//! shadow user on id collision.

use crate::services::files::{Error as FileError, FilesService, Scope};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    AlreadyExists(String),
    BadId(String),
    Io(String),
    ScopeUnavailable(Scope),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "rule `{s}` not found"),
            Self::AlreadyExists(s) => write!(f, "rule `{s}` already exists"),
            Self::BadId(s) => write!(f, "bad id `{s}` — must match ^[a-z0-9-]+$"),
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
            FileError::Json(s) => Self::Io(s),
            FileError::Patch(s) => Self::Io(s),
            FileError::Io(e) => Self::Io(e.to_string()),
            FileError::PathEscape(p) => Self::BadId(p.display().to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "specta", specta(rename = "RuleScopeTag"))]
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
#[cfg_attr(feature = "specta", specta(rename = "RuleIndexEntry"))]
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

fn rel_for(id: &str) -> PathBuf { PathBuf::from(format!("rules/{id}.md")) }

pub fn find(files: &FilesService, id: &str) -> Option<(Scope, PathBuf)> {
    let rel = rel_for(id);
    for scope in [Scope::Project, Scope::User] {
        if files.exists(scope, &rel) {
            return Some((scope, rel.clone()));
        }
    }
    None
}

fn find_in_scope(files: &FilesService, scope: Scope, id: &str) -> Option<PathBuf> {
    let rel = rel_for(id);
    if files.exists(scope, &rel) { Some(rel) } else { None }
}

// ── Read ───────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Ok(entries) = files.list_dir(scope, Path::new("rules")) else { continue };
        for p in entries {
            let ext = p.extension().and_then(|s| s.to_str());
            if ext != Some("md") { continue }
            let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { continue };
            if stem.eq_ignore_ascii_case("README") { continue }
            if !seen.insert(stem.to_string()) { continue } // project wins
            out.push(IndexEntry { id: stem.to_string(), scope: scope.into() });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Load the raw markdown body. Returns (scope, body).
pub fn load_body(
    files: &FilesService,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<(Scope, String), Error> {
    validate_id(id)?;
    let (scope, rel) = match explicit_scope {
        Some(s) => {
            let rel = find_in_scope(files, s, id).ok_or_else(|| Error::NotFound(id.into()))?;
            (s, rel)
        }
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into()))?,
    };
    let body = files.read_string(scope, &rel)?;
    Ok((scope, body))
}

// ── Write ──────────────────────────────────────────────────────────

pub fn write_body(
    files: &FilesService,
    id: &str,
    scope: Scope,
    content: &str,
    allow_overwrite: bool,
) -> Result<(), Error> {
    validate_id(id)?;
    let rel = rel_for(id);
    if !allow_overwrite && files.exists(scope, &rel) {
        return Err(Error::AlreadyExists(id.into()));
    }
    files.write(scope, &rel, content.as_bytes())?;
    Ok(())
}

pub fn delete_body(
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

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
