//! POST /api/v1/lint/view — lint a view by name or inline source.
//!
//! Auth: ReadWorkspace scope (linting is read-only).

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::auth::{AuthContext, Scope, require_scope};

#[derive(Deserialize)]
pub struct LintViewRequest {
    /// Named view to lint. Daemon resolves the view directory.
    pub name: Option<String>,
    /// Raw Jinja source to lint in-flight (no filesystem write).
    pub source: Option<String>,
}

#[derive(Serialize)]
struct LintFinding {
    file: String,
    line: usize,
    col: usize,
    rule: &'static str,
    severity: &'static str,
    message: String,
}

impl From<&lint::Finding> for LintFinding {
    fn from(f: &lint::Finding) -> Self {
        Self {
            file: f.file.display().to_string(),
            line: f.line,
            col: f.col,
            rule: f.rule,
            severity: f.severity.as_str(),
            message: f.message.clone(),
        }
    }
}

fn json_err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg.into() }))).into_response()
}

/// POST /api/v1/lint/view
pub async fn lint_view(
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<LintViewRequest>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }

    let findings: Vec<lint::Finding> = if let Some(source) = req.source {
        let label = req.name.as_deref().unwrap_or("inline");
        lint::lint_view_source(label, &source)
    } else if let Some(name) = req.name {
        match crate::views::resolve_view_dir(&name) {
            Some((dir, _)) => lint::lint_view_dir(&dir),
            None => {
                return json_err(StatusCode::NOT_FOUND, format!("view '{name}' not found"));
            }
        }
    } else {
        return json_err(StatusCode::BAD_REQUEST, "provide 'name' or 'source'");
    };

    let payload: Vec<LintFinding> = findings.iter().map(LintFinding::from).collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "data": { "findings": payload } })),
    )
        .into_response()
}
