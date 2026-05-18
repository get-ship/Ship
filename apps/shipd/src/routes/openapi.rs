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
        crate::routes::rest_api::mesh_register,
        crate::routes::rest_api::mesh_send,
        crate::routes::rest_api::mesh_broadcast,
        crate::routes::rest_api::mesh_discover,
        crate::routes::rest_api::mesh_status_update,
        crate::routes::rest_api::mesh_events,
        crate::routes::rest_api::capabilities,
        crate::routes::runtime_api::list_workspace_sessions,
        crate::routes::runtime_api::agents::list_agents,
        crate::routes::runtime_api::jobs::list_jobs,
        crate::routes::runtime_api::dispatch::dispatch_agent_session,
        crate::routes::runtime_api::pipeline_run::run_pipeline,
        crate::routes::runtime_api::jobs::end_job,
        crate::routes::runtime_api::events::event_stream,
        crate::routes::runtime_api::daemon::rebuild_projections,
        crate::routes::actor_api::actor_spawn,
        crate::routes::actor_api::event_route,
        crate::project_api::emit_event,
        crate::project_api::jobs_status::update_job_status,
        crate::project_api::jobs_status::job_step_done,
        crate::routes::settings_api::list_settings,
        crate::routes::settings_api::update_setting,
        crate::routes::settings_api::get_setting,
        crate::routes::settings_api::list_harnesses,
        // Framework surface added in the ship-set-architecture arc.
        crate::services::schema::serve_schema,
        crate::services::chat::send::send_message,
        crate::views::source_api::serve_view_source,
        crate::views::files_api::list_view_files,
        crate::routes::create_api::create_surface,
        crate::views::write_api::write_view_template,
        crate::views::write_api::write_view_manifest,
        crate::views::write_api::fork_view,
        crate::routes::themes_api::theme_contrast,
        crate::sessions::api::crud::spawn_session,
        crate::sessions::api::crud::list_pty_sessions,
        crate::sessions::api::crud::get_session,
        crate::sessions::api::crud::stop_session,
        crate::sessions::api::io::session_input,
        crate::sessions::api::messages::session_message,
        crate::sessions::api::messages::session_reply,
        crate::sessions::api::io::stream_session,
        crate::sessions::api::io::stream_session_events,
        crate::sessions::api::crud::installed_harnesses,
        crate::routes::projects_api::list_projects,
        crate::routes::projects_api::add_project,
        crate::routes::projects_api::switch_project,
        // SUI-era shared assets.
        crate::views::assets::serve_view_tokens,
        crate::views::assets::serve_theme_css,
        crate::views::assets::serve_themes_list,
        // Account — daemon-mediated Ship Cloud identity (CallerKind::Cli only).
        crate::routes::account_api::start_login,
        crate::routes::account_api::complete_login,
        crate::routes::account_api::get_state,
        crate::routes::account_api::logout,
        crate::routes::account_api::get_key,
    ),
    components(schemas(
        crate::routes::rest_api::MeshRegisterReq,
        crate::routes::rest_api::MeshSendReq,
        crate::routes::rest_api::MeshBroadcastReq,
        crate::routes::rest_api::MeshStatusReq,
        crate::wire::MeshResponse,
        crate::routes::rest_api::CapabilitiesResponse,
        crate::routes::rest_api::CheckoutHookReq,
        crate::routes::runtime_api::DispatchRequest,
        crate::routes::runtime_api::PipelineRunRequest,
        crate::routes::runtime_api::JobEndRequest,
        crate::routes::actor_api::ActorSpawnReq,
        crate::routes::actor_api::EventRouteReq,
        crate::project_api::EmitEventReq,
        crate::project_api::JobStatusReq,
        crate::project_api::StepDoneReq,
        crate::routes::settings_api::SettingsUpdateReq,
        crate::routes::settings_api::ProviderStatus,
        crate::services::chat::send::ChatSendRequest,
        crate::views::source_api::ViewSourceResponse,
        crate::routes::create_api::CreateRequest,
        crate::routes::create_api::CreateResponse,
        crate::routes::create_api::CreateData,
        crate::views::write_api::WriteTemplateRequest,
        crate::views::write_api::WriteManifestRequest,
        crate::views::write_api::WriteResponse,
        crate::views::write_api::WriteDiagnostics,
        crate::views::write_api::LintFinding,
        crate::routes::themes_api::ContrastRequest,
        crate::routes::themes_api::ContrastReport,
        crate::routes::themes_api::PairReport,
        crate::sessions::api::SpawnBody,
        crate::sessions::api::SpawnResponse,
        crate::sessions::api::SessionListResponse,
        crate::sessions::api::SessionMetaWire,
        crate::sessions::api::InputBody,
        crate::sessions::api::MessageBody,
        crate::sessions::api::InstalledProvidersResponse,
        crate::routes::projects_api::Project,
        crate::routes::projects_api::ProjectsResponse,
        crate::routes::projects_api::ProjectAddReq,
        crate::routes::projects_api::ProjectSwitchReq,
        crate::routes::account_api::StartLoginRequest,
        crate::routes::account_api::StartLoginResponse,
        crate::routes::account_api::CompleteLoginRequest,
        crate::routes::account_api::CompleteLoginResponse,
        crate::routes::account_api::GetKeyResponse,
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
