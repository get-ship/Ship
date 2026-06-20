// FILE_CAP_OVERRIDE: Single `pub const COMMANDS: &[CommandDescriptor]` array.
// Rust const slices cannot be split across files and concatenated statically;
// a runtime LazyLock<Vec<_>> would change the public type, breaking gen-ops
// and the gate/policy tests that iterate `COMMANDS` as `&'static [_]`.
//! The mutation (command) catalog — every write op the daemon exposes.

use super::types::{ArgDescriptor, CommandDescriptor};

/// The mutation catalog. Consumed by the schema route (`/api/v1/ship/schema`),
/// xtask's proto description back-port, and the gate conformance tests that
/// pin each entry's capability + blast against the proto descriptor.
pub const COMMANDS: &[CommandDescriptor] = &[
    // ── pipeline lifecycle ────────────────────────────────────────
    CommandDescriptor {
        name: "pipeline.update_status",
        description: "Advance a pipeline run's lifecycle/gate state (the agent's own run).",
        args: &[
            ArgDescriptor { name: "pipeline_run_id", kind: "string", required: true },
            ArgDescriptor { name: "status", kind: "string", required: true },
            ArgDescriptor { name: "worktree", kind: "string?", required: false },
            ArgDescriptor { name: "pid", kind: "number?", required: false },
            ArgDescriptor { name: "gate_agent", kind: "string?", required: false },
            ArgDescriptor { name: "blocker", kind: "string?", required: false },
            ArgDescriptor { name: "error", kind: "string?", required: false },
        ],
        permission: "ship.runtime.write",
        blast_radius: "C1_WORKSPACE",
        response_type: "",
        sensitive: false,
    },
    CommandDescriptor {
        name: "pipeline.step_done",
        description: "Agent declares the current pipeline step complete.",
        args: &[
            ArgDescriptor { name: "slug", kind: "string", required: true },
            ArgDescriptor { name: "summary", kind: "string?", required: false },
            ArgDescriptor { name: "outputs", kind: "map", required: false },
        ],
        permission: "ship.runtime.write",
        blast_radius: "C1_WORKSPACE",
        response_type: "",
        sensitive: false,
    },
    // ── mcp (external MCP-server config; reads are mcp.list/get) ───
    // Restricted (ship-authoring): registering a server adds tools agents can
    // call. JSON-valued args travel as JSON-string args (server_json/patch_json).
    // ── permissions (preset config; reads are permissions.list/get) ─
    // C2 (operator-only): a preset IS the security policy governing agents, so
    // agents never edit it. JSON args travel as JSON-string (preset_json/patch_json).
    // ── skills (authoring; restricted ship-authoring) ─────────────
    CommandDescriptor {
        name: "skills.create",
        description: "Create a new skill (optional initial frontmatter + body).",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
            ArgDescriptor { name: "frontmatter_json", kind: "string?", required: false },
            ArgDescriptor { name: "body", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.delete",
        description: "Delete a skill (scope omitted walks project → user).",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.rename",
        description: "Rename a skill's id.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "new_id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.duplicate",
        description: "Copy a skill to a new id.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "new_id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.write_frontmatter",
        description: "Replace a skill's frontmatter (YAML/JSON object).",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "frontmatter_json", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.write_body",
        description: "Replace a skill's markdown body.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "body", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.move_scope",
        description: "Move a skill between project and user scope.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "from", kind: "string", required: true },
            ArgDescriptor { name: "to", kind: "string", required: true },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.write_file",
        description: "Write a UTF-8 text file inside a skill dir.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "path", kind: "string", required: true },
            ArgDescriptor { name: "content", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.delete_file",
        description: "Delete a file inside a skill dir.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "path", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.set_var",
        description: "Set a KV variable (value_json) on a skill.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "key", kind: "string", required: true },
            ArgDescriptor { name: "value_json", kind: "string", required: true },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "",
        sensitive: false,
    },
    CommandDescriptor {
        name: "skills.delete_var",
        description: "Delete a KV variable from a skill.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "key", kind: "string", required: true },
        ],
        permission: "ship.skills.write",
        blast_radius: "C2_ELEVATED",
        response_type: "",
        sensitive: false,
    },
    // ── agents (profile authoring; restricted ship-authoring) ─────
    CommandDescriptor {
        name: "agents.create",
        description: "Create a new agent profile from a doc.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "doc_json", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.put",
        description: "Replace an agent profile's doc (use agents.create for net-new).",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "doc_json", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.patch",
        description: "Apply RFC-6902 ops to an agent profile's doc.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "patch_json", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.delete",
        description: "Delete an agent profile (scope omitted walks project → user).",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.rename",
        description: "Rename an agent profile's id.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "new_id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.duplicate",
        description: "Copy an agent profile to a new id.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "new_id", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
            ArgDescriptor { name: "to", kind: "string?", required: false },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.move_scope",
        description: "Move an agent profile between project and user scope.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "from", kind: "string", required: true },
            ArgDescriptor { name: "to", kind: "string", required: true },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    // ── pipelines (definition CRUD; reads are pipelines.list/get) ──
    CommandDescriptor {
        name: "pipelines.put",
        description: "Create or replace a pipeline definition (by name).",
        args: &[
            ArgDescriptor { name: "name", kind: "string", required: true },
            ArgDescriptor { name: "doc_json", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.pipelines.write",
        blast_radius: "C1_WORKSPACE",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "pipelines.patch",
        description: "Apply RFC-6902 ops to a pipeline definition.",
        args: &[
            ArgDescriptor { name: "name", kind: "string", required: true },
            ArgDescriptor { name: "patch_json", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.pipelines.write",
        blast_radius: "C1_WORKSPACE",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    CommandDescriptor {
        name: "pipelines.delete",
        description: "Delete a pipeline definition (scope omitted walks project → user).",
        args: &[
            ArgDescriptor { name: "name", kind: "string", required: true },
            ArgDescriptor { name: "scope", kind: "string?", required: false },
        ],
        permission: "ship.pipelines.write",
        blast_radius: "C1_WORKSPACE",
        response_type: "ScopedMutation",
        sensitive: false,
    },
    // ── settings (runtime config; reads are settings.list/get) ────
    CommandDescriptor {
        name: "settings.set",
        description: "Set a runtime setting (value is arbitrary JSON).",
        args: &[
            ArgDescriptor { name: "key", kind: "string", required: true },
            ArgDescriptor { name: "value_json", kind: "string", required: true },
        ],
        permission: "ship.settings.write",
        blast_radius: "C1_WORKSPACE",
        response_type: "SettingValue",
        sensitive: false,
    },
    // space.setting.set + space.window.close retired in the space cutover —
    // the per-space override write is now SpaceService.SetSetting (ship-space)
    // and the window close is DeviceService.CloseWindow (ship-device), both
    // self-gated wires that bypass the command dispatch table.
    // project.add + project.activate retired — SpaceService.Add / .Activate
    // (ship-space) are the SSOT; they route through the ConnectRPC wire.
    // ── agent control ─────────────────────────────────────────────
    CommandDescriptor {
        name: "agents.stop",
        description: "Stop a running agent by slug.",
        args: &[ArgDescriptor { name: "slug", kind: "string", required: true }],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.grant",
        description: "Grant restricted capabilities to an agent (operator approval). \
                      Upgrades the agent's live sessions; the next gated call passes. \
                      C3 — operator-only, the human-in-the-loop approval point.",
        args: &[
            ArgDescriptor { name: "agent", kind: "string", required: true },
            ArgDescriptor { name: "grants", kind: "string[]", required: true },
        ],
        permission: "ship.agents.grant",
        blast_radius: "C3_CRITICAL",
        response_type: "",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.steer",
        description: "Inject a message into a running agent.",
        args: &[
            ArgDescriptor { name: "slug", kind: "string", required: true },
            ArgDescriptor { name: "message", kind: "string", required: true },
        ],
        permission: "ship.agents.write",
        blast_radius: "C2_ELEVATED",
        response_type: "",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agents.dispatch",
        description: "Dispatch a single agent session (the canonical agent command).",
        args: &[
            ArgDescriptor { name: "agent", kind: "string", required: true },
            ArgDescriptor { name: "spec", kind: "string?", required: false },
            ArgDescriptor { name: "prompt", kind: "string?", required: false },
            ArgDescriptor { name: "resume", kind: "bool?", required: false },
            ArgDescriptor { name: "harness", kind: "string?", required: false },
            ArgDescriptor { name: "model", kind: "string?", required: false },
            ArgDescriptor { name: "open_tab", kind: "bool?", required: false },
            ArgDescriptor { name: "invocation", kind: "string?", required: false },
            ArgDescriptor { name: "keepalive", kind: "bool?", required: false },
            ArgDescriptor { name: "workspace", kind: "string?", required: false },
            ArgDescriptor { name: "slug", kind: "string?", required: false },
            ArgDescriptor { name: "branch", kind: "string?", required: false },
            ArgDescriptor { name: "source_spec_path", kind: "string?", required: false },
            ArgDescriptor { name: "vars", kind: "map", required: false },
        ],
        permission: "ship.runtime.dispatch",
        blast_radius: "C2_ELEVATED",
        response_type: "",
        sensitive: false,
    },
];
// create.surface, annotation.post, feedback.post removed:
// rehomed to UiService::Scaffold, AgentsService::PostAnnotation,
// AgentsService::PostFeedback (crate-owned wires, one rpc story).
