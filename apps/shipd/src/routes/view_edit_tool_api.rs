//! REST handler for the `view.*` MCP tool family.
//!
//! `POST /api/v1/views/{name}/edit/tool`
//!   body: { session_id, op, args }
//!
//! The MCP server in `apps/mcp` calls this single endpoint for every
//! `view.*` tool — the dispatch keeps capability enforcement, kernel /
//! data plumbing, and snapshot atomicity in one place rather than scattered
//! across per-tool routes. Frontends never hit this endpoint; only enter /
//! exit are user-facing.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;
use ship_artifacts::service::ArtifactService;
use ship_platform::view_authoring::{self, CompileError, ViewAnnotateResponse, ViewCompileCheckResponse,
    ViewListSnapshotsResponse, ViewPreviewSnapshotResponse, ViewReadSourceResponse,
    ViewRestoreSnapshotResponse, ViewWriteSourceResponse};

use super::view_edit_mode_api::is_valid_view_name_pub as is_valid_view_name;
use crate::views::sui_view_edit_sse::{
    AnnotationClearPayload, AnnotationPayload, publish_annotation, publish_annotation_clear,
};

/// Auto-clear delay (annotation pins) after a successful `view.write_source`.
const ANNOTATION_CLEAR_DELAY_SECS: u64 = 5;

#[derive(serde::Deserialize)]
pub struct ToolBody {
    pub session_id: String,
    pub op: String,
    #[serde(default = "default_args")]
    pub args: serde_json::Value,
}

fn default_args() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

fn err(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "ok": false,
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

fn ok<T: serde::Serialize>(value: &T) -> Response {
    match serde_json::to_value(value) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialize_failed",
            &e.to_string(),
        ),
    }
}

pub async fn dispatch_tool(
    Path(name): Path<String>,
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<ToolBody>,
) -> Response {
    dispatch_tool_core(&state, name, body).await
}

/// Shared core for the REST handler and the Connect command (#78 will give it a
/// typed result + move it into a service).
pub(crate) async fn dispatch_tool_core(state: &ApiState, name: String, body: ToolBody) -> Response {
    if !is_valid_view_name(&name) {
        return err(StatusCode::BAD_REQUEST, "invalid_view_name", "invalid view name");
    }
    let session_id = body.session_id.trim();
    if session_id.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "session_id_required",
            "session_id is required",
        );
    }
    if !view_authoring::has_grant(session_id, &name) {
        return err(
            StatusCode::FORBIDDEN,
            "not_authorized",
            "calling session is not in view-edit mode for this view",
        );
    }
    let agent_id = session_id.to_string();
    match body.op.as_str() {
        "read_source" => op_read_source(&name).await,
        "write_source" => op_write_source(state, &name, &agent_id, body.args).await,
        "compile_check" => op_compile_check(state, &name).await,
        "collections_schema" => op_collections_schema(state, body.args).await,
        "list_siblings" => op_list_siblings(&name).await,
        "theme_tokens" => op_theme_tokens().await,
        "preview_snapshot" => op_preview_snapshot(&name).await,
        "annotate" => op_annotate(&name, &agent_id, body.args).await,
        "list_snapshots" => op_list_snapshots(state, &name, body.args).await,
        "restore_snapshot" => op_restore_snapshot(state, &name, &agent_id, body.args).await,
        other => err(
            StatusCode::BAD_REQUEST,
            "unknown_op",
            &format!("unknown view.* op: {other}"),
        ),
    }
}

// ── op handlers ──────────────────────────────────────────────────────────────

async fn op_read_source(name: &str) -> Response {
    let (source, _path) = match resolve_view(name) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let content_hash = ship_types::sha256_hex(source.as_bytes());
    ok(&ViewReadSourceResponse { source, content_hash })
}

async fn op_compile_check(state: &ApiState, name: &str) -> Response {
    let target = format!("views/{name}/view");
    match state.bundler().build(&target).await {
        Ok(_) => ok(&ViewCompileCheckResponse {
            ok: true,
            errors: None,
        }),
        Err(e) => {
            let (message, file, line, col) = compile_error_from_worker(&e, name);
            ok(&ViewCompileCheckResponse {
                ok: false,
                errors: Some(vec![CompileError { file, line, col, message }]),
            })
        }
    }
}

async fn op_write_source(
    state: &ApiState,
    name: &str,
    agent_id: &str,
    args: serde_json::Value,
) -> Response {
    let req: WriteArgs = match serde_json::from_value(args) {
        Ok(r) => r,
        Err(e) => return err(StatusCode::BAD_REQUEST, "bad_args", &e.to_string()),
    };
    let (prior_source, path) = match resolve_view(name) {
        Ok(v) => v,
        Err(e) => return e,
    };

    // Take the new-source snapshot upfront so we can persist on success
    // or discard on failure.
    let space_id = state.project_slug().unwrap_or_default();
    if space_id.is_empty() {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "no_project",
            "daemon has no active project",
        );
    }
    let turn_id = ship_substrate::gen_ulid();

    // 1. Write the new bytes to disk.
    if let Err(e) = std::fs::write(&path, &req.source) {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "write_failed",
            &e.to_string(),
        );
    }

    // 2. Snapshot the new (post-write) source — we'll keep it on
    // compile-success, delete on compile-failure.
    let snapshot_meta = match snapshot_now(
        state,
        &space_id,
        name,
        &req.source,
        &turn_id,
        agent_id,
        req.turn_label.as_deref(),
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            // Roll back the disk write so the failure leaves the source
            // untouched.
            let _ = std::fs::write(&path, &prior_source);
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "snapshot_failed",
                &e.to_string(),
            );
        }
    };

    // 3. Compile-check against the on-disk source.
    let target = format!("views/{name}/view");
    let compile_result = state.bundler().build(&target).await;
    match compile_result {
        Ok(_) => {
            let content_hash = snapshot_meta.content_hash.clone();
            ok(&ViewWriteSourceResponse {
                snapshot_id: Some(snapshot_meta.snapshot_id),
                content_hash,
                compile_ok: true,
                errors: None,
            })
        }
        Err(worker_err) => {
            // Revert: rewrite prior source + delete the just-taken snapshot.
            let _ = std::fs::write(&path, &prior_source);
            let _ = delete_snapshot_now(state, &snapshot_meta.snapshot_id).await;
            let (message, file, line, col) = compile_error_from_worker(&worker_err, name);
            let content_hash =
                ship_types::sha256_hex(prior_source.as_bytes());
            ok(&ViewWriteSourceResponse {
                snapshot_id: None,
                content_hash,
                compile_ok: false,
                errors: Some(vec![CompileError { file, line, col, message }]),
            })
        }
    }
}

async fn op_collections_schema(state: &ApiState, args: serde_json::Value) -> Response {
    #[derive(serde::Deserialize, Default)]
    struct A {
        #[serde(default)]
        id: Option<String>,
    }
    let a: A = serde_json::from_value(args).unwrap_or_default();
    let svc = match state.collections() {
        Ok(svc) => svc,
        Err(e) => return err(StatusCode::CONFLICT, "no_active_space", &e.to_string()),
    };
    let schemas: Vec<ship_collection::types::SchemaDoc> = match a.id {
        Some(id) => match svc.list().into_iter().find(|s| s.id == id) {
            Some(s) => vec![s],
            None => {
                return err(StatusCode::NOT_FOUND, "schema_not_found", &format!("collection '{id}' not found"));
            }
        },
        None => svc.list().into_iter().filter(|s| !s.system).collect(),
    };
    ok(&schemas)
}

async fn op_list_siblings(active_view: &str) -> Response {
    let project_root = match project_root() {
        Some(p) => p,
        None => return ok(&view_authoring::ViewListSiblingsResponse { siblings: vec![] }),
    };
    let views_dir = project_root.join("views");
    let mut siblings: Vec<view_authoring::SiblingView> = Vec::new();
    let entries = match std::fs::read_dir(&views_dir) {
        Ok(e) => e,
        Err(_) => return ok(&view_authoring::ViewListSiblingsResponse { siblings: vec![] }),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else { continue };
        if name == active_view || name.starts_with('.') {
            continue;
        }
        let page = path.join("page.sui");
        if !page.is_file() {
            continue;
        }
        siblings.push(view_authoring::SiblingView {
            name: name.to_string(),
            path: page.display().to_string(),
            summary: read_summary(&path.join("manifest.jsonc"))
                .or_else(|| read_summary(&path.join("manifest.json"))),
        });
    }
    ok(&view_authoring::ViewListSiblingsResponse { siblings })
}

fn read_summary(manifest: &std::path::Path) -> Option<String> {
    let body = std::fs::read_to_string(manifest).ok()?;
    let stripped = ship_substrate::fs_util::strip_jsonc_comments(&body);
    let v: serde_json::Value = serde_json::from_str(&stripped).ok()?;
    v.get("summary")
        .and_then(|s| s.as_str())
        .map(str::to_string)
}

async fn op_theme_tokens() -> Response {
    use std::collections::BTreeMap;
    let css = crate::views::themes::resolve_theme_css();
    let mut tokens: BTreeMap<String, String> = BTreeMap::new();
    // Parse `--name: value;` declarations out of any rule block. We don't
    // care about selector scoping — `view.theme_tokens` is "what does the
    // agent see today," not "which selector wins."
    for raw in css.lines() {
        let line = raw.trim();
        let rest = match line.strip_prefix("--") {
            Some(r) => r,
            None => continue,
        };
        let Some((name, val)) = rest.split_once(':') else { continue };
        let value = val.trim().trim_end_matches(';').trim().to_string();
        if value.is_empty() {
            continue;
        }
        tokens.insert(format!("--{}", name.trim()), value);
    }
    ok(&view_authoring::ViewThemeTokensResponse { tokens })
}

async fn op_preview_snapshot(name: &str) -> Response {
    let resolved = match deck::resolver::resolve(name, None) {
        Ok(r) => r,
        Err(e) => return err(StatusCode::NOT_FOUND, "view_not_found", &e.message),
    };
    match deck::resolver::render_resolved(&resolved, name) {
        Ok(rendered) => ok(&ViewPreviewSnapshotResponse {
            html: rendered.html,
            computed_styles: None,
        }),
        Err(msg) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "render_failed",
            &msg,
        ),
    }
}

async fn op_annotate(name: &str, agent_id: &str, args: serde_json::Value) -> Response {
    #[derive(serde::Deserialize)]
    struct A {
        selector: String,
        note: String,
        #[serde(default)]
        kind: Option<view_authoring::AnnotationKind>,
        #[serde(default)]
        turn_id: Option<String>,
    }
    let a: A = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return err(StatusCode::BAD_REQUEST, "bad_args", &e.to_string()),
    };
    let annotation_id = ship_substrate::gen_ulid();
    let payload = AnnotationPayload {
        annotation_id: annotation_id.clone(),
        view_name: name.to_string(),
        selector: a.selector,
        note: a.note,
        kind: a
            .kind
            .map(|k| k.as_str().to_string())
            .unwrap_or_else(|| "info".to_string()),
        turn_id: a.turn_id.unwrap_or_else(|| agent_id.to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    publish_annotation(name, &payload).await;
    ok(&ViewAnnotateResponse { annotation_id })
}

async fn op_list_snapshots(state: &ApiState, name: &str, args: serde_json::Value) -> Response {
    #[derive(serde::Deserialize, Default)]
    struct A {
        #[serde(default)]
        limit: Option<u32>,
        #[serde(default)]
        cursor: Option<String>,
    }
    let a: A = serde_json::from_value(args).unwrap_or_default();
    let limit = a.limit.unwrap_or(50).clamp(1, 200);
    let ctx = view_authoring::space_context_for(
        state.kernel().clone(),
        state.ship_dir(),
        "global",
    );
    match view_authoring::list_snapshots(ctx, name, limit, a.cursor.as_deref()).await {
        Ok((snaps, cursor)) => ok(&ViewListSnapshotsResponse {
            snapshots: snaps,
            cursor,
        }),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "list_failed",
            &e.to_string(),
        ),
    }
}

async fn op_restore_snapshot(
    state: &ApiState,
    name: &str,
    agent_id: &str,
    args: serde_json::Value,
) -> Response {
    #[derive(serde::Deserialize)]
    struct A {
        snapshot_id: String,
    }
    let a: A = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return err(StatusCode::BAD_REQUEST, "bad_args", &e.to_string()),
    };
    // Read snapshot bytes.
    let source = {
        let ctx = view_authoring::space_context_for(
            state.kernel().clone(),
            state.ship_dir(),
            "global",
        );
        let artifacts = ArtifactService::new(ctx.clone(), state.ship_dir().to_path_buf());
        match view_authoring::read_snapshot(ctx, &artifacts, &a.snapshot_id).await {
            Ok(s) => s,
            Err(e) => {
                return err(StatusCode::NOT_FOUND, "snapshot_not_found", &e.to_string());
            }
        }
    };
    // Delegate to the atomic write path so restore is just another
    // snapshot-taking write.
    let args = serde_json::json!({
        "source": source,
        "turn_label": format!("restore:{}", a.snapshot_id),
    });
    let resp = op_write_source(state, name, agent_id, args).await;
    // The contract response for restore_snapshot is a subset of the write
    // response; reshape rather than send back the wrong type.
    let body_bytes = match axum::body::to_bytes(resp.into_body(), 1_048_576).await {
        Ok(b) => b,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                &e.to_string(),
            );
        }
    };
    let write_resp: ViewWriteSourceResponse = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(_) => {
            // Already an error envelope — pass it through unchanged.
            return (StatusCode::OK, body_bytes).into_response();
        }
    };
    ok(&ViewRestoreSnapshotResponse {
        content_hash: write_resp.content_hash,
        compile_ok: write_resp.compile_ok,
        errors: write_resp.errors,
    })
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn project_root() -> Option<std::path::PathBuf> {
    let ship_dir = ship_substrate::project::get_project_dir(None).ok()?;
    Some(ship_dir.parent().unwrap_or(&ship_dir).to_path_buf())
}

#[allow(
    clippy::result_large_err,
    reason = "axum::Response is the canonical error vehicle for handler helpers"
)]
fn resolve_view(name: &str) -> Result<(String, std::path::PathBuf), Response> {
    let resolved = deck::resolver::resolve(name, None)
        .map_err(|e| err(StatusCode::NOT_FOUND, "view_not_found", &e.message))?;
    let path = resolved
        .source_path
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "view_not_found", "view has no on-disk source"))?;
    Ok((resolved.source, path))
}

async fn snapshot_now(
    state: &ApiState,
    space_id: &str,
    view_name: &str,
    source: &str,
    turn_id: &str,
    agent_id: &str,
    label: Option<&str>,
) -> anyhow::Result<view_authoring::SnapshotMeta> {
    let ctx = view_authoring::space_context_for(
        state.kernel().clone(),
        state.ship_dir(),
        "global",
    );
    let artifacts = ArtifactService::new(ctx.clone(), state.ship_dir().to_path_buf());
    view_authoring::snapshot_view(
        ctx,
        &artifacts,
        space_id,
        view_name,
        source,
        turn_id,
        agent_id,
        label,
    )
    .await
}

async fn delete_snapshot_now(state: &ApiState, snapshot_id: &str) -> anyhow::Result<bool> {
    let ctx = view_authoring::space_context_for(
        state.kernel().clone(),
        state.ship_dir(),
        "global",
    );
    let artifacts = ArtifactService::new(ctx.clone(), state.ship_dir().to_path_buf());
    view_authoring::snapshot::delete_snapshot(ctx, &artifacts, snapshot_id).await
}

#[derive(serde::Deserialize)]
struct WriteArgs {
    source: String,
    #[serde(default)]
    turn_label: Option<String>,
}

fn compile_error_from_worker(
    err: &deck::bundler::BundlerError,
    view_name: &str,
) -> (String, String, u32, Option<u32>) {
    use deck::bundler::BundlerError;
    let default_file = format!("views/{view_name}/page.sui");
    match err {
        BundlerError::Bundle { message, .. } => (message.clone(), default_file, 1, None),
        BundlerError::UnknownTarget(t) => (format!("unknown target: {t}"), default_file, 1, None),
        BundlerError::NoCompiler => (
            "sui-compiler not found; run `ship runtime install`".into(),
            default_file,
            1,
            None,
        ),
    }
}

// Schedule a deferred annotation-clear broadcast for a view. Called by the
// write_source path after a successful compile + snapshot — agent UI clears
// its annotation pins so they don't linger past the edit they referenced.
//
// Kept public so the bundler watcher path (apps/shipd/src/lib.rs) can fire
// the same clear when the file changes via watcher rather than write_source.
pub fn schedule_annotation_clear(view_name: String) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(ANNOTATION_CLEAR_DELAY_SECS))
            .await;
        let _ = publish_annotation_clear(
            &view_name,
            &AnnotationClearPayload {
                annotation_id: None,
                view_name: view_name.clone(),
            },
        )
        .await;
    });
}

#[cfg(test)]
#[path = "view_edit_tool_api_tests.rs"]
mod tests;

// Re-export so the route handler module can call this without an extra
// `Arc` clone chain through state.
#[allow(dead_code)]
fn _state_arc(state: &ApiState) -> Arc<crate::ship_runtime::ShipRuntime> {
    state.clone()
}
