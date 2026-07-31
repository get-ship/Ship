//! `view.*` MCP tool dispatch domain logic.
//!
//! Relocated from `apps/shipd/src/routes/view_edit_tool_api.rs` (Batch C
//! routes-cull). All daemon-specific capabilities are injected via
//! `ViewToolPort` declared in `ship_ui::view_ports`.
//!
//! `dispatch` is the single entry point. shipd's REST handler and the
//! `ViewWire` Connect closure both funnel through it.

use std::collections::BTreeMap;

use ship_artifacts::service::ArtifactService;
use ship_substrate::gen_ulid;
use ship_types::sha256_hex;

use crate::view_authoring::{
    self, CompileError, SnapshotMeta,
    ViewCompileCheckResponse, ViewListSiblingsResponse, ViewListSnapshotsResponse,
    ViewPreviewSnapshotResponse, ViewReadSourceResponse, ViewRestoreSnapshotResponse,
    ViewThemeTokensResponse, ViewWriteSourceResponse,
};
use crate::view_ports::{BundlerBuildError, ViewToolPort};

// ── Public surface ────────────────────────────────────────────────────────────

/// Result returned by `dispatch` — callers map this to their transport response.
#[derive(Debug)]
pub enum DispatchResult {
    Ok(serde_json::Value),
    BadRequest { code: &'static str, message: String },
    NotFound { code: &'static str, message: String },
    Forbidden { code: &'static str, message: String },
    Conflict { code: &'static str, message: String },
    Internal { code: &'static str, message: String },
}

impl DispatchResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }
}

/// Entry point for all `view.*` tool operations.
///
/// Validates view name and session grant, then dispatches to the appropriate
/// op handler. `port` provides the daemon-specific capabilities.
pub async fn dispatch(
    port: &dyn ViewToolPort,
    view_name: String,
    session_id: String,
    op: String,
    args: serde_json::Value,
) -> DispatchResult {
    if !is_valid_view_name(&view_name) {
        return DispatchResult::BadRequest {
            code: "invalid_view_name",
            message: "invalid view name".into(),
        };
    }
    let session_id = session_id.trim().to_string();
    if session_id.is_empty() {
        return DispatchResult::BadRequest {
            code: "session_id_required",
            message: "session_id is required".into(),
        };
    }
    if !view_authoring::has_grant(&session_id, &view_name) {
        return DispatchResult::Forbidden {
            code: "not_authorized",
            message: "calling session is not in view-edit mode for this view".into(),
        };
    }
    match op.as_str() {
        "read_source" => op_read_source(port, &view_name),
        "write_source" => op_write_source(port, &view_name, &session_id, args).await,
        "compile_check" => op_compile_check(port, &view_name).await,
        "collections_schema" => op_collections_schema(port, args).await,
        "list_siblings" => op_list_siblings(&view_name),
        "theme_tokens" => op_theme_tokens(port),
        "preview_snapshot" => op_preview_snapshot(port, &view_name),
        "list_snapshots" => op_list_snapshots(port, &view_name, args).await,
        "restore_snapshot" => op_restore_snapshot(port, &view_name, &session_id, args).await,
        other => DispatchResult::BadRequest {
            code: "unknown_op",
            message: format!("unknown view.* op: {other}"),
        },
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

pub fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

// ── Op handlers ───────────────────────────────────────────────────────────────

fn op_read_source(port: &dyn ViewToolPort, name: &str) -> DispatchResult {
    match resolve_view_source(port, name) {
        Ok((source, _path)) => {
            let content_hash = sha256_hex(source.as_bytes());
            ok_json(&ViewReadSourceResponse { source, content_hash })
        }
        Err(e) => DispatchResult::NotFound { code: "view_not_found", message: e },
    }
}

async fn op_compile_check(port: &dyn ViewToolPort, name: &str) -> DispatchResult {
    let target = format!("views/{name}/view");
    match port.build_view(&target).await {
        Ok(_) => ok_json(&ViewCompileCheckResponse { ok: true, errors: None }),
        Err(e) => {
            let (message, file, line, col) = compile_error_from_build_error(&e, name);
            ok_json(&ViewCompileCheckResponse {
                ok: false,
                errors: Some(vec![CompileError { file, line, col, message }]),
            })
        }
    }
}

async fn op_write_source(
    port: &dyn ViewToolPort,
    name: &str,
    agent_id: &str,
    args: serde_json::Value,
) -> DispatchResult {
    #[derive(serde::Deserialize)]
    struct WriteArgs {
        source: String,
        #[serde(default)]
        turn_label: Option<String>,
    }
    let req: WriteArgs = match serde_json::from_value(args) {
        Ok(r) => r,
        Err(e) => {
            return DispatchResult::BadRequest { code: "bad_args", message: e.to_string() }
        }
    };

    let (prior_source, path) = match resolve_view_source(port, name) {
        Ok(v) => v,
        Err(e) => return DispatchResult::NotFound { code: "view_not_found", message: e },
    };

    let space_id = match port.active_space_id() {
        Some(s) if !s.is_empty() => s,
        _ => {
            return DispatchResult::Internal {
                code: "no_project",
                message: "daemon has no active space".into(),
            }
        }
    };
    let turn_id = gen_ulid();

    // 1. Write the new bytes to disk.
    if let Err(e) = std::fs::write(&path, &req.source) {
        return DispatchResult::Internal { code: "write_failed", message: e.to_string() };
    }

    // 2. Snapshot the new source — kept on success, discarded on failure.
    let snapshot_meta = match snapshot_now(
        port,
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
            let _ = std::fs::write(&path, &prior_source);
            return DispatchResult::Internal {
                code: "snapshot_failed",
                message: e.to_string(),
            };
        }
    };

    // 3. Compile-check.
    let target = format!("views/{name}/view");
    match port.build_view(&target).await {
        Ok(_) => {
            let content_hash = snapshot_meta.content_hash.clone();
            ok_json(&ViewWriteSourceResponse {
                snapshot_id: Some(snapshot_meta.snapshot_id),
                content_hash,
                compile_ok: true,
                errors: None,
            })
        }
        Err(worker_err) => {
            let _ = std::fs::write(&path, &prior_source);
            let _ = delete_snapshot_now(port, &snapshot_meta.snapshot_id).await;
            let (message, file, line, col) = compile_error_from_build_error(&worker_err, name);
            let content_hash = sha256_hex(prior_source.as_bytes());
            ok_json(&ViewWriteSourceResponse {
                snapshot_id: None,
                content_hash,
                compile_ok: false,
                errors: Some(vec![CompileError { file, line, col, message }]),
            })
        }
    }
}

async fn op_collections_schema(
    port: &dyn ViewToolPort,
    args: serde_json::Value,
) -> DispatchResult {
    #[derive(serde::Deserialize, Default)]
    struct A {
        #[serde(default)]
        id: Option<String>,
    }
    let a: A = serde_json::from_value(args).unwrap_or_default();
    let svc = match port.collections() {
        Ok(svc) => svc,
        Err(e) => {
            return DispatchResult::Conflict { code: "no_active_space", message: e.to_string() }
        }
    };
    let schemas: Vec<ship_collection::types::SchemaDoc> = match a.id {
        Some(id) => match svc.list().into_iter().find(|s| s.id == id) {
            Some(s) => vec![s],
            None => {
                return DispatchResult::NotFound {
                    code: "schema_not_found",
                    message: format!("collection '{id}' not found"),
                }
            }
        },
        None => svc.list().into_iter().filter(|s| !s.system).collect(),
    };
    ok_json(&schemas)
}

fn op_list_siblings(active_view: &str) -> DispatchResult {
    let project_root = match ship_substrate::project::get_stack_root(None).ok() {
        Some(ship_dir) => ship_dir
            .parent()
            .unwrap_or(ship_dir.as_path())
            .to_path_buf(),
        None => return ok_json(&ViewListSiblingsResponse { siblings: vec![] }),
    };
    let views_dir = project_root.join("views");
    let mut siblings = Vec::new();
    let Ok(entries) = std::fs::read_dir(&views_dir) else {
        return ok_json(&ViewListSiblingsResponse { siblings: vec![] });
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
        siblings.push(crate::view_authoring::SiblingView {
            name: name.to_string(),
            path: page.display().to_string(),
            summary: read_summary(&path.join("manifest.jsonc"))
                .or_else(|| read_summary(&path.join("manifest.json"))),
        });
    }
    ok_json(&ViewListSiblingsResponse { siblings })
}

fn op_theme_tokens(port: &dyn ViewToolPort) -> DispatchResult {
    let css = port.resolve_theme_css();
    let mut tokens: BTreeMap<String, String> = BTreeMap::new();
    for raw in css.lines() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix("--") else { continue };
        let Some((name, val)) = rest.split_once(':') else { continue };
        let value = val.trim().trim_end_matches(';').trim().to_string();
        if !value.is_empty() {
            tokens.insert(format!("--{}", name.trim()), value);
        }
    }
    ok_json(&ViewThemeTokensResponse { tokens })
}

fn op_preview_snapshot(port: &dyn ViewToolPort, name: &str) -> DispatchResult {
    match port.preview_view_html(name) {
        Ok(html) => ok_json(&ViewPreviewSnapshotResponse { html, computed_styles: None }),
        // The two error variants are observably different on the wire — a view
        // that exists but fails to render is Internal, not NotFound (the
        // pre-sever mapping, restored per the verifier's rejection).
        Err(crate::view_ports::PreviewError::ViewNotFound(e)) => {
            DispatchResult::NotFound { code: "view_not_found", message: e }
        }
        Err(crate::view_ports::PreviewError::RenderFailed(e)) => {
            DispatchResult::Internal { code: "render_failed", message: e }
        }
    }
}

async fn op_list_snapshots(
    port: &dyn ViewToolPort,
    name: &str,
    args: serde_json::Value,
) -> DispatchResult {
    #[derive(serde::Deserialize, Default)]
    struct A {
        #[serde(default)]
        limit: Option<u32>,
        #[serde(default)]
        cursor: Option<String>,
    }
    let a: A = serde_json::from_value(args).unwrap_or_default();
    let limit = a.limit.unwrap_or(50).clamp(1, 200);
    let ctx = port.space_context("global");
    match view_authoring::list_snapshots(ctx, name, limit, a.cursor.as_deref()).await {
        Ok((snaps, cursor)) => ok_json(&ViewListSnapshotsResponse { snapshots: snaps, cursor }),
        Err(e) => {
            DispatchResult::Internal { code: "list_failed", message: e.to_string() }
        }
    }
}

async fn op_restore_snapshot(
    port: &dyn ViewToolPort,
    name: &str,
    agent_id: &str,
    args: serde_json::Value,
) -> DispatchResult {
    #[derive(serde::Deserialize)]
    struct A {
        snapshot_id: String,
    }
    let a: A = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return DispatchResult::BadRequest { code: "bad_args", message: e.to_string() },
    };

    let ctx = port.space_context("global");
    let ship_dir = port.ship_dir();
    let artifacts = ArtifactService::new(ctx.clone(), ship_dir);
    let source = match view_authoring::read_snapshot(ctx, &artifacts, &a.snapshot_id).await {
        Ok(s) => s,
        Err(e) => {
            return DispatchResult::NotFound {
                code: "snapshot_not_found",
                message: e.to_string(),
            }
        }
    };

    // Delegate to the atomic write path so restore is just another write.
    let write_args = serde_json::json!({
        "source": source,
        "turn_label": format!("restore:{}", a.snapshot_id),
    });
    let write_result = op_write_source(port, name, agent_id, write_args).await;
    // Reshape write response → restore response.
    match write_result {
        DispatchResult::Ok(v) => {
            match serde_json::from_value::<ViewWriteSourceResponse>(v) {
                Ok(wr) => ok_json(&ViewRestoreSnapshotResponse {
                    content_hash: wr.content_hash,
                    compile_ok: wr.compile_ok,
                    errors: wr.errors,
                }),
                Err(e) => {
                    DispatchResult::Internal { code: "internal", message: e.to_string() }
                }
            }
        }
        other => other,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn ok_json<T: serde::Serialize>(v: &T) -> DispatchResult {
    match serde_json::to_value(v) {
        Ok(j) => DispatchResult::Ok(j),
        Err(e) => DispatchResult::Internal {
            code: "serialize_failed",
            message: e.to_string(),
        },
    }
}

/// Resolve a view by name to its (source, on-disk path). Delegates to `ViewToolPort`.
fn resolve_view_source(port: &dyn ViewToolPort, name: &str) -> Result<(String, std::path::PathBuf), String> {
    port.resolve_view_source(name)
}

fn read_summary(manifest: &std::path::Path) -> Option<String> {
    let body = std::fs::read_to_string(manifest).ok()?;
    let stripped = ship_substrate::fs_util::strip_jsonc_comments(&body);
    let v: serde_json::Value = serde_json::from_str(&stripped).ok()?;
    v.get("summary").and_then(|s| s.as_str()).map(str::to_string)
}

async fn snapshot_now(
    port: &dyn ViewToolPort,
    space_id: &str,
    view_name: &str,
    source: &str,
    turn_id: &str,
    agent_id: &str,
    label: Option<&str>,
) -> anyhow::Result<SnapshotMeta> {
    let ctx = port.space_context("global");
    let ship_dir = port.ship_dir();
    let artifacts = ArtifactService::new(ctx.clone(), ship_dir);
    view_authoring::snapshot_view(ctx, &artifacts, space_id, view_name, source, turn_id, agent_id, label).await
}

async fn delete_snapshot_now(
    port: &dyn ViewToolPort,
    snapshot_id: &str,
) -> anyhow::Result<bool> {
    let ctx = port.space_context("global");
    let ship_dir = port.ship_dir();
    let artifacts = ArtifactService::new(ctx.clone(), ship_dir);
    view_authoring::snapshot::delete_snapshot(ctx, &artifacts, snapshot_id).await
}

fn compile_error_from_build_error(
    e: &BundlerBuildError,
    view_name: &str,
) -> (String, String, u32, Option<u32>) {
    let default_file = format!("views/{view_name}/page.sui");
    match e {
        BundlerBuildError::Bundle { message, file, line, col } => {
            let f = file.clone().unwrap_or(default_file);
            let l = line.unwrap_or(1);
            (message.clone(), f, l, *col)
        }
        BundlerBuildError::UnknownTarget(t) => {
            (format!("unknown target: {t}"), default_file, 1, None)
        }
        BundlerBuildError::NoCompiler => (
            "sui-compiler unavailable — rebuild the daemon (`ship build`)".into(),
            default_file,
            1,
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_view_name_rules() {
        assert!(is_valid_view_name("my-view"));
        assert!(is_valid_view_name("view_1"));
        assert!(!is_valid_view_name(""));
        assert!(!is_valid_view_name("has space"));
        assert!(!is_valid_view_name("has/slash"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn known_ops_are_all_dispatched() {
        // Drive every op string through `dispatch` so a renamed (or deleted)
        // match arm is caught — the catch-all falls through to
        // `BadRequest { code: "unknown_op", .. }`, which is the only result
        // that signals "no arm matched".
        use crate::view_authoring::capability;
        use crate::view_ports::PreviewError;
        use serde_json::json;
        use ship_substrate::ports::SpaceContext;
        use std::path::PathBuf;
        use std::sync::Arc;

        capability::reset_for_test();
        capability::grant("sess-dispatch", "view-dispatch");

        let tmp = tempfile::tempdir().expect("tempdir");
        let ship_dir = tmp.path().to_path_buf();
        let router = ship_substrate::events::new_isolated_for_test(ship_dir.clone())
            .expect("kernel");
        let kernel = Arc::new(ship_substrate::ports::KernelLog::new(router));
        let ctx = SpaceContext::for_space(
            &ship_dir,
            ship_substrate::ports::SpaceId::new("test"),
            kernel,
        )
        .expect("space context");
        let ctx = Arc::new(ctx);
        let ship_dir = tmp.path().to_path_buf();

        struct MockPort {
            ctx: Arc<SpaceContext>,
            ship_dir: PathBuf,
        }
        #[async_trait::async_trait]
        impl ViewToolPort for MockPort {
            async fn build_view(&self, _target: &str) -> Result<(), BundlerBuildError> {
                Err(BundlerBuildError::UnknownTarget("mock".into()))
            }
            fn space_context(&self, _space: &str) -> Arc<SpaceContext> {
                self.ctx.clone()
            }
            fn ship_dir(&self) -> PathBuf {
                self.ship_dir.clone()
            }
            fn collections(&self) -> anyhow::Result<ship_collection::service::CollectionService> {
                Err(anyhow::anyhow!("mock: no active space"))
            }
            fn active_space_id(&self) -> Option<String> {
                None
            }
            fn resolve_theme_css(&self) -> String {
                String::new()
            }
            fn resolve_view_source(&self, _name: &str) -> Result<(String, PathBuf), String> {
                Err("mock: view not found".into())
            }
            fn preview_view_html(&self, _name: &str) -> Result<String, PreviewError> {
                Err(PreviewError::ViewNotFound("mock".into()))
            }
        }
        let port = MockPort { ctx, ship_dir };

        let known = [
            "read_source",
            "write_source",
            "compile_check",
            "collections_schema",
            "list_siblings",
            "theme_tokens",
            "preview_snapshot",
            "list_snapshots",
            "restore_snapshot",
        ];
        assert_eq!(known.len(), 9, "9 view.* ops expected");

        for op in known {
            let result = dispatch(
                &port,
                "view-dispatch".into(),
                "sess-dispatch".into(),
                op.into(),
                json!({}),
            )
            .await;
            if let DispatchResult::BadRequest { code: "unknown_op", message } = &result {
                panic!("op `{op}` did not match a dispatch arm: {message}");
            }
        }
    }
}
