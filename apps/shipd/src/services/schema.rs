//! `ship.*` op catalog — the single source of truth for what view
//! templates (`ship.<domain>.<op>`), `window.ship.*` codegen, and MCP
//! tool auto-registration all read.
//!
//! Served at `GET /api/ship/schema`. Each op carries enough metadata
//! for lint, autocomplete, and permission mapping:
//!
//! - `name`         — dotted op path (e.g. `agents.detail`).
//! - `description`  — one-line human summary.
//! - `args`         — list of argument descriptors with kwarg name + kind.
//! - `returns`      — string tag naming the return shape.
//! - `permission`   — capability required to call. Metadata-only today;
//!                    enforcement lands with the caller-identity model.
//! - `sensitive`    — flag that lint / UI can warn on (payloads may
//!                    contain secrets, git content, etc.).
//!
//! Arg / return kinds are string tags today — keeps the surface light
//! (no JSON schema per op). Autocomplete + MCP tool bindings only need
//! name-shape matching; full schemas land when there's a concrete need.

use axum::Json;
use serde::Serialize;

use crate::rest_api::MeshResponse;

/// `GET /api/ship/schema` — catalog of every `ship.<domain>.<op>` the
/// registry exposes. Consumers:
/// - `@ship/shell` TS SDK: codegen typed `window.ship.*` methods
/// - Lint: flag template calls that don't match a known op
/// - MCP server: auto-register ops as tools
/// - Third-party view authors: editor autocomplete via `/api/ship/schema`
pub async fn serve_schema() -> Json<MeshResponse> {
    Json(MeshResponse {
        ok: true,
        data: serde_json::json!({ "ops": OPS }),
    })
}

#[derive(Serialize, Clone, Copy)]
pub struct OpDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    pub returns: &'static str,
    pub permission: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub sensitive: bool,
}

#[derive(Serialize, Clone, Copy)]
pub struct ArgDescriptor {
    pub name: &'static str,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub required: bool,
}

fn is_false(v: &bool) -> bool { !v }

const PROJECT_SCOPE: ArgDescriptor = ArgDescriptor { name: "scope", kind: "string?", required: false };

/// The full registry op catalog. Order mirrors `services::registry::call`'s
/// match arms so drift between the two is visible in diff.
pub const OPS: &[OpDescriptor] = &[
    // ── agents ────────────────────────────────────────────────────
    OpDescriptor {
        name: "agents.list",
        description: "List all agents as minimal IndexEntry rows.",
        args: &[],
        returns: "{items: [IndexEntry]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.summaries",
        description: "List all agents with display metadata (label, providers, tags).",
        args: &[],
        returns: "{items: [AgentSummary]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.detail",
        description: "Full view-shaped agent detail — catalogs, permissions, providers, tags.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "AgentDetail",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.catalogs",
        description: "Catalog rows for the agents list (providers, skills, etc.).",
        args: &[],
        returns: "AgentCatalogs",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.get",
        description: "Raw agent doc from disk at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, doc}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.wire",
        description: "Graph payload (nodes + edges) for the agent-wire canvas.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "AgentWireCtx",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.settings",
        description: "Tier list + catalog rows for the per-agent settings pane.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "AgentSettingsCtx",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.templates",
        description: "Starter templates for the agent-create wizard.",
        args: &[],
        returns: "{items: [AgentTemplate]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.permission_presets",
        description: "Permission presets for the agent-create wizard.",
        args: &[],
        returns: "{items: [PermissionPreset]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.providers",
        description: "Enumerate known providers (claude, openai, gemini, ...).",
        args: &[],
        returns: "{items: [string]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.create_ctx",
        description: "Convenience: templates + presets + providers for the create view.",
        args: &[],
        returns: "AgentsCreateCtx",
        permission: "ship.agents.read",
        sensitive: false,
    },

    // ── skills ────────────────────────────────────────────────────
    OpDescriptor {
        name: "skills.list",
        description: "List all skills as minimal IndexEntry rows.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.summaries",
        description: "Rail shape: filtered + grouped + kind counts + tags.",
        args: &[
            ArgDescriptor { name: "q", kind: "string?", required: false },
            ArgDescriptor { name: "kind", kind: "string?", required: false },
            ArgDescriptor { name: "tag", kind: "string?", required: false },
        ],
        returns: "SkillsRailCtx",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.detail",
        description: "Full view-shaped skill detail — authors, variables, files.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "SkillDetail",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.file",
        description: "HTML-escaped file content within a skill dir.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "path", kind: "string", required: true },
        ],
        returns: "{path, name, ext, content}",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.get",
        description: "Raw skill detail at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "SkillDetail",
        permission: "ship.skills.read", sensitive: false,
    },

    // ── rules ─────────────────────────────────────────────────────
    OpDescriptor {
        name: "rules.list",
        description: "List markdown rules across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.rules.read", sensitive: false,
    },
    OpDescriptor {
        name: "rules.get",
        description: "Raw rule body at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, body}",
        permission: "ship.rules.read", sensitive: false,
    },

    // ── mcp ───────────────────────────────────────────────────────
    OpDescriptor {
        name: "mcp.list",
        description: "List MCP server entries across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.mcp.read", sensitive: false,
    },
    OpDescriptor {
        name: "mcp.get",
        description: "Raw MCP entry at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, entry}",
        permission: "ship.mcp.read", sensitive: false,
    },

    // ── permissions ──────────────────────────────────────────────
    OpDescriptor {
        name: "permissions.list",
        description: "List permission presets across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.permissions.read", sensitive: false,
    },
    OpDescriptor {
        name: "permissions.get",
        description: "Raw permission preset at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, entry}",
        permission: "ship.permissions.read", sensitive: false,
    },
    OpDescriptor {
        name: "permissions.tool_patterns",
        description: "Enumerate tool-pattern capability names (for agent builders).",
        args: &[], returns: "{patterns: [ToolPattern]}",
        permission: "ship.permissions.read", sensitive: false,
    },

    // ── pipelines ─────────────────────────────────────────────────
    OpDescriptor {
        name: "pipelines.list",
        description: "List pipeline definitions across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.get",
        description: "Raw pipeline doc at the requested scope.",
        args: &[ArgDescriptor { name: "name", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{name, scope, format, doc}",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.summaries",
        description: "Index-view shape: defs + active/history + pills + agents.",
        args: &[], returns: "PipelinesSummariesCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.focus",
        description: "Per-pipeline detail with active + other runs precomputed.",
        args: &[ArgDescriptor { name: "name", kind: "string", required: true }],
        returns: "PipelineFocusCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.run",
        description: "Run detail with steps + timeline.",
        args: &[ArgDescriptor { name: "slug", kind: "string", required: true }],
        returns: "RunDetailCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.history",
        description: "Paginated run history with pipeline filter options.",
        args: &[
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "offset", kind: "number?", required: false },
            ArgDescriptor { name: "pipeline", kind: "string?", required: false },
        ],
        returns: "HistoryCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },

    // ── events ────────────────────────────────────────────────────
    OpDescriptor {
        name: "events.list",
        description: "Kernel events with filters, grouped by entity.",
        args: &[
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "event_type", kind: "string?", required: false },
            ArgDescriptor { name: "actor", kind: "string?", required: false },
            ArgDescriptor { name: "entity", kind: "string?", required: false },
            ArgDescriptor { name: "q", kind: "string?", required: false },
        ],
        returns: "{groups, types, actors}",
        permission: "ship.events.read", sensitive: true,
    },

    // ── specs ─────────────────────────────────────────────────────
    OpDescriptor {
        name: "specs.list",
        description: "Raw spec records from the DB.",
        args: &[], returns: "{items: [Spec]}",
        permission: "ship.specs.read", sensitive: false,
    },
    OpDescriptor {
        name: "specs.get",
        description: "Full spec record + body.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "{spec, body}",
        permission: "ship.specs.read", sensitive: false,
    },
    OpDescriptor {
        name: "specs.summaries",
        description: "Rail shape: grouped + counts + pipelines.",
        args: &[
            ArgDescriptor { name: "q", kind: "string?", required: false },
            ArgDescriptor { name: "selected", kind: "string?", required: false },
        ],
        returns: "SpecsSummariesCtx",
        permission: "ship.specs.read", sensitive: false,
    },
    OpDescriptor {
        name: "specs.detail",
        description: "Full view-shaped spec detail: parsed sections, launch ctx.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "SpecDetail",
        permission: "ship.specs.read", sensitive: false,
    },

    // ── git ───────────────────────────────────────────────────────
    OpDescriptor {
        name: "git.status",
        description: "Working tree + index status buckets.",
        args: &[], returns: "GitStatusBuckets",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.log",
        description: "Commit log up to `limit` entries.",
        args: &[ArgDescriptor { name: "limit", kind: "number?", required: false }],
        returns: "{commits: [CommitSummary]}",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.branches",
        description: "Local + remote branches with ahead/behind vs upstream.",
        args: &[], returns: "{branches, ahead, behind}",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.overview",
        description: "One-call shape for the git view's default route.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "limit", kind: "number?", required: false },
        ],
        returns: "GitCtx",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.diff",
        description: "Diff for a commit or 'WORKING'.",
        args: &[
            ArgDescriptor { name: "commit", kind: "string", required: true },
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
        ],
        returns: "DiffView",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.annotations",
        description: "AnnotationsByLine for the active diff file.",
        args: &[
            ArgDescriptor { name: "file", kind: "string", required: true },
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
        ],
        returns: "AnnotationsByLine",
        permission: "ship.git.read", sensitive: false,
    },

    // ── settings / dashboard / collections / chat ────────────────
    OpDescriptor {
        name: "settings.state",
        description: "Providers, defaults, terminal, about — everything the settings view renders.",
        args: &[], returns: "SettingsCtx",
        permission: "ship.settings.read", sensitive: false,
    },
    OpDescriptor {
        name: "dashboard.state",
        description: "Aggregate stats + recent runs + events + agents + collections.",
        args: &[], returns: "DashboardCtx",
        permission: "ship.dashboard.read", sensitive: false,
    },
    OpDescriptor {
        name: "collections.state",
        description: "Collections list + active schema + items (tier-3 app data).",
        args: &[ArgDescriptor { name: "active", kind: "string?", required: false }],
        returns: "CollectionsCtx",
        permission: "ship.collections.read", sensitive: false,
    },
    OpDescriptor {
        name: "chat.state",
        description: "Recent messages for the chat drawer, scoped by workspace.",
        args: &[ArgDescriptor { name: "workspace_id", kind: "string?", required: false }],
        returns: "{workspace_id, messages, total}",
        permission: "ship.chat.read", sensitive: true,
    },
];
