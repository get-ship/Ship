//! SkillService — state layer. Pure domain logic over the file tree.
//!
//! A skill is a directory at `skills/<id>/` with:
//!  - `SKILL.md` — required. Markdown with YAML frontmatter.
//!  - arbitrary companion files (assets/, references/, evals/, ...).
//!
//! Scope model matches AgentService: project wins by default, explicit
//! `?scope=user` forces the user root. Vars are delegated to the runtime
//! `skill_vars` module (KV-backed).

use ship_platform::services::files::{Error as FileError, FilesService, Scope};
use serde::Serialize;
use std::path::{Path, PathBuf};

// ── Errors ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    AlreadyExists(String),
    BadId(String),
    BadPath(String),
    Parse(String),
    Io(String),
    ScopeUnavailable(Scope),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "skill `{s}` not found"),
            Self::AlreadyExists(s) => write!(f, "skill `{s}` already exists"),
            Self::BadId(s) => write!(f, "bad id `{s}`"),
            Self::BadPath(s) => write!(f, "bad path `{s}`"),
            Self::Parse(s) => write!(f, "parse: {s}"),
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
            FileError::Patch(s) => Self::Parse(s),
            FileError::Io(e) => Self::Io(e.to_string()),
            FileError::PathEscape(p) => Self::BadPath(p.display().to_string()),
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

// ── Types ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct IndexEntry {
    pub id: String,
    pub scope: ScopeTag,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct SkillDetail {
    pub id: String,
    pub scope: ScopeTag,
    pub frontmatter: serde_json::Value,
    pub body: String,
    pub files: Vec<String>,
}

// ── Id + path validation ──────────────────────────────────────────────

/// Validate against the schema regex: `^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$`.
pub fn validate_id(id: &str) -> Result<(), Error> {
    if id.is_empty() { return Err(Error::BadId(id.to_string())); }
    if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(Error::BadId(id.to_string()));
    }
    let first = id.chars().next().unwrap();
    let last = id.chars().last().unwrap();
    if first == '-' || last == '-' {
        return Err(Error::BadId(id.to_string()));
    }
    Ok(())
}

/// Reject absolute paths, leading slashes, and `..` segments. `FilesService`
/// re-checks too, but we want nicer errors before we reach it.
fn validate_file_path(p: &str) -> Result<(), Error> {
    if p.is_empty() || p.starts_with('/') { return Err(Error::BadPath(p.into())); }
    for seg in p.split('/') {
        if seg.is_empty() || seg == ".." || seg == "." {
            return Err(Error::BadPath(p.into()));
        }
    }
    Ok(())
}

fn skill_dir_rel(id: &str) -> PathBuf { PathBuf::from(format!("skills/{id}")) }
fn skill_md_rel(id: &str) -> PathBuf { PathBuf::from(format!("skills/{id}/SKILL.md")) }
fn skill_file_rel(id: &str, path: &str) -> PathBuf {
    PathBuf::from(format!("skills/{id}/{path}"))
}

// ── Scope resolution ──────────────────────────────────────────────────

pub fn find(files: &FilesService, id: &str) -> Option<Scope> {
    [Scope::Project, Scope::User].into_iter().find(|&scope| files.exists(scope, &skill_md_rel(id)))
}

fn resolve_scope(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
) -> Result<Scope, Error> {
    match explicit {
        Some(s) => {
            if !files.exists(s, &skill_md_rel(id)) {
                return Err(Error::NotFound(id.into()));
            }
            Ok(s)
        }
        None => find(files, id).ok_or_else(|| Error::NotFound(id.into())),
    }
}

// ── Frontmatter parser (YAML-lite — matches skills_state::parse_skill_md) ─

/// Parse SKILL.md → (frontmatter-as-object, body). Matches the existing
/// `views::state::parse_skill_md` shape: top-level scalars become strings,
/// inline lists (`[a, b]`) become arrays.
pub fn parse_skill_md(raw: &str) -> (serde_json::Value, String) {
    let mut obj = serde_json::Map::new();
    if !raw.starts_with("---") {
        return (serde_json::Value::Object(obj), raw.to_string());
    }
    let rest = &raw[3..];
    let Some(end) = rest.find("\n---") else {
        return (serde_json::Value::Object(obj), raw.to_string());
    };
    let fm = &rest[..end];
    // Body starts after the closing fence; trim the leading newline.
    let after = &rest[end + 4..];
    let body = after.trim_start_matches('\n').to_string();
    for line in fm.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim();
            obj.insert(key, parse_scalar_or_list(val));
        }
    }
    (serde_json::Value::Object(obj), body)
}

fn parse_scalar_or_list(val: &str) -> serde_json::Value {
    let t = val.trim();
    if t.starts_with('[') && t.ends_with(']') {
        let inner = &t[1..t.len() - 1];
        let arr: Vec<serde_json::Value> = inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\''))
            .filter(|s| !s.is_empty())
            .map(|s| serde_json::Value::String(s.to_string()))
            .collect();
        return serde_json::Value::Array(arr);
    }
    let unq = t.trim_matches('"').trim_matches('\'');
    serde_json::Value::String(unq.to_string())
}

/// Serialise a frontmatter object back to the `---\n…\n---` block. Scalar
/// strings and arrays only; anything else is stringified via JSON.
pub fn serialize_frontmatter(fm: &serde_json::Value) -> String {
    let mut out = String::from("---\n");
    if let Some(obj) = fm.as_object() {
        for (k, v) in obj {
            match v {
                serde_json::Value::String(s) => {
                    out.push_str(&format!("{k}: {s}\n"));
                }
                serde_json::Value::Array(items) => {
                    let parts: Vec<String> = items
                        .iter()
                        .map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string()))
                        .collect();
                    out.push_str(&format!("{k}: [{}]\n", parts.join(", ")));
                }
                other => {
                    out.push_str(&format!("{k}: {other}\n"));
                }
            }
        }
    }
    out.push_str("---\n");
    out
}

// ── List / read ────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Ok(entries) = files.list_dir(scope, Path::new("skills")) else { continue };
        for p in entries {
            if !p.is_dir() { continue; }
            let Some(id) = p.file_name().and_then(|s| s.to_str()) else { continue };
            let skill_md = p.join("SKILL.md");
            if !skill_md.is_file() { continue; }
            if !seen.insert(id.to_string()) { continue; }
            let raw = std::fs::read_to_string(&skill_md).unwrap_or_default();
            let (fm, _) = parse_skill_md(&raw);
            let name = fm
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string();
            let description = fm
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            out.push(IndexEntry {
                id: id.to_string(),
                scope: scope.into(),
                name,
                description,
            });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn load(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
) -> Result<SkillDetail, Error> {
    validate_id(id)?;
    let scope = resolve_scope(files, id, explicit)?;
    let raw = files.read_string(scope, &skill_md_rel(id))?;
    let (fm, body) = parse_skill_md(&raw);
    let dir = files.resolve(scope, &skill_dir_rel(id))?;
    let files_list = walk_rel(&dir, &dir);
    Ok(SkillDetail {
        id: id.into(),
        scope: scope.into(),
        frontmatter: fm,
        body,
        files: files_list,
    })
}

fn walk_rel(root: &Path, dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    let mut sorted: Vec<_> = entries.flatten().collect();
    sorted.sort_by_key(|e| e.file_name());
    for e in sorted {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if p.is_dir() {
            out.extend(walk_rel(root, &p));
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().to_string());
        }
    }
    out
}

// ── Writes ─────────────────────────────────────────────────────────────

pub fn write_frontmatter(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
    fm: &serde_json::Value,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = resolve_scope(files, id, explicit)?;
    let raw = files.read_string(scope, &skill_md_rel(id))?;
    let (_, body) = parse_skill_md(&raw);
    let new_raw = format!("{}{}", serialize_frontmatter(fm), body);
    files.write(scope, &skill_md_rel(id), new_raw.as_bytes())?;
    Ok(scope)
}

pub fn write_body(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
    body: &str,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = resolve_scope(files, id, explicit)?;
    let raw = files.read_string(scope, &skill_md_rel(id))?;
    let (fm, _) = parse_skill_md(&raw);
    let new_raw = format!("{}{}", serialize_frontmatter(&fm), body);
    files.write(scope, &skill_md_rel(id), new_raw.as_bytes())?;
    Ok(scope)
}

pub fn write_file(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
    path: &str,
    bytes: &[u8],
) -> Result<Scope, Error> {
    validate_id(id)?;
    validate_file_path(path)?;
    let scope = resolve_scope(files, id, explicit)?;
    files.write(scope, &skill_file_rel(id, path), bytes)?;
    Ok(scope)
}

pub fn delete_file(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
    path: &str,
) -> Result<Scope, Error> {
    validate_id(id)?;
    validate_file_path(path)?;
    if path == "SKILL.md" {
        return Err(Error::BadPath(
            "cannot delete SKILL.md — delete the skill instead".into(),
        ));
    }
    let scope = resolve_scope(files, id, explicit)?;
    let rel = skill_file_rel(id, path);
    if !files.exists(scope, &rel) {
        return Err(Error::NotFound(format!("{id}/{path}")));
    }
    files.trash(scope, &rel)?;
    Ok(scope)
}

/// Scaffold a new skill: create `skills/<id>/SKILL.md` with the provided
/// frontmatter and body. Refuses when the id already exists in the target
/// scope.
pub fn create(
    files: &FilesService,
    id: &str,
    scope: Scope,
    frontmatter: &serde_json::Value,
    body: &str,
) -> Result<(), Error> {
    validate_id(id)?;
    if files.exists(scope, &skill_md_rel(id)) {
        return Err(Error::AlreadyExists(id.into()));
    }
    let raw = format!("{}{}", serialize_frontmatter(frontmatter), body);
    files.write(scope, &skill_md_rel(id), raw.as_bytes())?;
    Ok(())
}

pub fn delete(
    files: &FilesService,
    id: &str,
    explicit: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    let scope = resolve_scope(files, id, explicit)?;
    files.trash(scope, &skill_dir_rel(id))?;
    Ok(scope)
}

pub fn rename(
    files: &FilesService,
    id: &str,
    new_id: &str,
    explicit: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    validate_id(new_id)?;
    if id == new_id { return Ok(explicit.unwrap_or(Scope::Project)); }
    let scope = resolve_scope(files, id, explicit)?;
    if files.exists(scope, &skill_md_rel(new_id)) {
        return Err(Error::AlreadyExists(new_id.into()));
    }
    // Move the entire directory by reading files one-by-one and writing
    // them at the new path. This is the only portable way through
    // FilesService — rename-across-subtree isn't exposed, and we want the
    // same atomic-write guarantees.
    copy_dir(files, scope, id, scope, new_id)?;
    files.trash(scope, &skill_dir_rel(id))?;
    Ok(scope)
}

pub fn duplicate(
    files: &FilesService,
    id: &str,
    new_id: &str,
    explicit: Option<Scope>,
) -> Result<Scope, Error> {
    validate_id(id)?;
    validate_id(new_id)?;
    let scope = resolve_scope(files, id, explicit)?;
    if files.exists(scope, &skill_md_rel(new_id)) {
        return Err(Error::AlreadyExists(new_id.into()));
    }
    copy_dir(files, scope, id, scope, new_id)?;
    Ok(scope)
}

pub fn move_scope(
    files: &FilesService,
    id: &str,
    from: Scope,
    to: Scope,
) -> Result<(), Error> {
    validate_id(id)?;
    if from == to { return Ok(()); }
    if !files.exists(from, &skill_md_rel(id)) {
        return Err(Error::NotFound(id.into()));
    }
    if files.exists(to, &skill_md_rel(id)) {
        return Err(Error::AlreadyExists(id.into()));
    }
    copy_dir(files, from, id, to, id)?;
    files.trash(from, &skill_dir_rel(id))?;
    Ok(())
}

fn copy_dir(
    files: &FilesService,
    from_scope: Scope,
    from_id: &str,
    to_scope: Scope,
    to_id: &str,
) -> Result<(), Error> {
    let root = files.resolve(from_scope, &skill_dir_rel(from_id))?;
    let rels = walk_rel(&root, &root);
    for rel in rels {
        let bytes = files.read(from_scope, &skill_file_rel(from_id, &rel))?;
        files.write(to_scope, &skill_file_rel(to_id, &rel), &bytes)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
