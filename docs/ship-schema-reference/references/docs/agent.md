---
group: Schema Reference
title: Agent Profile
section: reference
order: 3
---

# Agent Profile — agents/*.jsonc

**Location**: `.ship/agents/<id>.jsonc`
**Schema**: `agent.schema.json`

An agent profile composes skills, MCP servers, permissions, rules, and provider settings into a single deployable identity.

## Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent` | object | yes | Identity and metadata |
| `model` | string | no | Model override, compiled to provider's model field |
| `skills` | object | no | Skill activation |
| `mcp` | object | no | MCP server activation |
| `plugins` | object | no | Plugin installation intent |
| `permissions` | object | no | Permission overrides on top of preset |
| `rules` | object | no | Inline rules appended to context |
| `provider_settings` | object | no | Per-provider overrides (deep-merged on project defaults) |

## agent (identity)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Machine identifier. Must match filename stem. Pattern: `^[a-z0-9-]+$` |
| `name` | string | yes | Human-readable name |
| `version` | string | no | Semver version. Default: `"0.1.0"` |
| `description` | string | no | What this agent does — shown in Studio UI |
| `tags` | string[] | no | Searchable tags for registry discovery |
| `icon` | string | no | Emoji or icon name |
| `providers` | string[] | no | Provider targets. Overrides project-level `providers`. Values: `claude`, `cursor`, `codex`, `gemini`, `opencode` |

## skills

| Field | Type | Description |
|-------|------|-------------|
| `refs` | string[] | Skill IDs to activate. Local (`tdd`) or namespaced (`github.com/owner/repo/skill-name`) |

## mcp

| Field | Type | Description |
|-------|------|-------------|
| `servers` | string[] | MCP server IDs from `.ship/mcp.jsonc` to activate |

## plugins

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `install` | string[] | — | Plugin IDs, e.g. `rust-analyzer-lsp@claude-plugins-official` |
| `scope` | `"project"` \| `"user"` | `"project"` | Installation scope |

## permissions

Merged on top of the named preset from `permissions.jsonc`.

| Field | Type | Description |
|-------|------|-------------|
| `preset` | string | Preset name from `permissions.jsonc` |
| `tools_allow` | ToolPattern[] | Additional tools to allow |
| `tools_deny` | ToolPattern[] | Additional tools to deny (takes precedence) |
| `tools_ask` | ToolPattern[] | Additional tools requiring confirmation |

**ToolPattern syntax**: bare name (`Read`, `Bash`), parameterized (`Bash(git *)`), MCP tool (`mcp__ship__create_job`), or glob (`mcp__*__delete*`).

## rules

| Field | Type | Description |
|-------|------|-------------|
| `inline` | string | Rules injected into compiled context after `.ship/rules/*.md` files. Use `\n` for line breaks. |

## provider_settings

Per-provider overrides deep-merged on top of project-level `provider_defaults`. Ship-managed keys (permissions, hooks, model, env) must be set via Ship's own fields.

Supported keys: `claude`, `codex`, `gemini`, `cursor`, `opencode`. Each validates against the upstream provider's schema with Ship-managed keys excluded.

## Example

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json",
  "agent": {
    "id": "backend",
    "name": "Backend Engineer",
    "version": "0.1.0",
    "description": "Rust backend specialist",
    "providers": ["claude", "gemini"]
  },
  "model": "opus",
  "skills": {
    "refs": ["tdd", "code-review"]
  },
  "mcp": {
    "servers": ["ship", "github"]
  },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(git push --force*)"]
  },
  "rules": {
    "inline": "Your domain is Rust backend code.\nRun cargo test before marking work done."
  }
}
```
