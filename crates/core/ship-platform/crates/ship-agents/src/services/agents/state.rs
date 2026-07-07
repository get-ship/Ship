//! AgentsService — state layer. Pure domain logic over the agent-def tree.
//! HTTP handlers in `mod.rs` are axum glue on top.
//!
//! Scope model: project wins by default. `?scope=user` forces user scope.
//! List returns both with a `scope` annotation per entry. Writes go to
//! the scope supplied by the caller; absent caller scope, project when
//! available else user.
//!
//! Read paths (the C0 Get/List surface the `AgentsWire::list` /
//! `AgentsWire::get` handlers ride) compose through
//! `view_state::compose_space_library` — the same roll-up the index-backed
//! read and `view_state::list_agents` ride. Post format-migration Wave 5
//! there is no `.ship/agents/*.jsonc` two-scope directory scan in the
//! read path: agents only exist as `defineAgent` declare-forms in
//! `ship.config.ts`, so the roll-up IS the canonical read. Helmd's
//! `Get { id: "rust-lane" }` returning bare `id` (the W5 follow-up
//! symptom) was a `state::Error::NotFound` mapped through `rpc_wire.rs:392`
//! — that path is now composed and the message is formatted as
//! `agent \`id\``.
//!
//! Write paths (CRUD + lifecycle) are Phase-6-fenced: they continue to
//! ride the jsonc write door (`ship_config::agents_write`) since the
//! roll-up does not pick up writes. `load_doc_for_duplicate` is the
//! read-in-service-of-a-write: it stays on the jsonc door so `duplicate`
//! can clone a source def the roll-up may not reflect yet. `FilesService`
//! is used ONLY to resolve a `Scope` to its `.ship` dir — zero raw
//! `std::fs` on agent-def paths in this module.

use ship_files::files::{FilesService, Scope};
use ship_config::agents_write as door;
use ship_substrate::ports::LocalFsBroker;
use serde::Serialize;

use super::view_state::{agent_source_scope, compose_space_library, resolve_space_root};
use crate::services::errors::{ServiceEntity, ServiceError};

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

/// Marker giving [`ServiceError`] its agent-flavored Display text.
#[derive(Debug)]
pub enum AgentEntity {}

impl ServiceEntity for AgentEntity {
    const NOUN: &'static str = "agent";
    const BAD_ID_HINT: &'static str = " — must match ^[a-z0-9-]+$";
}

pub type Error = ServiceError<AgentEntity>;

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
#[derive(Debug, Serialize)]
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

// ── Read (composed roll-up) ───────────────────────────────────────
//
// The C0 Get/List read surface (`AgentsWire::list` / `AgentsWire::get`)
// resolves through the SAME composed roll-up the index-backed read +
// `view_state::list_agents` ride — the canonical, single read model post
// format-migration Wave 5. There is no `.ship/agents/*.jsonc` directory
// scan in the read path. `space_root` is `None` for the wire's production
// calls (helmd walks the project from cwd via `view_state::resolve_space_root`),
// `Some(root)` for tests.

/// Resolve the roll-up's space root: explicit override when given, else
/// the cwd-walked `view_state::resolve_space_root` (helmd's path).
fn read_space_root(space_root: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    match space_root {
        Some(r) => Some(r.to_path_buf()),
        None => resolve_space_root(),
    }
}

/// List agents visible in the composed roll-up (source + cache tier +
/// operator-enabled library tier). Project-wins precedence is already
/// applied by the roll-up; the per-id `ScopeTag` is derived from the
/// winning extension's `ExtSource` (Space → Project, anything else → User).
pub fn list(space_root: Option<&std::path::Path>) -> Vec<IndexEntry> {
    let Some(root) = read_space_root(space_root) else {
        return Vec::new();
    };
    let Some(composed) = compose_space_library(&root) else {
        return Vec::new();
    };
    let mut out: Vec<IndexEntry> = composed
        .library
        .agents
        .iter()
        .map(|profile| IndexEntry {
            id: profile.agent.id.clone(),
            scope: agent_source_scope(&composed, &profile.agent.id),
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Load the raw parsed document from the composed roll-up. `explicit_scope`
/// honors a forced scope (must match the roll-up's resolved scope) or
/// falls back to the roll-up's project-wins answer. NotFound is formatted
/// as `agent \`id\`` — the wire handler forwards it verbatim, matching the
/// canonical `view_state::load_agent` shape `rpc_wire::detail` uses.
pub fn load_doc(
    space_root: Option<&std::path::Path>,
    id: &str,
    explicit_scope: Option<Scope>,
) -> Result<(Scope, serde_json::Value), Error> {
    validate_id(id)?;
    let Some(root) = read_space_root(space_root) else {
        return Err(Error::NotFound(format!("agent `{id}`")));
    };
    let Some(composed) = compose_space_library(&root) else {
        return Err(Error::NotFound(format!("agent `{id}`")));
    };
    let Some(profile) = composed
        .library
        .agents
        .iter()
        .find(|p| p.agent.id == id)
    else {
        return Err(Error::NotFound(format!("agent `{id}`")));
    };
    let scope_tag = agent_source_scope(&composed, id);
    let scope: Scope = scope_tag.into();
    if let Some(requested) = explicit_scope
        && requested != scope
    {
        return Err(Error::NotFound(format!("agent `{id}`")));
    }
    let value = serde_json::to_value(profile)
        .map_err(|e| Error::Parse(format!("serialize agent profile: {e}")))?;
    Ok((scope, value))
}

// ── Read in service of a write (Phase-6-fenced) ───────────────────
//
// `duplicate` clones a source def before re-writing it. The roll-up
// does not reflect writes (CRUD is Phase-6-fenced), so duplicate's
// source read stays on the jsonc door — the same door `find` /
// `present_in` / `resolve_holder` (below) walk. The `_jsonc` suffix
// marks these as the legacy read path; the canonical C0 Get/List read
// is the composed `list` / `load_doc` above. The state tests exercise
// the jsonc door directly (the writes' read-after-write contract).

/// Resolve which scope holds `id`: the forced scope (must hold it) or the
/// project-wins default search. Used by `load_doc_jsonc` and by
/// `put` to pick a target scope when the caller did not.
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

/// List agents across both jsonc scopes (project-wins precedence, dedup on
/// id). The legacy directory-scan read path — used by state tests to
/// verify the jsonc write door, and by the catalog's pre-existing
/// "list_rule_files" bug (`view_state::list_rule_files` calls the wrong
/// state module — predates this work; documented in the summary). NOT
/// the C0 List read surface — that's [`list`] above.
pub fn list_jsonc(files: &FilesService) -> Vec<IndexEntry> {
    let mut out: Vec<IndexEntry> = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for scope in [Scope::Project, Scope::User] {
        let Some(dir) = scope_dir(files, scope) else { continue };
        let Ok(defs) = ship_config::agents::read_agent_defs(&fs(), dir) else {
            continue;
        };
        for d in defs {
            if !seen.insert(d.id.clone()) {
                continue;
            } // project wins
            out.push(IndexEntry { id: d.id, scope: scope.into() });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Load a source def from the jsonc door (Phase-6-fenced: write paths +
/// the reads that serve them do not touch the roll-up). Used by
/// `duplicate` and by state tests.
pub fn load_doc_jsonc(
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
    let (scope, doc) = load_doc_jsonc(files, id, explicit_scope)?;
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
    let (_scope, doc) = load_doc_jsonc(files, id, Some(from))?;
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
