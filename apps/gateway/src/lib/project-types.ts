// Project-config types used by the gateway's GitHub → .ship/ import pipeline.
//
// These mirror the Rust shapes in `crates/core/compiler/src/{resolve.rs,
// types/{config,permissions,rule,skill}.rs}` — the gateway reads a foreign
// repo (CLAUDE.md, .mcp.json, AGENTS.md, .cursor/rules/*.mdc, .gemini/GEMINI.md,
// .ship/agents/...) and produces a `ProjectLibrary` that the compiler crate
// understands. The TS shape is the bridge between the foreign formats and
// the Rust serde model.
//
// Lives here, not in @ship/sdk, because these are compiler-domain shapes
// (project config on disk), not browser SDK shapes. The dead specta re-export
// in @ship/sdk was removed in 2026-06-20 (job/sdk-consolidate) when the
// generator was retired and only the JS bridge here remained as a consumer.

export interface Rule {
  file_name: string
  content: string
}

export interface Skill {
  id: string
  name: string
  content: string
  vars: Record<string, unknown>
  artifacts: string[]
}

export interface McpServerConfig {
  name: string
  command: string
  url: string | null
  args: string[]
  env: Record<string, string>
  timeout_secs: number | null
  codex_enabled_tools: string[]
  codex_disabled_tools: string[]
  gemini_include_tools: string[]
  gemini_exclude_tools: string[]
}

export interface ToolPermissions {
  allow: string[]
  deny: string[]
}

export interface Permissions {
  tools: ToolPermissions
}

/**
 * Library of agent config assets loaded from a foreign repo.
 * Subset of the Rust `ProjectLibrary` — gateway writes only these fields
 * to `.ship/agents/*` TOML files. Other fields (hooks, plugins, harness
 * defaults, claude_settings_extra, agents) are out of scope here; they
 * are emitted by the dedicated authoring flow, not GitHub import.
 */
export interface ProjectLibrary {
  /** Pre-existing ProjectLibrary field kept for forward-compat with any
   * downstream consumer that still reads it. The compiler crate's
   * `ProjectLibrary` does NOT carry a `modes` field, so this is a no-op
   * on the wire. */
  modes: unknown[]
  active_agent: string | null
  mcp_servers: McpServerConfig[]
  skills: Skill[]
  rules: Rule[]
  agent_profiles: unknown[]
  claude_team_agents: unknown[]
  env: Record<string, string>
  available_models: string[]
  harness_defaults: Record<string, unknown>
}
