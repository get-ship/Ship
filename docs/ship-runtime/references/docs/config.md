---
group: Runtime
order: 17
title: Config
description: .ship/ config parsing, merge order, hooks, security, catalog, MCP servers, modes, statuses.
audience: internal
---

# Config

`crates/core/ship-platform/src/config/`. Everything that reads `.ship/ship.jsonc` and related files, normalizes the content, and serves it to the rest of the runtime. Config mutations happen here too — CRUD over the same on-disk shape.

```
config/
  mod.rs              — re-exports
  types.rs            — ProjectConfig, AgentLayerConfig, HookConfig, HookTrigger, NamespaceConfig, PermissionConfig, StatusConfig, GitConfig, AiConfig
  io.rs               — get_config, get_effective_config, save_config
  crud.rs             — add_agent, add_hook, add_mcp_server, add_status, set_active_agent, etc.
  merge.rs            — merge_hooks, merge_mcp_servers, merge_modes, merge_string_lists
  project.rs          — ProjectConfig shape + defaults
  mcp.rs              — get_mcp_config, save_mcp_config (separate file: .ship/mcp.jsonc)
  modes.rs            — get_modes_config, save_modes_config
  runtime_settings.rs — get_setting, set_setting, setting_default, migrate_json_config
  git.rs              — GitConfig, generate_gitignore, is_category_committed
  catalog.rs          — CatalogEntry, CatalogKind, list_catalog, search_catalog
  security.rs         — security policy enforcement
  types.rs            — HookConfig, HookTrigger, and other config data types
  discover.rs         — discover_projects (find Ship projects on disk)
  artifact_registry.rs — artifact metadata
```

## The primary file

`.ship/ship.jsonc` (constant `PRIMARY_CONFIG_FILE`). JSONC only; legacy `config.json` / `ship.toml` paths have been removed.

Separate files siblings to `ship.jsonc`:

- `.ship/mcp.jsonc` — MCP server definitions.
- `.ship/modes.jsonc` — mode/agent profile overlays.
- `.ship/permissions.jsonc` — permission presets.
- `.ship/.gitignore` — generated, lists everything not in `git.commit`.

Collection schemas at user scope live under `~/.ship/library/collections/`; project-scope under `<project>/.ship/collections/`. See the [home-layout reference](./home-layout.md) for the full picture.

## ProjectConfig

```rust
pub struct ProjectConfig {
    pub harnesses: Vec<String>,
    pub agents: Vec<AgentLayerConfig>,
    pub hooks: Vec<HookConfig>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub permissions: PermissionConfig,
    pub statuses: Vec<StatusConfig>,
    pub namespaces: Vec<NamespaceConfig>,
    pub git: GitConfig,
    pub ai: AiConfig,
    // ... more
}
```

The merged result of project + agent-active layer + mode overlay. Produced by `get_effective_config`.

## get_config vs get_effective_config

- `get_config(project_dir)` — the raw `ProjectConfig` from `.ship/ship.jsonc` (or the fallback paths). No merging. Used for edits — CRUD operates against this unmerged view.
- `get_effective_config(project_dir)` — the merged view. Applies active agent's layer + active mode's overlay on top of the project defaults. Used for reads where "what's actually in effect" is what the caller wants.

Both take `Option<PathBuf>`. `None` means global config at `get_global_dir()`. `Some(dir)` means the project's `.ship/`.

Reads cascade: primary JSONC → `ProjectConfig::default()`.

## Merge order

In `merge.rs`:

- `merge_string_lists(base, overlay)` — dedupe-preserving concat.
- `merge_modes(base_modes, overlay_modes)` — overlay modes by `id`; new modes append.
- `merge_hooks(base, overlay)` — same pattern for hooks.
- `merge_mcp_servers(base, overlay)` — same for MCP configs.

Merge order applied in `get_effective_config`:

1. Start with `ProjectConfig` from disk.
2. If there's an active agent, apply its `AgentLayerConfig` over the project defaults.
3. If there's an active mode inside that agent, apply the mode overlay.

No deep-field merging beyond the lists. Scalar overlap wins by last-apply (overlay wins). Deep merging via `claude_settings_extra` or `gemini_settings_extra` happens at compile time, not config read time.

## HookConfig + HookTrigger

These are **agent lifecycle hooks** — the config that tells a harness to run a shell command on specific events (`PreToolUse`, `Stop`, `SessionEnd`, etc.). They are compiled into harness-native config; Ship never executes them itself. Distinct from the platform reactive triggers in `ship_platform::hooks` (`HookDef` / `.ship/hooks/*.jsonc`) — see [Hooks](/runtime/hooks/).

```rust
pub enum HookTrigger {
    SessionStart,
    UserPromptSubmit,
    PreToolUse, PermissionRequest, PostToolUse, PostToolUseFailure,
    Notification, SubagentStart, Stop, SubagentStop, PreCompact,
    BeforeTool, AfterTool, BeforeAgent, AfterAgent,
    SessionEnd, BeforeModel, AfterModel, BeforeToolSelection,
}

pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    pub matcher: Option<String>,   // glob/regex on tool name
    pub timeout_ms: Option<u64>,
    pub description: Option<String>,
    pub command: String,
}
```

19 triggers. `matcher` is a glob/regex on tool name — `"Bash"`, `"mcp__*"`, empty = all tools. `timeout_ms` caps runtime. The compiler translates these into each harness's native hook format (Claude's `PreToolUse`/`Stop`/etc.; Gemini's `BeforeTool`/`AfterTool`/etc.) — the compiler's own narrower `HookTrigger` has only 6 variants. Harnesses with different trigger vocabularies get mapped-or-dropped based on their schema.

All triggers are harness-side. Ship doesn't observe or execute them — the harness runs them inside its execution loop. See [Hooks](/runtime/hooks/) for the full breakdown, including the separate platform reactive-trigger system.

## CRUD

`crud.rs` exposes mutation helpers that preserve formatting (JSONC comments survive round-trip):

- `add_agent(ship_dir, agent)`, `remove_agent(ship_dir, agent_id)`
- `add_hook(ship_dir, hook)`, `remove_hook(ship_dir, hook_id)`, `list_hooks(ship_dir)`
- `add_mcp_server(ship_dir, server)`, `remove_mcp_server(...)`, `list_mcp_servers(...)`
- `add_status(...)`, `remove_status(...)`, `get_project_statuses(...)`
- `set_active_agent(ship_dir, agent_id)`, `get_active_agent(...)`

These load the config, mutate, write back atomically. The write path goes through `fs_util::write_atomic` (write-temp-then-rename).

## Catalog

`catalog.rs`. Discovery API for listing what's installed/available:

```rust
pub enum CatalogKind { Skill, Rule, Agent, Plugin }

pub struct CatalogEntry {
    pub kind: CatalogKind,
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: String,           // "project" | "registry/<package>"
    pub path: PathBuf,
}

pub fn list_catalog(ship_dir: &Path) -> Result<Vec<CatalogEntry>>
pub fn list_catalog_by_kind(ship_dir, kind) -> Result<Vec<CatalogEntry>>
pub fn search_catalog(ship_dir, query) -> Result<Vec<CatalogEntry>>
```

Walks `.ship/skills/`, `.ship/rules/`, agent profiles, plugin entries in the package registry. Used by Studio and MCP tools that need "what's available here."

## Security

`security.rs`. Security-policy enforcement helpers — checks like "does this MCP config declare a URL we trust," "does this hook command match a known-good shape." Called during config load when policy is strict. Not a hard sandbox; more of a lint.

## Git config

`git.rs` handles the `.ship/.gitignore` generation. `GitConfig` declares which categories commit to git by default; everything else is added to the generated gitignore.

- `generate_gitignore(ship_dir, &git_config)` — writes the gitignore.
- `is_category_committed(&git_config, "skills")` / `set_category_committed(...)` — per-category opt-in.

## Runtime settings

`runtime_settings.rs`. Key-value settings stored inside the project config's `settings` block. Used for things like active agent id, active mode, UI preferences.

- `get_setting(name, project_dir)`, `set_setting(name, value, project_dir)`.
- `get_all_settings(project_dir)` — snapshot.
- `setting_default(name)` — default value for unset settings.
- `migrate_json_config(json_path)` — one-shot migration from legacy JSON.

## Invariants

- Every write goes through `write_atomic`. No partial writes observed by readers.
- JSONC comments round-trip on CRUD. User-added comments survive config mutations.
- `get_effective_config` never touches `data.db` — it only reads JSONC files and runtime settings.
- Primary config is `ship.jsonc`.

## Related

- [Hooks](/runtime/hooks/) — compiler lifecycle hooks vs platform reactive triggers, in full.
- [Sessions](/runtime/sessions/) — fires `SessionEnd` and `Stop` hooks on session end.
- [Compiler](/compiler/) — translates hooks into harness-native formats.
