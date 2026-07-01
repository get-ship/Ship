//! ConnectRPC wire layer for the unified `UiService`.
//!
//! `UiWire` implements `ship_ops_rpc::UiService` (12 RPCs) so shipd can mount
//! it via `UiServiceExt::register`. The unified service replaces the retired
//! `SuiService` (compiler analysis) + `ViewService` (view authoring) pair; see
//! `proto/ship/ops/v1/ui.proto` for the single-source contract.
//!
//! Four groups of handlers:
//!   * Component discovery (`packs`, `read`) — C0 reads delegated to an
//!     injected `Arc<dyn ComponentResolver>`. The resolver chain is
//!     constructed by the host (shipd) at mount time; this crate
//!     never walks a directory or composes a path from a tag.
//!   * SUI compiler analysis (`diagnostics`, `hover`, `dependencies`) — C0
//!     reads. The analyses run in-process against the `sui_compiler` analysis
//!     facade (the same Rust intelligence the LSP server uses — no bun/node).
//!   * View authoring writes (`patch_session`, `append_session`,
//!     `signal_session`, `enter_edit_mode`, `exit_edit_mode`, `tool`) — the
//!     first three are injected via typed async closures from the host;
//!     the edit-mode grants + tool call into
//!     `ship_ui::view_authoring` directly.
//!   * Scaffold (`scaffold`) — sync fs work via a closure, spawn_blocking
//!     wrapped.
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
    JsonResponse, UiQueryResponse, UiReadResponse, UiService as UiServiceTrait,
    ViewScaffoldResponse,
};

// ── Closure type aliases ──────────────────────────────────────────────────────

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;


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
/// Component discovery (`Packs`, `Read`) is delegated to an injected
/// `Arc<dyn ComponentResolver>`. The wire does NOT know where
/// components live — that's the host's convention, expressed as a
/// chain of `Root` entries (workspace, per-space views, future
/// shop / CDN). The host (shipd) constructs the resolver chain at
/// mount time from its config; the wire just calls it.
pub struct UiWire {
    resolver:     Arc<dyn crate::registry::ComponentResolver>,
    tool_fn:      ToolFn,
    scaffold_fn:  ScaffoldViewFn,
}

impl UiWire {
    pub fn new(
        resolver:    Arc<dyn crate::registry::ComponentResolver>,
        tool_fn:     ToolFn,
        scaffold_fn: ScaffoldViewFn,
    ) -> Self {
        Self {
            resolver,
            tool_fn,
            scaffold_fn,
        }
    }
}

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
// Component discovery (packs, read) lives in `crate::registry`. The
// wire delegates to the resolver chain; the wire never walks a
// bundle directory or composes a path from a tag.
// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl UiServiceTrait for UiWire {
    // ── Component discovery (C0) ──────────────────────────────────────────

    async fn packs(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedUiQueryReqView,
    ) -> ServiceResult<UiQueryResponse> {
        let packs = self
            .resolver
            .packs(crate::registry::PacksQuery {
                category:     request.category.to_string(),
                tag_contains: request.tag_contains.to_string(),
            })
            .await
            .map_err(ConnectError::internal)?;
        Ok(Response::new(UiQueryResponse {
            packs,
            __buffa_unknown_fields: Default::default(),
        }))
    }

    async fn read(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedUiReadReqView,
    ) -> ServiceResult<UiReadResponse> {
        let loc = self
            .resolver
            .read(request.path.to_string())
            .await
            .map_err(ConnectError::invalid_argument)?;
        Ok(Response::new(UiReadResponse {
            source:      loc.source,
            resolved_at: loc.origin,
            mtime:       loc.mtime,
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
        let analysis = sui_compiler::analyze(&source, Some(abs.as_path()), &[]);
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
        let analysis = sui_compiler::analyze(&source, Some(abs.as_path()), &[]);
        json_ok(json!({
            "tags": analysis.tags,
            "events": analysis.events,
            "controller": analysis.controller_path,
        }))
    }

    // ── View authoring writes (C1) ────────────────────────────────────────
    //
    // The 3 session-write RPCs (Patch/Append/Signal) were removed in B12 — the
    // ViewSessionRegistry was deleted with the UI purge, so no surface can
    // create or watch a session and these calls were always answering
    // `SessionError::NotFound`. The `ship.ui.write` writes that remain are
    // the edit-mode grants + tool + scaffold, all of which only need a
    // validated auth context, not a view-session id.

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
