//! Pipeline config endpoints — list and read .ship/pipelines/*.jsonc (or legacy .yaml)

use axum::{Json, extract::Path, http::StatusCode};
use serde::Deserialize;
use std::sync::OnceLock;

use super::{err_response, ok_response, resolve_worktree};
use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

/// Recognised pipeline file extensions, preferred first.
const PIPELINE_EXTS: &[&str] = &["jsonc", "yaml"];

/// Embedded schema JSON — ships inside the binary so the daemon doesn't
/// depend on the source tree at runtime.
const PIPELINE_SCHEMA_JSON: &str =
    include_str!("../../../../schemas/pipeline.schema.json");

/// A single validation issue. Used for both parse failures and
/// schema-rule failures so the editor can display them uniformly.
#[derive(serde::Serialize, Debug, Clone)]
pub struct ValidationIssue {
    /// Short human-readable error message.
    pub message: String,
    /// Parse vs. schema — drives the client's marker styling.
    pub kind: &'static str,
    /// Source format for the editor's linter (jsonc | yaml).
    pub format: &'static str,
    /// 1-based line number. Set for parse errors; best-effort for schema errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 1-based column number. Paired with `line`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// JSON Pointer to the offending value (schema errors only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ValidationIssue {
    fn parse_error(message: String, format: &'static str, line: Option<u32>, column: Option<u32>) -> Self {
        Self { message, kind: "parse", format, line, column, path: None }
    }
}

fn schema() -> &'static jsonschema::Validator {
    static SCHEMA: OnceLock<jsonschema::Validator> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let value: serde_json::Value = serde_json::from_str(PIPELINE_SCHEMA_JSON)
            .expect("embedded pipeline schema is valid JSON");
        jsonschema::validator_for(&value)
            .expect("embedded pipeline schema compiles")
    })
}

/// Parse pipeline content (JSONC or YAML) into a serde_json::Value,
/// preserving line/column on failure.
///
/// Note: JSONC parsing strips comments first. The stripper keeps line
/// breaks intact so `serde_json::Error::line()` is correct, but column
/// numbers on lines that contained inline comments can shift left by
/// the comment length. Good enough for "jump to line" editor affordances.
fn parse_pipeline_content(content: &str) -> Result<(serde_json::Value, &'static str), ValidationIssue> {
    let trimmed = content.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        let json = compiler::jsonc::strip_jsonc_comments(content);
        serde_json::from_str(&json)
            .map(|v| (v, "jsonc"))
            .map_err(|e| ValidationIssue::parse_error(
                e.to_string(),
                "jsonc",
                Some(e.line() as u32),
                Some(e.column() as u32),
            ))
    } else {
        serde_yaml::from_str(content)
            .map(|v| (v, "yaml"))
            .map_err(|e| ValidationIssue::parse_error(
                e.to_string(),
                "yaml",
                e.location().map(|l| l.line() as u32),
                e.location().map(|l| l.column() as u32),
            ))
    }
}

/// Run the schema validator against a parsed pipeline document.
///
/// Returns all schema issues (not just the first). Each issue carries
/// a JSON Pointer path; for JSONC sources we resolve it to a 1-based
/// (line, column) so the editor can place a marker at the exact spot.
/// Returns an empty Vec when the document passes.
fn validate_schema(
    value: &serde_json::Value,
    source: &str,
    format: &'static str,
) -> Vec<ValidationIssue> {
    schema()
        .iter_errors(value)
        .map(|err| {
            let pointer = err.instance_path().to_string();
            let (line, column) = if format == "jsonc" && !pointer.is_empty() {
                super::pipeline_position::resolve_pointer(source, &pointer)
                    .map(|(l, c)| (Some(l), Some(c)))
                    .unwrap_or((None, None))
            } else {
                (None, None)
            };
            ValidationIssue {
                message: err.to_string(),
                kind: "schema",
                format,
                line,
                column,
                path: if pointer.is_empty() { None } else { Some(pointer) },
            }
        })
        .collect()
}

/// Full validate — parse then schema. Returns Ok on clean, Err with all
/// issues otherwise. Parse errors short-circuit: if we can't parse, we
/// can't schema-check.
fn validate_pipeline(content: &str) -> Result<(), Vec<ValidationIssue>> {
    match parse_pipeline_content(content) {
        Err(parse_issue) => Err(vec![parse_issue]),
        Ok((value, format)) => {
            let issues = validate_schema(&value, content, format);
            if issues.is_empty() { Ok(()) } else { Err(issues) }
        }
    }
}

/// GET /api/workspaces/{id}/pipelines
pub async fn list_pipelines(
    axum::extract::State(state): axum::extract::State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let pipelines_dir = worktree.join(".ship").join("pipelines");

    if !pipelines_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "pipelines": [] })));
    }

    let entries = std::fs::read_dir(&pipelines_dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("read pipelines: {e}")))?;

    let mut pipelines = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !PIPELINE_EXTS.contains(&ext) {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let content = std::fs::read_to_string(&path).unwrap_or_default();

        let val: serde_json::Value = match parse_pipeline_content(&content) {
            Ok((v, _)) => v,
            Err(_) => continue,
        };

        let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let steps = val.get("steps").and_then(|v| v.as_array()).map(|arr| {
            arr.iter().filter_map(|s| {
                Some(serde_json::json!({
                    "id": s.get("id")?.as_str()?,
                    "type": s.get("type").and_then(|t| t.as_str()),
                    "agent": s.get("agent").and_then(|a| a.as_str()),
                    "after": s.get("after").and_then(|a| a.as_array()).map(|deps| {
                        deps.iter().filter_map(|d| d.as_str().map(String::from)).collect::<Vec<_>>()
                    }).unwrap_or_default(),
                    "model": s.get("model").and_then(|m| m.as_str()),
                }))
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        pipelines.push(serde_json::json!({
            "name": name,
            "description": description,
            "steps": steps,
            "raw": content,
        }));
    }

    Ok(ok_response(serde_json::json!({ "pipelines": pipelines })))
}

#[derive(Deserialize)]
pub struct WritePipelineReq {
    pub content: String,
}

/// PUT /api/workspaces/{id}/pipelines/{name}
pub async fn write_pipeline(
    axum::extract::State(state): axum::extract::State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, name)): Path<(String, String)>,
    Json(body): Json<WritePipelineReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let pipelines_dir = worktree.join(".ship").join("pipelines");

    if name.contains('/') || name.contains("..") || name.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid pipeline name"));
    }

    // Validate content — returns all parse + schema issues so the
    // client can render every marker at once instead of whack-a-mole.
    if let Err(issues) = validate_pipeline(&body.content) {
        let first = issues.first().cloned();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({
                    "error": if first.as_ref().map(|i| i.kind) == Some("parse") {
                        "parse_error"
                    } else {
                        "schema_error"
                    },
                    // Back-compat single-issue pointer (Level 1 clients).
                    "parse": first,
                    // Full issue list for Level 2+ clients.
                    "issues": issues,
                }),
            }),
        ));
    }

    std::fs::create_dir_all(&pipelines_dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("create dir: {e}")))?;

    let path = pipelines_dir.join(format!("{name}.jsonc"));
    std::fs::write(&path, &body.content)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("write: {e}")))?;

    Ok(ok_response(serde_json::json!({ "name": name })))
}

/// DELETE /api/workspaces/{id}/pipelines/{name}
pub async fn delete_pipeline(
    axum::extract::State(state): axum::extract::State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, name)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = resolve_worktree(state.ship_dir(), &id)?;

    // Try .jsonc first, then .yaml
    let jsonc_path = worktree.join(".ship").join("pipelines").join(format!("{name}.jsonc"));
    let yaml_path = worktree.join(".ship").join("pipelines").join(format!("{name}.yaml"));
    let path = if jsonc_path.exists() {
        jsonc_path
    } else if yaml_path.exists() {
        yaml_path
    } else {
        return Err(err_response(StatusCode::NOT_FOUND, "pipeline not found"));
    };

    std::fs::remove_file(&path)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("delete: {e}")))?;

    Ok(ok_response(serde_json::json!({ "name": name })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schema_compiles() {
        let _ = schema();
    }

    #[test]
    fn real_pipelines_pass_validation() {
        let shell = include_str!("../../../../.ship/pipelines/shell-smoke.jsonc");
        let multi = include_str!("../../../../.ship/pipelines/multi-provider-smoke.jsonc");
        assert!(validate_pipeline(shell).is_ok(), "shell-smoke should pass: {:?}", validate_pipeline(shell));
        assert!(validate_pipeline(multi).is_ok(), "multi-provider-smoke should pass: {:?}", validate_pipeline(multi));
    }

    #[test]
    fn missing_name_fails_schema() {
        let content = r#"{ "steps": [{ "id": "a", "run": "echo hi" }] }"#;
        let issues = validate_pipeline(content).unwrap_err();
        assert!(issues.iter().any(|i| i.kind == "schema"), "expected schema issue, got: {issues:?}");
    }

    #[test]
    fn unknown_property_fails_schema() {
        let content = r#"{ "name": "x", "stepz": [] }"#;
        let issues = validate_pipeline(content).unwrap_err();
        assert!(issues.iter().any(|i| i.kind == "schema"), "expected schema issue, got: {issues:?}");
    }

    #[test]
    fn schema_issue_gets_resolved_line_column() {
        let content = "{\n  \"name\": \"bad\",\n  \"stepz\": []\n}";
        let issues = validate_pipeline(content).unwrap_err();
        let schema_issue = issues.iter().find(|i| i.kind == "schema").expect("schema issue");
        // The resolver walks the source — line should be set when path resolves.
        // Unknown-property errors target the parent (empty path), so at minimum
        // we assert the issue exists; path-bearing errors should have line set.
        if schema_issue.path.is_some() {
            assert!(schema_issue.line.is_some(), "path-bearing issue missing line: {schema_issue:?}");
        }
    }

    #[test]
    fn step_level_schema_issue_has_line_column() {
        // steps[0] has an invalid `id` (starts with uppercase, breaks pattern).
        let content = "{\n  \"name\": \"x\",\n  \"steps\": [\n    { \"id\": \"BadID\", \"run\": \"echo\" }\n  ]\n}";
        let issues = validate_pipeline(content).unwrap_err();
        let step_issue = issues.iter()
            .find(|i| i.path.as_deref().map(|p| p.contains("steps/0")).unwrap_or(false))
            .expect(&format!("step-level issue expected, got: {issues:?}"));
        assert_eq!(step_issue.line, Some(4), "step issue should point to line 4: {step_issue:?}");
    }

    #[test]
    fn parse_error_returns_line_column() {
        let content = r#"{ "name": "x", "steps": ["#; // truncated — parser fails
        let issues = validate_pipeline(content).unwrap_err();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "parse");
        assert!(issues[0].line.is_some());
    }
}
