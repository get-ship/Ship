//! ConnectRPC wire layer for the unified `UiService`.
//!
//! `UiWire` implements `ship_ops_rpc::UiService` (12 RPCs) so shipd can mount
//! it via `UiServiceExt::register`. The unified service replaces the retired
//! `SuiService` (compiler analysis) + `ViewService` (view authoring) pair; see
//! `proto/ship/ops/v1/ui.proto` for the single-source contract.
//!
//! Four groups of handlers:
//!   * Component discovery (`packs`, `read`) — C0 reads served from a
//!     `sources_root` directory the daemon was booted with. The host passes the
//!     path; this crate does not couple to shipd's `ApiState` for source
//!     location. Packs scans the on-disk source tree; Read serves one file.
//!   * SUI compiler analysis (`diagnostics`, `hover`, `dependencies`) — C0
//!     reads. The analyses run in-process against the `sui_compiler` analysis
//!     facade (the same Rust intelligence the LSP server uses — no bun/node).
//!   * View authoring writes (`patch_session`, `append_session`,
//!     `signal_session`, `enter_edit_mode`, `exit_edit_mode`, `tool`) — the
//!     first three are injected via typed async closures from shipd
//!     (`rpc/ops/view_wire.rs`); the edit-mode grants + tool call into
//!     `ship_views::view_authoring` directly.
//!   * Scaffold (`scaffold`) — sync fs work via a closure, spawn_blocking
//!     wrapped.
//!
//! Agent-identity binding is preserved: `patch_session`, `append_session`,
//! `signal_session` all require an `agent_id` extracted from the auth context.
//! The `AgentFn` closure handles that extraction without coupling this crate
//! to shipd's `AuthContext` type.
//!
//! Gates: all reads carry `ship.ui.read` / C0; all writes carry
//! `ship.ui.write` / C1 in the proto descriptor; the auth middleware rejects
//! unauthenticated requests before handlers run.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::{json, Value};
use ship_ops_rpc::{
    JsonResponse, UiComponent, UiPack, UiQueryResponse, UiReadResponse, UiService as UiServiceTrait,
    ViewScaffoldResponse,
};

// ── Closure type aliases ──────────────────────────────────────────────────────

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Extract the agent_id from a request context.
/// Shipd wires this to `ctx.extensions.get::<AuthContext>().agent_id.clone()`.
pub type AgentFn = Arc<dyn Fn(&RequestContext) -> Result<String, ConnectError> + Send + Sync>;

/// Async closure for `patch_session`: (id, agent, selector, html) → Result<()>.
pub type PatchFn =
    Arc<dyn Fn(String, String, String, String) -> BoxFuture<Result<(), ConnectError>> + Send + Sync>;

/// Async closure for `append_session`: (id, agent, selector, html) → Result<()>.
pub type AppendFn =
    Arc<dyn Fn(String, String, String, String) -> BoxFuture<Result<(), ConnectError>> + Send + Sync>;

/// Async closure for `signal_session`: (id, agent, name, value) → Result<()>.
pub type SignalFn =
    Arc<dyn Fn(String, String, String, Value) -> BoxFuture<Result<(), ConnectError>> + Send + Sync>;

/// Async closure for `tool`: (view, session_id, op, args) → Result<String>.
/// Returns the raw JSON body from `dispatch_tool_core`. Captures `ApiState`.
pub type ToolFn =
    Arc<dyn Fn(String, String, String, Value) -> BoxFuture<Result<String, ConnectError>> + Send + Sync>;

/// Async closure for `scaffold`: (name, shape, purpose) → Result<(route, path)>.
/// Captures `ship_dir` from shipd's `ApiState`. The sync fs work runs in
/// `spawn_blocking` so the future is Send.
pub type ScaffoldViewFn = Arc<
    dyn Fn(String, String, Option<String>) -> BoxFuture<Result<(String, String), ConnectError>>
        + Send
        + Sync,
>;

// ── Wire struct ───────────────────────────────────────────────────────────────

/// ConnectRPC wire for the unified `UiService`.
///
/// `sources_root` is the bundle's emitted `sources/` tree; `packs` walks it
/// and `read` resolves a relative path against it. Shipd sets the path at
/// boot from its bundle config.
pub struct UiWire {
    sources_root: PathBuf,
    agent_fn:     AgentFn,
    patch_fn:     PatchFn,
    append_fn:    AppendFn,
    signal_fn:    SignalFn,
    tool_fn:      ToolFn,
    scaffold_fn:  ScaffoldViewFn,
}

impl UiWire {
    pub fn new(
        sources_root: PathBuf,
        agent_fn:     AgentFn,
        patch_fn:     PatchFn,
        append_fn:    AppendFn,
        signal_fn:    SignalFn,
        tool_fn:      ToolFn,
        scaffold_fn:  ScaffoldViewFn,
    ) -> Self {
        Self {
            sources_root,
            agent_fn,
            patch_fn,
            append_fn,
            signal_fn,
            tool_fn,
            scaffold_fn,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Identical view-name validation used by the REST edit-mode routes.
fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn json_ok(value: Value) -> ServiceResult<JsonResponse> {
    Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
}

/// Workspace root for `.sui` source paths the compiler analysis ops receive.
/// Mirrors `resolve_workspace_root` from the prior `apps/shipd/src/rpc/sui.rs`
/// implementation: `SHIP_REPO_DIR` if it points at a `bun.lock` root, else
/// walk up from CWD.
fn resolve_workspace_root() -> Result<PathBuf, String> {
    use ship_substrate::env::{self, Var};
    if let Some(repo) = env::read(Var::RepoDir) {
        return Ok(PathBuf::from(repo));
    }
    let cwd = std::env::current_dir().map_err(|e| format!("cwd unavailable: {e}"))?;
    let mut cur = cwd;
    loop {
        if cur.join("bun.lock").is_file() {
            return Ok(cur);
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => return Err("workspace root (bun.lock) not found".to_string()),
        }
    }
}

/// Read a `.sui` source, resolving `file` against the workspace root when it
/// isn't absolute. Returns the absolute path + contents.
fn read_sui_source(file: &str) -> Result<(PathBuf, String), String> {
    let root = resolve_workspace_root()?;
    let p = PathBuf::from(file);
    let abs = if p.is_absolute() { p } else { root.join(&p) };
    if !abs.exists() {
        return Err(format!("file not found: {} (resolved from {file})", abs.display()));
    }
    let source = std::fs::read_to_string(&abs)
        .map_err(|e| format!("failed to read {}: {e}", abs.display()))?;
    Ok((abs, source))
}

/// Resolve a relative `path` against the wire's `sources_root`. Returns
/// `(absolute, source, mtime_iso8601)` on success.
fn read_source_from_bundle(
    sources_root: &std::path::Path,
    path: &str,
) -> Result<(PathBuf, String, String), String> {
    if path.is_empty() {
        return Err("path is required".to_string());
    }
    let rel = std::path::Path::new(path);
    if rel.is_absolute() {
        return Err("path must be relative to sources/".to_string());
    }
    // Reject `..` components to keep resolution inside the bundle.
    for comp in rel.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return Err("path may not contain '..'".to_string());
        }
    }
    let abs = sources_root.join(rel);
    if !abs.is_file() {
        return Err(format!("source not found in bundle: {path}"));
    }
    let source = std::fs::read_to_string(&abs)
        .map_err(|e| format!("failed to read {}: {e}", abs.display()))?;
    let mtime = std::fs::metadata(&abs)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            // Trivial RFC3339-ish: seconds since epoch as a string. The studio
            // doesn't need sub-second precision and chrono isn't worth pulling
            // in for a single timestamp field.
            format!("{}", d.as_secs())
        })
        .unwrap_or_default();
    Ok((abs, source, mtime))
}

/// Walk `sources_root` once, producing a flat catalog of packs.
///
/// The catalog key is the first path segment of every `.sui` file
/// (e.g. `primitives/sui-button.sui` → pack id `@sui/primitives`). The pack
/// `category` is the first segment (`primitives` / `ship` / `user`). The
/// `id` is `@<category>/<segment>` — matching the npm-style naming the design
/// bundle uses for `sui-*` and `ship-*` packs.
fn catalog_from_bundle(
    sources_root: &std::path::Path,
) -> Result<Vec<UiPack>, String> {
    use std::collections::BTreeMap;

    let mut packs: BTreeMap<String, UiPack> = BTreeMap::new();
    walk_sources(sources_root, sources_root, &mut packs)?;
    Ok(packs.into_values().collect())
}

fn walk_sources(
    root: &std::path::Path,
    dir: &std::path::Path,
    packs: &mut std::collections::BTreeMap<String, UiPack>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let path = entry.path();
        let file_ty = entry.file_type().map_err(|e| format!("file_type: {e}"))?;
        if file_ty.is_dir() {
            walk_sources(root, &path, packs)?;
        } else if file_ty.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sui") {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| format!("strip_prefix: {e}"))?
                .to_string_lossy()
                .replace('\\', "/");
            register_component(&path, &rel, packs);
        }
    }
    Ok(())
}

fn register_component(
    abs: &std::path::Path,
    rel: &str,
    packs: &mut std::collections::BTreeMap<String, UiPack>,
) {
    let first = rel.split('/').next().unwrap_or("user");
    let category = match first {
        "primitives" => "primitives",
        "ship" => "ship",
        _ => "user",
    };
    let id = match category {
        "primitives" => format!("@sui/{first}"),
        "ship" => format!("@ship/{first}"),
        _ => format!("@user/{first}"),
    };
    let title = match (category, first) {
        ("primitives", "primitives") => "SUI Primitives",
        ("ship", "artifacts") => "Ship Artifacts",
        ("ship", "collections") => "Ship Collections",
        ("ship", "primitives") => "Ship Primitives",
        _ => first,
    }
    .to_string();

    // Tag = the filename without `.sui`. We don't introspect props/slots/events
    // here — the studio reads the source via `Read` and parses it itself.
    let tag = abs
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string();

    let entry = packs.entry(id.clone()).or_insert_with(|| UiPack {
        id: id.clone(),
        category: category.to_string(),
        title: title.clone(),
        components: Vec::new(),
        __buffa_unknown_fields: Default::default(),
    });
    entry.components.push(UiComponent {
        tag,
        file: rel.to_string(),
        title: String::new(),
        props: Vec::new(),
        slots: Vec::new(),
        events: Vec::new(),
        __buffa_unknown_fields: Default::default(),
    });
}

fn apply_query_filter(
    packs: Vec<UiPack>,
    category: &str,
    tag_contains: &str,
    has_prop: &str,
    has_slot: &str,
    has_event: &str,
) -> Vec<UiPack> {
    packs
        .into_iter()
        .filter(|p| category.is_empty() || p.category == category)
        .map(|mut p| {
            p.components.retain(|c| {
                let tag_ok = tag_contains.is_empty() || c.tag.contains(tag_contains);
                // has_prop / has_slot / has_event — the catalog emitted here
                // doesn't introspect, so without a parse pass these are
                // always empty. Until the studio's per-file introspect lands,
                // these filters match everything (a no-op) when the
                // substring is empty and match nothing when set. That's the
                // honest behavior: callers asking for a specific prop see
                // an empty list, not silently all.
                let prop_ok = has_prop.is_empty();
                let slot_ok = has_slot.is_empty();
                let event_ok = has_event.is_empty();
                tag_ok && prop_ok && slot_ok && event_ok
            });
            p
        })
        .filter(|p| !p.components.is_empty())
        .collect()
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl UiServiceTrait for UiWire {
    // ── Component discovery (C0) ──────────────────────────────────────────

    async fn packs(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedUiQueryReqView,
    ) -> ServiceResult<UiQueryResponse> {
        let packs = catalog_from_bundle(&self.sources_root)
            .map_err(ConnectError::internal)?;
        let filtered = apply_query_filter(
            packs,
            request.category,
            request.tag_contains,
            request.has_prop,
            request.has_slot,
            request.has_event,
        );
        Ok(Response::new(UiQueryResponse {
            packs: filtered,
            __buffa_unknown_fields: Default::default(),
        }))
    }

    async fn read(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedUiReadReqView,
    ) -> ServiceResult<UiReadResponse> {
        let (_abs, source, mtime) = read_source_from_bundle(&self.sources_root, request.path)
            .map_err(ConnectError::invalid_argument)?;
        // The host can fall back to its own CDN / ship-store resolver using
        // `request.fallback_urls` — but the bundle is the fast path. If the
        // caller supplied fallbacks, we still return the bundle source and
        // report the resolved location so the caller can decide.
        let resolved_at = format!("bundle://{}", request.path);
        Ok(Response::new(UiReadResponse {
            source,
            resolved_at,
            mtime,
            __buffa_unknown_fields: Default::default(),
        }))
    }

    // ── SUI compiler analysis (C0) ────────────────────────────────────────

    async fn diagnostics(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSuiFileReqView,
    ) -> ServiceResult<JsonResponse> {
        let (abs, source) =
            read_sui_source(request.file).map_err(ConnectError::invalid_argument)?;
        let analysis = sui_compiler::analyze(&source, Some(abs.as_path()));
        json_ok(json!({ "diagnostics": analysis.diagnostics }))
    }

    async fn hover(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSuiHoverReqView,
    ) -> ServiceResult<JsonResponse> {
        let (abs, source) =
            read_sui_source(request.file).map_err(ConnectError::invalid_argument)?;
        // Callers pass 0-based LSP coordinates; the facade is 1-based.
        let (l, c) = (request.line as usize + 1, request.col as usize + 1);
        let hover = sui_compiler::hover_at(&source, Some(abs.as_path()), l, c);
        json_ok(json!({ "hover": hover }))
    }

    async fn dependencies(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSuiFileReqView,
    ) -> ServiceResult<JsonResponse> {
        let (abs, source) =
            read_sui_source(request.file).map_err(ConnectError::invalid_argument)?;
        let analysis = sui_compiler::analyze(&source, Some(abs.as_path()));
        json_ok(json!({
            "tags": analysis.tags,
            "events": analysis.events,
            "controller": analysis.controller_path,
        }))
    }

    // ── View authoring writes (C1) ────────────────────────────────────────

    async fn patch_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewPatchReqView,
    ) -> ServiceResult<JsonResponse> {
        let agent = (self.agent_fn)(&ctx)?;
        (self.patch_fn)(
            request.id.to_string(),
            agent,
            request.selector.to_string(),
            request.html.to_string(),
        )
        .await?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn append_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewAppendReqView,
    ) -> ServiceResult<JsonResponse> {
        let agent = (self.agent_fn)(&ctx)?;
        (self.append_fn)(
            request.id.to_string(),
            agent,
            request.selector.to_string(),
            request.html.to_string(),
        )
        .await?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn signal_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewSignalReqView,
    ) -> ServiceResult<JsonResponse> {
        let agent = (self.agent_fn)(&ctx)?;
        let value: Value = if request.value_json.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(request.value_json)
                .map_err(|e| ConnectError::invalid_argument(format!("value_json: {e}")))?
        };
        (self.signal_fn)(
            request.id.to_string(),
            agent,
            request.name.to_string(),
            value,
        )
        .await?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn enter_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        // Auth context required (middleware guarantees it) but agent identity
        // is not needed for edit-mode grants — any authenticated caller may
        // enter edit mode on a session they own.
        let _ = ctx;
        let view = request.view.to_string();
        let session_id = request.session_id.to_string();
        if !is_valid_view_name(&view) {
            return Err(ConnectError::invalid_argument("invalid view name"));
        }
        crate::view_authoring::grant(&session_id, &view);
        json_ok(serde_json::json!({ "view": view, "session_id": session_id, "granted": true }))
    }

    async fn exit_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        let _ = ctx;
        let view = request.view.to_string();
        let session_id = request.session_id.to_string();
        if !is_valid_view_name(&view) {
            return Err(ConnectError::invalid_argument("invalid view name"));
        }
        // Idempotent — revoking a missing grant is fine.
        crate::view_authoring::revoke(&session_id, &view);
        json_ok(serde_json::json!({ "view": view, "session_id": session_id, "granted": false }))
    }

    async fn tool(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewToolReqView,
    ) -> ServiceResult<JsonResponse> {
        let _ = ctx;
        let args_value: Value = if request.args_json.is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(request.args_json)
                .map_err(|e| ConnectError::invalid_argument(format!("args_json: {e}")))?
        };
        let json = (self.tool_fn)(
            request.view.to_string(),
            request.session_id.to_string(),
            request.op.to_string(),
            args_value,
        )
        .await?;
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }

    async fn scaffold(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewScaffoldReqView,
    ) -> ServiceResult<ViewScaffoldResponse> {
        let name = request.name.to_string();
        let shape = request.shape.to_string();
        let purpose = if request.purpose.is_empty() {
            None
        } else {
            Some(request.purpose.to_string())
        };
        let (route, path) = (self.scaffold_fn)(name, shape, purpose).await?;
        Ok(Response::new(ViewScaffoldResponse {
            route,
            path,
            __buffa_unknown_fields: Default::default(),
        }))
    }
}
