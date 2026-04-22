//! PipelineService — state layer. Pure domain logic over the file tree.
//!
//! Pipelines live at `<scope>/pipelines/<name>.{jsonc,json,yaml,yml}`.
//! The canonical write format is JSONC; YAML is legacy but still
//! discovered on reads so projects mid-migration don't silently lose
//! their definitions.
//!
//! Scope model: project wins by default, exactly as AgentService.

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

/// Recognised pipeline extensions, preferred first (JSONC is canonical
/// write target; others surface on read for legacy definitions).
const EXTS: &[&str] = &["jsonc", "json", "yaml", "yml"];

fn rel_for(name: &str, ext: &str) -> PathBuf {
    PathBuf::from(format!("pipelines/{name}.{ext}"))
}

/// Locate a pipeline across both scopes (project wins). Returns the scope,
/// relative path, and extension found.
pub fn find(files: &FilesService, name: &str) -> Option<(Scope, PathBuf, String)> {
    for scope in [Scope::Project, Scope::User] {
        for ext in EXTS {
            let rel = rel_for(name, ext);
            if files.exists(scope, &rel) {
                return Some((scope, rel, (*ext).to_string()));
            }
        }
    }
    None
}

fn find_in_scope(files: &FilesService, scope: Scope, name: &str) -> Option<(PathBuf, String)> {
    for ext in EXTS {
        let rel = rel_for(name, ext);
        if files.exists(scope, &rel) {
            return Some((rel, (*ext).to_string()));
        }
    }
    None
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
            if !EXTS.contains(&ext) { continue }
            if !seen.insert(stem.to_string()) { continue } // project wins
            out.push(IndexEntry {
                name: stem.to_string(),
                scope: scope.into(),
                format: ext.to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a pipeline definition as raw JSON. YAML sources are parsed and
/// converted through `serde_yaml::Value` so the output shape is always
/// a `serde_json::Value`.
pub fn load_doc(
    files: &FilesService,
    name: &str,
    explicit_scope: Option<Scope>,
) -> Result<(Scope, String, serde_json::Value), Error> {
    validate_name(name)?;
    let (scope, rel, ext) = match explicit_scope {
        Some(s) => {
            let (rel, ext) = find_in_scope(files, s, name)
                .ok_or_else(|| Error::NotFound(name.into()))?;
            (s, rel, ext)
        }
        None => find(files, name).ok_or_else(|| Error::NotFound(name.into()))?,
    };
    let doc = parse_pipeline_bytes(files, scope, &rel, &ext)?;
    Ok((scope, ext, doc))
}

fn parse_pipeline_bytes(
    files: &FilesService,
    scope: Scope,
    rel: &Path,
    ext: &str,
) -> Result<serde_json::Value, Error> {
    let raw = files.read_string(scope, rel)?;
    match ext {
        "jsonc" | "json" => {
            let json = compiler::jsonc::strip_jsonc_comments(&raw);
            serde_json::from_str(&json).map_err(|e| Error::Parse(e.to_string()))
        }
        "yaml" | "yml" => {
            let v: serde_yaml::Value = serde_yaml::from_str(&raw)
                .map_err(|e| Error::Parse(e.to_string()))?;
            serde_json::to_value(v).map_err(|e| Error::Parse(e.to_string()))
        }
        other => Err(Error::Parse(format!("unsupported extension `{other}`"))),
    }
}

// ── Write ──────────────────────────────────────────────────────────

/// Write a pipeline doc. Always writes JSONC (`.jsonc`). If an existing
/// legacy YAML file is present, it is trashed so the two formats can't
/// drift.
pub fn write_doc(
    files: &FilesService,
    name: &str,
    scope: Scope,
    doc: &serde_json::Value,
    allow_overwrite: bool,
) -> Result<(), Error> {
    validate_name(name)?;
    let target = rel_for(name, "jsonc");
    let existing = find_in_scope(files, scope, name);
    if existing.is_some() && !allow_overwrite {
        return Err(Error::AlreadyExists(name.into()));
    }
    // If there is a legacy YAML copy at the same name, retire it so the
    // project converges on JSONC.
    if let Some((rel, ext)) = existing
        && ext != "jsonc"
    {
        let _ = files.trash(scope, &rel);
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
    let (scope, rel, ext) = match explicit_scope {
        Some(s) => {
            let (rel, ext) = find_in_scope(files, s, name)
                .ok_or_else(|| Error::NotFound(name.into()))?;
            (s, rel, ext)
        }
        None => find(files, name).ok_or_else(|| Error::NotFound(name.into()))?,
    };
    let mut doc = parse_pipeline_bytes(files, scope, &rel, &ext)?;
    ensure_paths_for_adds(&mut doc, patch);
    json_patch::patch(&mut doc, patch).map_err(|e| Error::Patch(e.to_string()))?;
    // Migrate legacy formats to jsonc on write so the tree normalises.
    let target = rel_for(name, "jsonc");
    files.write_jsonc(scope, &target, &doc)?;
    if ext != "jsonc" {
        let _ = files.trash(scope, &rel);
    }
    Ok(scope)
}

/// Copied verbatim from AgentService — keeps path-tolerant `add` ops
/// working for templates that don't do a GET-then-PUT dance.
fn ensure_paths_for_adds(doc: &mut serde_json::Value, patch: &json_patch::Patch) {
    use json_patch::PatchOperation as Op;
    for op in patch.0.iter() {
        let Op::Add(add) = op else { continue };
        let path = add.path.to_string();
        if path.is_empty() { continue }
        let container_path = if path.ends_with("/-") {
            &path[..path.len() - 2]
        } else {
            path.as_str()
        };
        let segments: Vec<&str> = container_path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        if segments.is_empty() { continue }
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
        let _ = cursor;
    }
}

pub fn delete_doc(
    files: &FilesService,
    name: &str,
    explicit_scope: Option<Scope>,
) -> Result<Scope, Error> {
    validate_name(name)?;
    let (scope, rel, _ext) = match explicit_scope {
        Some(s) => {
            let (rel, ext) = find_in_scope(files, s, name)
                .ok_or_else(|| Error::NotFound(name.into()))?;
            (s, rel, ext)
        }
        None => find(files, name).ok_or_else(|| Error::NotFound(name.into()))?,
    };
    files.trash(scope, &rel)?;
    Ok(scope)
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
