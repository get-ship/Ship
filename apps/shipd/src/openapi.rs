//! OpenAPI spec generation for the Ship daemon REST API.

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Ship Daemon API", version = "0.3.0"),
    paths(
        crate::rest_api::mesh_register,
        crate::rest_api::mesh_send,
        crate::rest_api::mesh_broadcast,
        crate::rest_api::mesh_discover,
        crate::rest_api::mesh_status_update,
        crate::rest_api::mesh_events,
        crate::rest_api::capabilities,
        crate::runtime_api::list_workspaces,
        crate::runtime_api::list_sessions,
        crate::runtime_api::list_agents,
        crate::runtime_api::list_jobs,
        crate::runtime_api::create_job,
        crate::runtime_api::end_job,
        crate::runtime_api::event_stream,
        crate::runtime_api::rebuild_projections,
        crate::actor_api::actor_spawn,
        crate::actor_api::event_route,
        crate::settings_api::list_settings,
        crate::settings_api::update_setting,
        crate::settings_api::get_setting,
        crate::settings_api::list_providers,
    ),
    components(schemas(
        crate::rest_api::MeshRegisterReq,
        crate::rest_api::MeshSendReq,
        crate::rest_api::MeshBroadcastReq,
        crate::rest_api::MeshStatusReq,
        crate::rest_api::MeshResponse,
        crate::rest_api::CapabilitiesResponse,
        crate::rest_api::CheckoutHookReq,
        crate::runtime_api::CreateJobRequest,
        crate::runtime_api::JobEndRequest,
        crate::actor_api::ActorSpawnReq,
        crate::actor_api::EventRouteReq,
        crate::settings_api::SettingsUpdateReq,
        crate::settings_api::ProviderStatus,
    ))
)]
pub struct ApiDoc;
