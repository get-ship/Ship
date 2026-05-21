//! Single-source `agents.*` ops — descriptors + dispatch from one
//! declaration via `ship_ops!`. Replaces the hand-synced OPS/dispatch pair
//! (and supersedes the `agents_typed_proof` spike).

use super::{parse_scope, required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, PROJECT_SCOPE};

use ship_platform::services::agents::state::ScopeTag;
use ship_platform::services::agents::view_state::{
    list_agent_catalogs, list_agents, load_agent, AgentCatalogs, AgentDetail,
    AgentsSummariesResponse,
};
use ship_platform::services::agents::wire_types::{
    AgentGetResponse, AgentIndexResponse, AgentTemplatesResponse, HarnessesResponse,
    PermissionPresetsResponse,
};
use ship_platform::services::agents::create::AgentsCreateCtx;
use ship_platform::services::agents::settings_view::AgentSettingsCtx;
use ship_platform::services::agents::wire::AgentWireCtx;
use ship_platform::services::agents::{create, settings_view, state, wire};

ship_ops! {
    AGENTS_OPS;
    {
        name: "agents.list",
        desc: "List all agents as minimal IndexEntry rows.",
        perm: "ship.agents.read",
        args: &[],
        ret: AgentIndexResponse,
        run: |_args, _state, files| Ok(AgentIndexResponse { items: state::list(files) }),
    },
    {
        name: "agents.summaries",
        desc: "List all agents with display metadata (label, harnesses, tags).",
        perm: "ship.agents.read",
        args: &[],
        ret: AgentsSummariesResponse,
        run: |_args, state, _files| {
            Ok(AgentsSummariesResponse { items: list_agents(state.ship_dir()) })
        },
    },
    {
        name: "agents.detail",
        desc: "Full view-shaped agent detail — catalogs, permissions, harnesses, tags.",
        perm: "ship.agents.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        ret: AgentDetail,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            load_agent(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))
        },
    },
    {
        name: "agents.catalogs",
        desc: "Catalog rows for the agents list (harnesses, skills, etc.).",
        perm: "ship.agents.read",
        args: &[],
        ret: AgentCatalogs,
        run: |_args, state, _files| Ok(list_agent_catalogs(state.ship_dir())),
    },
    {
        name: "agents.get",
        desc: "Raw agent doc from disk at the requested scope.",
        perm: "ship.agents.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        ret: AgentGetResponse,
        run: |args, _state, files| {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, doc) = state::load_doc(files, id, scope).map_err(|e| match e {
                state::Error::NotFound(m) => RegistryError::not_found(m),
                other => RegistryError::service(other.to_string()),
            })?;
            Ok(AgentGetResponse { id: id.to_string(), scope: ScopeTag::from(resolved), doc })
        },
    },
    {
        name: "agents.wire",
        desc: "Graph payload (nodes + edges) for the agent-wire canvas.",
        perm: "ship.agents.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        ret: AgentWireCtx,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            wire::build(id, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))
        },
    },
    {
        name: "agents.settings",
        desc: "Tier list + catalog rows for the per-agent settings pane.",
        perm: "ship.agents.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        ret: AgentSettingsCtx,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            settings_view::build(id, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))
        },
    },
    {
        name: "agents.templates",
        desc: "Starter templates for the agent-create wizard.",
        perm: "ship.agents.read",
        args: &[],
        ret: AgentTemplatesResponse,
        run: |_args, _state, _files| {
            Ok(AgentTemplatesResponse { items: create::builtin_templates() })
        },
    },
    {
        name: "agents.permission_presets",
        desc: "Permission presets for the agent-create wizard.",
        perm: "ship.agents.read",
        args: &[],
        ret: PermissionPresetsResponse,
        run: |_args, _state, _files| {
            Ok(PermissionPresetsResponse { items: create::builtin_permission_presets() })
        },
    },
    {
        name: "agents.harnesses",
        desc: "Enumerate known harnesses (claude, openai, gemini, ...).",
        perm: "ship.agents.read",
        args: &[],
        ret: HarnessesResponse,
        run: |_args, _state, _files| {
            Ok(HarnessesResponse { items: create::builtin_harnesses() })
        },
    },
    {
        name: "agents.create_ctx",
        desc: "Convenience: templates + presets + harnesses for the create view.",
        perm: "ship.agents.read",
        args: &[],
        ret: AgentsCreateCtx,
        run: |_args, _state, _files| Ok(create::build_create_ctx()),
    },
}
