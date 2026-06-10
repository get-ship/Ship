//! McpService — state layer. Pure domain logic over `mcp.jsonc`.
//!
//! Singleton-file-with-entries pattern: the file at `<scope>/mcp.jsonc`
//! holds `{ "mcp": { "servers": { <id>: {...}, ... } } }`. We address the
//! sub-objects as if they were a collection — list enumerates the keys,
//! get/put/patch/delete operate on `/mcp/servers/<id>`, and writes
//! atomically re-serialise the whole file through FilesService.
//!
//! Scope model: project wins by default. Listing returns both scopes with
//! annotations; on id collision the project entry shadows user.

use ship_files::files::{FilesService, Scope};
use serde::Serialize;
use std::path::PathBuf;

use crate::services::errors::{ServiceEntity, ServiceError};

/// Marker giving [`ServiceError`] its mcp-server-flavored Display text.
#[derive(Debug)]
pub enum McpEntity {}

impl ServiceEntity for McpEntity {
    const NOUN: &'static str = "mcp server";
}

pub type Error = ServiceError<McpEntity>;

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
    pub id: String,
    pub scope: ScopeTag,
}

// ── Id validation ───────────────────────────────────────────────────

fn validate_id(id: &str) -> Result<(), Error> {
    if id.is_empty()
        || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(Error::BadId(id.to_string()));
    }
    Ok(())
}

fn file_rel() -> PathBuf { PathBuf::from("mcp.jsonc") }

// ── Whole-file load/save ───────────────────────────────────────────

/// Load the whole `mcp.jsonc` for a scope, returning the parsed root and a
/// mutable-friendly handle into `mcp.servers`. Missing files yield an empty
/// doc so writers can seed from nothing.
fn load_root(files: &FilesService, scope: Scope) -> Result<serde_json::Value, Error> {
    let rel = file_rel();
    if !files.exists(scope, &rel) {
        return Ok(empty_root());
    }
    let v: serde_json::Value = files.read_jsonc(scope, &rel)?;
    Ok(v)
}

fn empty_root() -> serde_json::Value {
    serde_json::json!({ "mcp": { "servers": {} } })
}

fn save_root(files: &FilesService, scope: Scope, doc: &serde_json::Value) -> Result<(), Error> {
    files.write_jsonc(scope, &file_rel(), doc).map_err(Into::into)
}

/// Ensure `root.mcp.servers` is an object and return a mutable handle.
fn servers_mut(root: &mut serde_json::Value) -> Result<&mut serde_json::Map<String, serde_json::Value>, Error> {
    if !root.is_object() {
        *root = empty_root();
    }
    let obj = root.as_object_mut().unwrap();
    let mcp = obj
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    if !mcp.is_object() {
        *mcp = serde_json::Value::Object(serde_json::Map::new());
    }
    let mcp_obj = mcp.as_object_mut().unwrap();
    let servers = mcp_obj
        .entry("servers".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    if !servers.is_object() {
        *servers = serde_json::Value::Object(serde_json::Map::new());
    }
    Ok(servers.as_object_mut().unwrap())
}

fn servers_ref(root: &serde_json::Value) -> Option<&serde_json::Map<String, serde_json::Value>> {
    root.get("mcp").and_then(|m| m.get("servers")).and_then(|s| s.as_object())
}

// ── Read ───────────────────────────────────────────────────────────

pub fn list(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Ok(root) = load_root(files, scope) else { continue };
        let Some(servers) = servers_ref(&root) else { continue };
        for key in servers.keys() {
            if !seen.insert(key.clone()) { continue }
            out.push(IndexEntry { id: key.clone(), scope: scope.into() });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn find(files: &FilesService, id: &str) -> Option<Scope> {
    for scope in [Scope::Project, Scope::User] {
        let Ok(root) = load_root(files, scope) else { continue };
        if servers_ref(&root).and_then(|s| s.get(id)).is_some() {
            return Some(scope);
        }
    }
    None
}

fn exists_in_scope(files: &FilesService, scope: Scope, id: &str) -> bool {
    load_root(files, scope).ok()
        .as_ref()
        .and_then(servers_ref)
        .and_then(|s| s.get(id))
        .is_some()
}

/// Load a single server entry. Returns (scope, entry-value).
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
    let entry = servers_ref(&root)
        .and_then(|s| s.get(id))
        .cloned()
        .ok_or_else(|| Error::NotFound(id.into()))?;
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
    let mut root = load_root(files, scope)?;
    let servers = servers_mut(&mut root)?;
    if !allow_overwrite && servers.contains_key(id) {
        return Err(Error::AlreadyExists(id.into()));
    }
    servers.insert(id.to_string(), entry.clone());
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
    // Extract the entry, patch it in place, then re-insert.
    let mut entry = servers_ref(&root)
        .and_then(|s| s.get(id))
        .cloned()
        .ok_or_else(|| Error::NotFound(id.into()))?;
    ship_files::path_tolerant::ensure_paths_for_adds(&mut entry, patch);
    json_patch::patch(&mut entry, patch).map_err(|e| Error::Patch(e.to_string()))?;
    let servers = servers_mut(&mut root)?;
    servers.insert(id.to_string(), entry);
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
    let servers = servers_mut(&mut root)?;
    if servers.remove(id).is_none() {
        return Err(Error::NotFound(id.into()));
    }
    save_root(files, scope, &root)?;
    Ok(scope)
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
