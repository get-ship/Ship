//! `SuiService` handler — SUI language intelligence on the daemon (#141).
//!
//! Diagnostics / hover / dependency analysis for `.sui` templates, answered
//! in-process by the `sui_compiler` analysis facade (the same Rust intelligence
//! the LSP server uses — no bun/node). Promoted off the MCP `sui_*` tools so the
//! capability survives the MCP crate's removal. All C0 reads, no gate.

use std::path::{Path, PathBuf};

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;

use ship_ops_rpc::{JsonResponse, SuiService};

use crate::rpc::ops::OpsImpl;

/// Workspace root that workspace-relative file args resolve against:
/// `SHIP_REPO_DIR` if it points at a `bun.lock` root, else walk up from CWD.
fn resolve_workspace_root() -> Result<PathBuf, String> {
    use ship_substrate::env::{self, Var};
    if let Some(repo) = env::read(Var::RepoDir) {
        let root = PathBuf::from(repo);
        if root.join("bun.lock").is_file() {
            return Ok(root);
        }
    }
    let cwd = std::env::current_dir().map_err(|e| format!("cwd unavailable: {e}"))?;
    let mut cur = cwd;
    loop {
        if cur.join("bun.lock").is_file() {
            return Ok(cur);
        }
        if !cur.pop() {
            return Err(
                "workspace root not found; set SHIP_REPO_DIR or run inside the repo".to_string(),
            );
        }
    }
}

/// Read a `.sui` source, resolving `file` against the workspace root when it
/// isn't absolute. Returns the absolute path + contents.
fn read_source(file: &str) -> Result<(PathBuf, String), String> {
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

fn ok_json(value: serde_json::Value) -> ServiceResult<JsonResponse> {
    Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
}

#[allow(refining_impl_trait)]
impl SuiService for OpsImpl {
    async fn diagnostics(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSuiFileReqView,
    ) -> ServiceResult<JsonResponse> {
        let (abs, source) =
            read_source(request.file).map_err(ConnectError::invalid_argument)?;
        let analysis = sui_compiler::analyze(&source, Some(abs.as_path() as &Path));
        ok_json(json!({ "diagnostics": analysis.diagnostics }))
    }

    async fn hover(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSuiHoverReqView,
    ) -> ServiceResult<JsonResponse> {
        let (abs, source) =
            read_source(request.file).map_err(ConnectError::invalid_argument)?;
        // Callers pass 0-based LSP coordinates; the facade is 1-based.
        let (l, c) = (request.line as usize + 1, request.col as usize + 1);
        let hover = sui_compiler::hover_at(&source, Some(abs.as_path() as &Path), l, c);
        ok_json(json!({ "hover": hover }))
    }

    async fn dependencies(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSuiFileReqView,
    ) -> ServiceResult<JsonResponse> {
        let (abs, source) =
            read_source(request.file).map_err(ConnectError::invalid_argument)?;
        let analysis = sui_compiler::analyze(&source, Some(abs.as_path() as &Path));
        ok_json(json!({
            "tags": analysis.tags,
            "events": analysis.events,
            "controller": analysis.controller_path,
        }))
    }
}
