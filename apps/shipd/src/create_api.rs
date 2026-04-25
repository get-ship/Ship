//! `POST /api/create/{surface}` — server-side feature scaffolding.
//!
//! The shell's create dialog posts here to materialize a new feature
//! on disk. For `view`, we copy from
//! `.ship/skills/ship-views/scaffolds/<shape>/` into
//! `<ship-dir>/views/<name>/`, substituting `{{TITLE}}` and `{{NAME}}`
//! placeholders. Other surfaces return 501 until their skills grow
//! scaffold directories in a later commit.
//!
//! The handler is intentionally synchronous: the user submits the
//! dialog, the scaffold lands, we return the new route. Shipwright's
//! role is post-scaffold iteration via chat — not the initial write.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::rest_api::ApiState;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateRequest {
    /// Kebab-case feature id. Becomes the directory or file name.
    pub name: String,
    /// One-line purpose for the agent + human readers. Optional.
    #[serde(default)]
    pub purpose: Option<String>,
    /// Starting shape (e.g. `list` | `form` | `dashboard` | `blank`
    /// for views). Skill-specific.
    pub shape: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CreateResponse {
    pub ok: bool,
    pub data: CreateData,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CreateData {
    /// Route where the new feature can be opened. `None` when the
    /// surface doesn't have a browsable route.
    pub route: Option<String>,
    /// Absolute path to the scaffolded artifact (directory or file).
    pub path: String,
    /// Surface that was created.
    pub surface: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

const KEBAB: &str = "abcdefghijklmnopqrstuvwxyz0123456789-";

fn valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 48 {
        return false;
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
        return false;
    }
    name.chars().all(|c| KEBAB.contains(c))
}

fn title_case(name: &str) -> String {
    name.split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[utoipa::path(
    operation_id = "createSurface",
    post,
    path = "/api/create/{surface}",
    params(("surface" = String, Path, description = "view|skill|agent|pipeline|trigger|theme")),
    request_body = CreateRequest,
    responses(
        (status = 200, description = "Scaffold written", body = CreateResponse),
        (status = 400, description = "Invalid name or shape"),
        (status = 409, description = "Target already exists"),
        (status = 501, description = "Surface scaffolding not yet implemented"),
    ),
    tag = "framework"
)]
pub async fn create_surface(
    State(state): State<ApiState>,
    Path(surface): Path<String>,
    Json(req): Json<CreateRequest>,
) -> axum::response::Response {
    if !valid_name(&req.name) {
        return err(StatusCode::BAD_REQUEST, "name must be kebab-case");
    }

    let resp = match surface.as_str() {
        "view" => scaffold_view(&state, &req),
        "skill" | "agent" | "pipeline" | "trigger" | "theme" => err(
            StatusCode::NOT_IMPLEMENTED,
            format!("surface '{surface}' scaffolding lands in a later commit"),
        ),
        other => err(StatusCode::BAD_REQUEST, format!("unknown surface '{other}'")),
    };

    // Emit a kernel completion event so the shell can toast + route.
    // Fire-and-forget — scaffolding already succeeded by the time we
    // get here, and a best-effort notification shouldn't fail the
    // request if the kernel is busy.
    if resp.status() == StatusCode::OK {
        let state = state.clone();
        let surface = surface.clone();
        let name = req.name.clone();
        let route = if surface == "view" {
            Some(format!("/api/v/{name}"))
        } else {
            None
        };
        tokio::spawn(async move {
            let payload = serde_json::json!({
                "surface": surface,
                "name": name,
                "route": route,
            });
            if let Ok(envelope) = runtime::events::EventEnvelope::new(
                "agent.task.complete",
                &name,
                &payload,
            ) {
                let ctx = runtime::events::EmitContext {
                    caller_kind: runtime::events::CallerKind::Cli,
                    skill_id: None,
                    workspace_id: None,
                    session_id: None,
                };
                let _ = state.kernel().lock().await.route(envelope, &ctx).await;
            }
        });
    }

    resp
}

fn scaffold_view(state: &ApiState, req: &CreateRequest) -> axum::response::Response {
    // Locate the scaffold source. Looks in the project .ship/ first,
    // then the user-global .ship/. Matches how resolve_view_dir ranks
    // lookup so overrides behave predictably.
    let scaffold_rel = std::path::PathBuf::from("skills")
        .join("ship-views")
        .join("scaffolds")
        .join(&req.shape);

    let mut scaffold_src: Option<std::path::PathBuf> = None;
    if let Ok(d) = runtime::project::get_project_dir(None) {
        let p = d.join(&scaffold_rel);
        if p.is_dir() {
            scaffold_src = Some(p);
        }
    }
    if scaffold_src.is_none() {
        let p = state.ship_dir().join(&scaffold_rel);
        if p.is_dir() {
            scaffold_src = Some(p);
        }
    }

    let Some(src) = scaffold_src else {
        return err(
            StatusCode::BAD_REQUEST,
            format!("unknown shape '{}'", req.shape),
        );
    };

    // Target: the user-global ship dir so the new view is editable by
    // definition (bundled views would fail the "source = bundled" edit
    // rule when we land editability tiers).
    let views_dir = state.ship_dir().join("views");
    if !views_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&views_dir) {
            return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    }
    let target = views_dir.join(&req.name);
    if target.exists() {
        return err(
            StatusCode::CONFLICT,
            format!("view '{}' already exists", req.name),
        );
    }

    let title = title_case(&req.name);
    if let Err(e) = copy_scaffold(&src, &target, &title, &req.name) {
        // Best-effort cleanup so a partial copy doesn't leave junk behind.
        let _ = std::fs::remove_dir_all(&target);
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let route = format!("/api/v/{}", req.name);
    (
        StatusCode::OK,
        Json(CreateResponse {
            ok: true,
            data: CreateData {
                route: Some(route),
                path: target.display().to_string(),
                surface: "view".into(),
            },
        }),
    )
        .into_response()
}

fn copy_scaffold(
    src: &std::path::Path,
    dst: &std::path::Path,
    title: &str,
    name: &str,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_scaffold(&from, &to, title, name)?;
        } else if ty.is_file() {
            let body = std::fs::read_to_string(&from).unwrap_or_default();
            let substituted = body
                .replace("{{TITLE}}", title)
                .replace("{{NAME}}", name);
            std::fs::write(&to, substituted)?;
        }
    }
    Ok(())
}
