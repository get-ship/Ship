//! OpenAPI spec generation for the Ship daemon REST API.
//!
//! Every enumerated route in `routes.rs` + the hand-rolled paths in
//! `router.rs` / `webhook_ext.rs` is advertised via `ApiDoc::paths(...)`.
//! Post-generation surgery (global security + public-path overrides) lives
//! in `openapi_modifier::SecurityAndServer`.
//!
//! Handlers declare their own `#[utoipa::path]` annotations with explicit
//! 4xx/5xx `ErrorResponse` entries. The drift-check script
//! (`apps/docs/scripts/check-openapi-drift.ts`) re-runs `ship openapi export`
//! and compares against the committed `apps/docs/openapi.json` so reviewers
//! catch the "I forgot to regenerate" class of bug.

use utoipa::OpenApi;

use crate::openapi_modifier::SecurityAndServer;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Ship Daemon API",
        version = "0.3.0",
        description = "HTTP surface for the Ship daemon (shipd).\n\n\
            **Base URL:** `http://localhost:<port>` — the daemon runs locally.\n\
            - Port: `~/.ship/daemon.port`\n\
            - Bearer token: `~/.ship/daemon.token`\n\n\
            **Quick-start**\n\
            ```bash\n\
            export SHIP_TOKEN=$(cat ~/.ship/daemon.token)\n\
            export SHIP_PORT=$(cat ~/.ship/daemon.port)\n\
            curl -s -H \"Authorization: Bearer $SHIP_TOKEN\" \\\n\
              http://localhost:$SHIP_PORT/api/runtime/daemon/status\n\
            ```\n\n\
            Bearer auth is required by default; public operations carry an empty `security` override. \
            See the [errors reference](/reference/http-api/errors/) for the `ErrorResponse` contract.",
    ),
    modifiers(&SecurityAndServer),
    tags(
        (name = "health", description = "Daemon liveness and supervised-task status."),
        (name = "auth", description = "Browser pairing, server attestation, and session management."),
        (name = "runtime", description = "Workspaces, sessions, agents, jobs, events."),
        (name = "specs", description = "Spec CRUD and job-launch-from-spec."),
        (name = "artifacts", description = "Three-scope artifact upload/list/download/promote."),
        (name = "collections", description = "Structured-data collections CRUD + query."),
        (name = "settings", description = "Daemon settings and provider detection."),
        (name = "capabilities", description = "Caller-visible daemon capabilities and mode."),
        (name = "mesh", description = "Agent mesh registration, directed send, broadcast, SSE."),
        (name = "actor", description = "Actor spawn and event routing."),
        (name = "hooks", description = "Claude hook adapters, checkout hook, and public webhook trigger."),
        (name = "views", description = "HTML view shells, SSE streams, and shared assets."),
        (name = "project", description = "Project- and workspace-scoped git, files, agents, skills, and pipelines."),
    ),
    paths(
        // ── Mesh / actors
        crate::rest_api::mesh_register,
        crate::rest_api::mesh_send,
        crate::rest_api::mesh_broadcast,
        crate::rest_api::mesh_discover,
        crate::rest_api::mesh_status_update,
        crate::rest_api::mesh_events,
        crate::rest_api::capabilities,
        crate::rest_api::hooks_checkout,
        crate::actor_api::actor_spawn,
        crate::actor_api::event_route,
        // ── Auth + pairing
        crate::auth::pair_init,
        crate::auth::pair_confirm,
        crate::auth::pair_status,
        crate::auth::server_attest,
        crate::auth::list_sessions,
        crate::auth::revoke_sessions,
        crate::scopes::session_register,
        crate::session::issue_session_handler,
        // ── Health
        crate::router::health_check,
        // ── Runtime + jobs + events
        crate::runtime_api::list_workspaces,
        crate::runtime_api::list_sessions,
        crate::runtime_api::list_agents,
        crate::runtime_api::list_jobs,
        crate::runtime_api::get_job_detail,
        crate::runtime_api::create_job,
        crate::runtime_api::end_job,
        crate::runtime_api::cancel_job,
        crate::runtime_api::stop_agent,
        crate::runtime_api::steer_agent,
        crate::runtime_api::list_procs,
        crate::runtime_api::poll_events,
        crate::runtime_api::daemon_status,
        crate::runtime_api::event_stream,
        crate::runtime_api::rebuild_projections,
        // ── Feedback + annotations
        crate::feedback_api::post_feedback,
        crate::annotations_api::post_annotation,
        // ── Specs
        crate::specs_api::list_specs,
        crate::specs_api::get_spec,
        crate::specs_api::create_spec,
        crate::specs_api::update_spec,
        crate::specs_api::delete_spec,
        crate::specs_api::launch_spec,
        // ── Artifacts
        crate::artifacts_api::upload_artifact,
        crate::artifacts_api::list_artifacts,
        crate::artifacts_api::download_artifact,
        crate::artifacts_api::head_artifact,
        crate::artifacts_api::artifact_metadata,
        crate::artifacts_api::patch_artifact,
        crate::artifacts_api::delete_artifact,
        crate::artifacts_api::promote_artifact,
        // ── Collections
        crate::collection_api::list_collections,
        crate::collection_api::get_schema,
        crate::collection_api::create_item,
        crate::collection_api::get_item,
        crate::collection_api::update_item,
        crate::collection_api::delete_item,
        crate::collection_api::query_items,
        // ── Settings
        crate::settings_api::list_settings,
        crate::settings_api::update_setting,
        crate::settings_api::get_setting,
        crate::settings_api::list_providers,
        // ── Hooks
        crate::hooks_api::hook_session_start,
        crate::hooks_api::hook_before_tool,
        crate::hooks_api::hook_after_tool,
        crate::hooks_api::hook_session_end,
        crate::webhook_ext::handle_webhook,
        crate::human_webhook::telegram_webhook,
        // ── MCP registry
        crate::mcp_registry::list_mcp_servers,
        // ── Supervisor + PTY
        crate::supervisor::workspace_api::start_workspace,
        crate::pty_handler::workspace_pty,
        // ── Project / workspace
        crate::project_api::session_files::list_session_files,
        crate::project_api::session_files::read_session_file,
        crate::project_api::session_files::write_session_file,
        crate::project_api::session_files::delete_session_file,
        crate::project_api::agents_skills_write::write_skill_file,
        crate::project_api::agents_skills_write::delete_skill_file,
        crate::project_api::git::git_status,
        crate::project_api::git::git_diff,
        crate::project_api::git::git_log,
        crate::project_api::git::project_git_status,
        crate::project_api::git::project_git_diff,
        crate::project_api::git::project_git_log,
        crate::project_api::agents_skills::list_agents,
        crate::project_api::agents_skills_write::write_agent,
        crate::project_api::agents_skills::list_skills,
        crate::project_api::agents_skills_write::get_skill_vars,
        crate::project_api::agents_skills_write::set_skill_var,
        crate::project_api::pipelines::list_pipelines,
        crate::project_api::pipelines::write_pipeline,
        crate::project_api::pipelines::delete_pipeline,
        crate::project_api::delete_workspace,
        crate::project_api::activate_workspace,
        crate::project_api::emit_event,
        // ── Views (HTML shells + SSE)
        crate::views::handlers::list_views,
        crate::views::handlers::serve_view,
        crate::views::handlers::serve_view_manifest,
        crate::views::handlers::serve_view_bundle,
        crate::views_raw::serve_view_raw,
        crate::views::generic_sse::serve_view_sse,
        crate::view_sessions_api::open_session,
        crate::view_sessions_api::patch_session,
        crate::view_sessions_api::append_session,
        crate::view_sessions_api::signal_session,
        crate::view_sessions_api::close_session,
        crate::view_sessions_api::subscribe_session,
        crate::view_sessions_api::raw_session,
        crate::views_git::git_shell,
        crate::views_git::git_sse,
        crate::views_pipelines::pipelines_shell,
        crate::views_pipelines::pipelines_sse,
        crate::views_settings::settings_shell,
        crate::views_settings::settings_sse,
        crate::views_agents::agents_shell,
        crate::views_agents::agent_detail,
        crate::views_agents::agents_create,
        crate::views_skills::skills_shell,
        crate::views_skills::skills_rail,
        crate::views_skills::skill_detail,
        crate::views_chat::chat_shell,
        crate::views_chat::chat_sse,
        crate::views_chat::chat_send,
        crate::views_events::events_shell,
        crate::views_events::events_sse,
        crate::specs_view::specs_shell,
        crate::specs_view::specs_view_sse,
        crate::views::setup::index_handler,
        crate::views::setup::setup_handler,
        crate::views::assets::serve_datastar_js,
        crate::views::assets::serve_view_tokens,
        crate::views::assets::serve_ship_elements,
        crate::views::assets::serve_code_editor,
        crate::views::assets::serve_markdown_editor,
        crate::views::assets::serve_terminal,
        crate::views::assets::serve_elements_manifest,
        crate::views::assets::serve_elements_catalog,
        crate::views::assets::serve_theme_css,
        crate::views::assets::serve_themes_list,
    ),
    components(schemas(
        // Shared envelope + error types
        crate::rest_api::MeshResponse,
        crate::openapi_types::ErrorResponse,
        crate::openapi_types::WebhookAck,
        crate::openapi_types::GenericOk,
        // Mesh + actor + capabilities
        crate::rest_api::MeshRegisterReq,
        crate::rest_api::MeshSendReq,
        crate::rest_api::MeshBroadcastReq,
        crate::rest_api::MeshStatusReq,
        crate::rest_api::CapabilitiesResponse,
        crate::rest_api::CheckoutHookReq,
        crate::actor_api::ActorSpawnReq,
        crate::actor_api::EventRouteReq,
        // Auth + scopes + sessions
        crate::auth::PairInitRequest,
        crate::auth::PairInitResponse,
        crate::auth::PairConfirmRequest,
        crate::auth::PairConfirmResponse,
        crate::auth::PairStatusResponse,
        crate::auth::AttestRequest,
        crate::auth::AttestResponse,
        crate::auth::SessionSummary,
        crate::auth::ListSessionsResponse,
        crate::auth::RevokeSessionsRequest,
        crate::auth::RevokeSessionsResponse,
        crate::scopes::Scope,
        crate::scopes::SessionRegisterRequest,
        crate::scopes::SessionRegisterResponse,
        crate::session::SessionIssueResponse,
        // Runtime / jobs
        crate::runtime_api::CreateJobRequest,
        crate::runtime_api::JobEndRequest,
        // Specs
        crate::specs_api::CreateSpecRequest,
        crate::specs_api::UpdateSpecRequest,
        // Settings + providers
        crate::settings_api::SettingsUpdateReq,
        crate::settings_api::ProviderStatus,
        // Artifacts
        crate::artifacts_api::PatchBody,
        crate::artifacts_api::PromoteBody,
        // Collections
        crate::collection_api::CreateItemRequest,
        crate::collection_api::UpdateItemRequest,
        // Hooks
        crate::hooks_api::ShipHookEvent,
        crate::hooks_api::HookResponse,
        // MCP registry
        crate::mcp_registry::McpServer,
        crate::mcp_registry::McpServersResponse,
        // Project / workspace
        crate::project_api::WriteAgentReq,
        crate::project_api::SetVarReq,
        crate::project_api::EmitEventReq,
        crate::project_api::WritePipelineReq,
        // View sessions
        crate::view_sessions_api::OpenReq,
        crate::view_sessions_api::PatchReq,
        crate::view_sessions_api::SignalReq,
        // Views: chat
        crate::views_chat::ChatSendRequest,
        // Supervisor
        crate::supervisor::workspace_api::StartWorkspaceRequest,
        crate::supervisor::workspace_api::StartWorkspaceResponse,
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
