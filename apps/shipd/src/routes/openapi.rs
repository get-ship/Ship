//! OpenAPI spec generation for the Ship daemon REST API.

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::ServerBuilder;
use utoipa::{Modify, OpenApi};

/// Adds the bearer security scheme and localhost server to the generated spec.
struct SecurityAndServer;

impl Modify for SecurityAndServer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("opaque")
                        .description(Some(
                            "Daemon bearer token — found in ~/.ship/machine/daemon/token after `ship daemon start`.",
                        ))
                        .build(),
                ),
            );
        }
        openapi.servers = Some(vec![
            ServerBuilder::new()
                .url("http://localhost:9315")
                .description(Some("Local shipd daemon"))
                .build(),
        ]);
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Ship Daemon API", version = "0.3.0"),
    modifiers(&SecurityAndServer),
    paths(
        crate::rest_api::mesh_register,
        crate::rest_api::mesh_send,
        crate::rest_api::mesh_broadcast,
        crate::rest_api::mesh_discover,
        crate::rest_api::mesh_status_update,
        crate::rest_api::mesh_events,
        crate::rest_api::capabilities,
        crate::runtime_api::list_sessions,
        crate::runtime_api::agents::list_agents,
        crate::runtime_api::jobs::list_jobs,
        crate::runtime_api::dispatch::dispatch_pipeline,
        crate::runtime_api::jobs::end_job,
        crate::runtime_api::events::event_stream,
        crate::runtime_api::daemon::rebuild_projections,
        crate::actor_api::actor_spawn,
        crate::actor_api::event_route,
        crate::project_api::emit_event,
        crate::project_api::jobs_status::update_job_status,
        crate::project_api::jobs_status::job_step_done,
        crate::settings_api::list_settings,
        crate::settings_api::update_setting,
        crate::settings_api::get_setting,
        crate::settings_api::list_providers,
        // Framework surface added in the ship-set-architecture arc.
        crate::services::schema::serve_schema,
        crate::services::chat::send::send_message,
        crate::views::routed_handler::serve_view_routed,
        crate::views::routed_sse::serve_view_sse_routed,
        crate::views::source_api::serve_view_source,
        crate::create_api::create_surface,
        crate::views::write_api::write_view_template,
        crate::views::write_api::write_view_manifest,
        crate::views::write_api::fork_view,
        crate::themes_api::theme_contrast,
        crate::sessions::api::spawn_session,
        crate::sessions::api::list_sessions,
        crate::sessions::api::get_session,
        crate::sessions::api::stop_session,
        crate::sessions::api::session_input,
        crate::sessions::api::session_message,
        crate::sessions::api::session_reply,
        crate::sessions::api::stream_session,
        crate::sessions::api::stream_session_events,
        crate::sessions::api::installed_providers,
        crate::projects_api::list_projects,
        crate::projects_api::add_project,
        crate::projects_api::switch_project,
        // Static assets + lazy island bundles.
        crate::views::assets::serve_datastar_js,
        crate::views::assets::serve_view_tokens,
        crate::views::assets::serve_ship_elements,
        crate::views::assets::serve_elements_manifest,
        crate::views::assets::serve_elements_catalog,
        crate::views::assets::serve_code_editor,
        crate::views::assets::serve_markdown_editor,
        crate::views::assets::serve_terminal,
        crate::views::assets::serve_git,
        crate::views::assets::serve_jinja_macros_json,
        crate::views::assets::serve_jinja_macros_md,
    ),
    components(schemas(
        crate::rest_api::MeshRegisterReq,
        crate::rest_api::MeshSendReq,
        crate::rest_api::MeshBroadcastReq,
        crate::rest_api::MeshStatusReq,
        crate::rest_api::MeshResponse,
        crate::rest_api::CapabilitiesResponse,
        crate::rest_api::CheckoutHookReq,
        crate::runtime_api::DispatchRequest,
        crate::runtime_api::JobEndRequest,
        crate::actor_api::ActorSpawnReq,
        crate::actor_api::EventRouteReq,
        crate::project_api::EmitEventReq,
        crate::project_api::JobStatusReq,
        crate::project_api::StepDoneReq,
        crate::settings_api::SettingsUpdateReq,
        crate::settings_api::ProviderStatus,
        crate::services::chat::send::ChatSendRequest,
        crate::views::source_api::ViewSourceResponse,
        crate::create_api::CreateRequest,
        crate::create_api::CreateResponse,
        crate::create_api::CreateData,
        crate::views::write_api::WriteTemplateRequest,
        crate::views::write_api::WriteManifestRequest,
        crate::views::write_api::WriteResponse,
        crate::views::write_api::WriteDiagnostics,
        crate::views::write_api::LintFinding,
        crate::themes_api::ContrastRequest,
        crate::themes_api::ContrastReport,
        crate::themes_api::PairReport,
        crate::sessions::api::SpawnBody,
        crate::sessions::api::SpawnResponse,
        crate::sessions::api::SessionListResponse,
        crate::sessions::api::SessionMetaWire,
        crate::sessions::api::InputBody,
        crate::sessions::api::MessageBody,
        crate::sessions::api::InstalledProvidersResponse,
        crate::projects_api::Project,
        crate::projects_api::ProjectsResponse,
        crate::projects_api::ProjectAddReq,
        crate::projects_api::ProjectSwitchReq,
    ))
)]
pub struct ApiDoc;

/// Generate the OpenAPI spec as a JSON string.
///
/// `pretty` controls indented vs compact output. Errors are returned as
/// `anyhow::Error` so callers don't need to import utoipa.
pub fn to_json(pretty: bool) -> anyhow::Result<String> {
    use utoipa::OpenApi as _;
    let spec = ApiDoc::openapi();
    let json = if pretty {
        spec.to_pretty_json()?
    } else {
        spec.to_json()?
    };
    Ok(json)
}

#[cfg(test)]
#[path = "openapi_tests.rs"]
mod tests;
