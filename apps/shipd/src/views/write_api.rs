//! Write endpoints for views — template, manifest, fork.
//!
//! - `PUT /api/v1/views/{name}/template` writes a `.html.j2` file under the
//!   view's directory. Body: `{ file?, content }`. `file` defaults to
//!   `index.html.j2` and may be any relative path ending in `.html.j2`
//!   for nested routes or partials. Lint runs inline; errors block the
//!   write.
//! - `PUT /api/v1/views/{name}/manifest` writes `manifest.{jsonc|yaml|json}`.
//!   Body: `{ content, format }`. Validated against the bundled JSON
//!   schema before the file lands on disk.
//! - `POST /api/v1/views/{name}/fork` copies a view from the project
//!   (repo-shipped) scope into the user-global `<ship-dir>/views/`
//!   scope so subsequent edits don't mutate bundled files.
//!
//! All three resolve the view through the same walker the source GET
//! uses so scope semantics stay consistent.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::{Path as StdPath, PathBuf};

use crate::ship_runtime::ApiState;

const MANIFEST_SCHEMA: &str =
    include_str!("../../../../schemas/view-manifest.schema.json");

#[derive(Deserialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WriteTemplateRequest {
    /// Path relative to the view directory. Defaults to `index.html.j2`.
    /// Must end in `.html.j2` and stay inside the view dir.
    #[serde(default)]
    pub file: Option<String>,
    /// Full template body.
    pub content: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WriteManifestRequest {
    /// Manifest body (text).
    pub content: String,
    /// Manifest format: `jsonc` | `yaml` | `json`.
    pub format: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WriteResponse {
    pub ok: bool,
    pub path: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WriteDiagnostics {
    pub ok: bool,
    pub findings: Vec<LintFinding>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LintFinding {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub rule: String,
    pub severity: String,
    pub message: String,
}

fn json_err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

fn kebab_name_ok(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn relative_safe(rel: &str) -> Option<PathBuf> {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return None;
    }
    for comp in p.components() {
        use std::path::Component::*;
        match comp {
            ParentDir | RootDir | Prefix(_) => return None,
            Normal(s) if s.to_string_lossy().starts_with('.') => return None,
            _ => {}
        }
    }
    Some(p)
}

/// Locate a view by name, returning (view_dir, scope). Delegates to
/// the canonical `views::resolve::resolve_view_dir`, passing the
/// runtime's ship_dir as the global-fallback so writes resolve
/// identically to reads under the daemon's configured root.
fn resolve_view(state: &ApiState, name: &str) -> Option<(PathBuf, &'static str)> {
    super::resolve::resolve_view_dir(name, Some(state.ship_dir()))
}

#[utoipa::path(
    operation_id = "writeViewTemplate",
    put,
    path = "/api/v1/views/{name}/template",
    params(("name" = String, Path, description = "View name")),
    request_body = WriteTemplateRequest,
    responses(
        (status = 200, description = "Template written", body = WriteResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "View not found"),
        (status = 422, description = "Lint errors", body = WriteDiagnostics),
    ),
    tag = "framework"
)]
pub async fn write_view_template(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(req): Json<WriteTemplateRequest>,
) -> axum::response::Response {
    if !kebab_name_ok(&name) {
        return json_err(StatusCode::BAD_REQUEST, "invalid view name");
    }
    let Some((view_dir, _scope)) = resolve_view(&state, &name) else {
        return json_err(StatusCode::NOT_FOUND, format!("view '{name}' not found"));
    };

    let rel = req.file.as_deref().unwrap_or("index.html.j2");
    if !rel.ends_with(".html.j2") {
        return json_err(StatusCode::BAD_REQUEST, "file must end in .html.j2");
    }
    let Some(rel_path) = relative_safe(rel) else {
        return json_err(StatusCode::BAD_REQUEST, "file path must stay inside view dir");
    };
    let target = view_dir.join(&rel_path);

    if let Some(parent) = target.parent()
        && let Err(e) = std::fs::create_dir_all(parent) {
            return json_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    if let Err(e) = std::fs::write(&target, &req.content) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    (
        StatusCode::OK,
        Json(WriteResponse {
            ok: true,
            path: target.display().to_string(),
        }),
    )
        .into_response()
}

#[utoipa::path(
    operation_id = "writeViewManifest",
    put,
    path = "/api/v1/views/{name}/manifest",
    params(("name" = String, Path, description = "View name")),
    request_body = WriteManifestRequest,
    responses(
        (status = 200, description = "Manifest written", body = WriteResponse),
        (status = 400, description = "Validation error or schema failure"),
        (status = 404, description = "View not found"),
    ),
    tag = "framework"
)]
pub async fn write_view_manifest(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(req): Json<WriteManifestRequest>,
) -> axum::response::Response {
    if !kebab_name_ok(&name) {
        return json_err(StatusCode::BAD_REQUEST, "invalid view name");
    }
    let Some((view_dir, _scope)) = resolve_view(&state, &name) else {
        return json_err(StatusCode::NOT_FOUND, format!("view '{name}' not found"));
    };

    let (filename, parsed) = match req.format.as_str() {
        "jsonc" => {
            let stripped = compiler::jsonc::strip_jsonc_comments(&req.content);
            match serde_json::from_str::<serde_json::Value>(&stripped) {
                Ok(v) => ("manifest.jsonc", v),
                Err(e) => {
                    return json_err(StatusCode::BAD_REQUEST, format!("jsonc parse: {e}"))
                }
            }
        }
        "json" => match serde_json::from_str::<serde_json::Value>(&req.content) {
            Ok(v) => ("manifest.json", v),
            Err(e) => return json_err(StatusCode::BAD_REQUEST, format!("json parse: {e}")),
        },
        "yaml" | "yml" => match serde_yaml::from_str::<serde_json::Value>(&req.content) {
            Ok(v) => ("manifest.yaml", v),
            Err(e) => return json_err(StatusCode::BAD_REQUEST, format!("yaml parse: {e}")),
        },
        other => {
            return json_err(
                StatusCode::BAD_REQUEST,
                format!("unknown format '{other}'; expected jsonc|json|yaml"),
            )
        }
    };

    if let Err(msg) = validate_manifest(&parsed) {
        return json_err(StatusCode::BAD_REQUEST, format!("schema: {msg}"));
    }

    let target = view_dir.join(filename);
    if let Err(e) = std::fs::write(&target, &req.content) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    (
        StatusCode::OK,
        Json(WriteResponse {
            ok: true,
            path: target.display().to_string(),
        }),
    )
        .into_response()
}

fn validate_manifest(value: &serde_json::Value) -> Result<(), String> {
    let schema: serde_json::Value =
        serde_json::from_str(MANIFEST_SCHEMA).map_err(|e| e.to_string())?;
    let validator = jsonschema::validator_for(&schema).map_err(|e| e.to_string())?;
    let errs: Vec<String> = validator
        .iter_errors(value)
        .map(|e| format!("{}: {}", e.instance_path(), e))
        .collect();
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.join("; "))
    }
}

#[utoipa::path(
    operation_id = "forkView",
    post,
    path = "/api/v1/views/{name}/fork",
    params(("name" = String, Path, description = "View name")),
    responses(
        (status = 200, description = "View forked to user scope", body = WriteResponse),
        (status = 404, description = "View not found"),
        (status = 409, description = "View already user-scoped"),
    ),
    tag = "framework"
)]
pub async fn fork_view(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> axum::response::Response {
    if !kebab_name_ok(&name) {
        return json_err(StatusCode::BAD_REQUEST, "invalid view name");
    }
    let Some((src, scope)) = resolve_view(&state, &name) else {
        return json_err(StatusCode::NOT_FOUND, format!("view '{name}' not found"));
    };
    if scope == "global" {
        return json_err(
            StatusCode::CONFLICT,
            format!("view '{name}' is already user-scoped"),
        );
    }

    let dst_parent = state.ship_dir().join("views");
    if let Err(e) = std::fs::create_dir_all(&dst_parent) {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    let dst = dst_parent.join(&name);
    if dst.exists() {
        return json_err(StatusCode::CONFLICT, "target already exists");
    }
    if let Err(e) = copy_dir(&src, &dst) {
        let _ = std::fs::remove_dir_all(&dst);
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    (
        StatusCode::OK,
        Json(WriteResponse {
            ok: true,
            path: dst.display().to_string(),
        }),
    )
        .into_response()
}

fn copy_dir(src: &StdPath, dst: &StdPath) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
