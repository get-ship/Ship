//! Service registry — the single entry point for calling domain
//! services by name. The template SDK (`ship.*` minijinja globals),
//! the forthcoming MCP tool surface, and the TS SDK codegen all
//! resolve through this module.
//!
//! Registered names read like `<domain>.<op>` — `agents.list`,
//! `git.diff`, `permissions.tool_patterns`. Args are a JSON object;
//! unknown keys ignored, required-missing raise `BadArgs`.
//!
//! Read ops only. Writes are exposed as HTTP endpoints (POST/PATCH/
//! PUT/DELETE on each service), not registry entries — keeps the
//! security boundary between "reading" and "mutating" explicit.
//!
//! The registry is synchronous. Every domain state function we call
//! is sync (file I/O via FilesService, libgit2 direct, DB queries).
//! That lets us expose `call` to minijinja without a blocking bridge.

use serde_json::Value;

use crate::ship_runtime::ApiState;
use ship_platform::services::files::{FilesService, Scope};

/// Structured error returned from a registry call. Rendered verbatim
/// into the template as an `{error, message}` object (per spec §11.3
/// — structured errors, template decides how to render).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryError {
    pub code: &'static str,
    pub message: String,
}

impl RegistryError {
    fn bad_args(msg: impl Into<String>) -> Self {
        Self { code: "bad_args", message: msg.into() }
    }
    fn unknown_service(name: &str) -> Self {
        Self { code: "unknown_service", message: format!("unknown service `{name}`") }
    }
    fn service(msg: impl Into<String>) -> Self {
        Self { code: "service_error", message: msg.into() }
    }
    fn not_found(msg: impl Into<String>) -> Self {
        Self { code: "not_found", message: msg.into() }
    }
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
impl std::error::Error for RegistryError {}

/// Minimal HTML entity escape for interpolated user content in views.
/// The user-view env runs AutoEscape::None (needed for Datastar attrs),
/// so untrusted content (file bodies) must be pre-escaped at the
/// service boundary to prevent `<script>`/`<style>`/`<link>` injection.
pub(super) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

// ── Args helpers ─────────────────────────────────────────────────────

pub(super) fn parse_scope(args: &Value) -> Result<Option<Scope>, RegistryError> {
    match args.get("scope").and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(RegistryError::bad_args(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

pub(super) fn required_str<'a>(args: &'a Value, field: &str) -> Result<&'a str, RegistryError> {
    args.get(field)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| RegistryError::bad_args(format!("missing required arg `{field}`")))
}

pub(super) fn optional_usize(args: &Value, field: &str, default: usize) -> usize {
    args.get(field)
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(default)
}


#[macro_use]
mod op_macro;
pub mod wire;
// Behind `fixtures` only (not `test`): the daemon-side fixtures reuse
// ship-platform `example()` builders, which need `ship-platform/fixtures`
// — a plain `cargo test -p shipd` doesn't enable that cross-crate feature.
#[cfg(feature = "fixtures")]
pub mod fixtures;

pub mod agents;
pub mod sessions;
pub mod actors;
pub mod mesh;
pub mod diagnostics;

// ── Dispatch ─────────────────────────────────────────────────────────

/// Dispatch a registered service call. Synchronous — see module docs.
///
/// Each domain submodule has a `try_dispatch` returning
/// `Result<Option<Value>, RegistryError>`:
///   - Ok(Some(v)) — handled successfully
///   - Ok(None) — name doesn't belong to this domain; try the next
///   - Err(e) — name belongs but the call errored; return verbatim
pub fn call(name: &str, args: &Value, state: &ApiState) -> Result<Value, RegistryError> {
    let files = FilesService::resolve_roots(state.ship_dir());
    if let Some(value) = agents::try_dispatch(name, args, state, &files)? {
        return Ok(value);
    }
    if let Some(value) = sessions::try_dispatch(name, args, state, &files)? {
        return Ok(value);
    }
    if let Some(value) = actors::try_dispatch(name, args, state, &files)? {
        return Ok(value);
    }
    if let Some(value) = mesh::try_dispatch(name, args, state, &files)? {
        return Ok(value);
    }
    if let Some(value) = diagnostics::try_dispatch(name, args, state, &files)? {
        return Ok(value);
    }
    Err(RegistryError::unknown_service(name))
}

/// Enumerate registered service names — derived from the rich op
/// descriptors in `services::schema`. Keep `services::registry::call`'s
/// dispatch chain in sync with `schema::OPS`; drift is visible in diff.
pub fn names() -> Vec<&'static str> {
    super::schema::OPS.iter().map(|op| op.name).collect()
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
