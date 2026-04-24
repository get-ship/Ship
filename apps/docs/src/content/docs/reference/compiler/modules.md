---
title: "Compiler Internals"
description: "Vars (MiniJinja), permissions enforcement, schemas, agent parser, per-provider builders. The modules that make the compiler more than a JSON rewriter."
sidebar:
  label: "Compiler Internals"
  order: 6
---
The transformation pipeline is documented elsewhere. This page covers the submodules that handle specific concerns inside `compile` and `resolve`.

## Template variables (`vars.rs`)

Single function: `resolve_template(content, vars) -> String`. MiniJinja (Jinja2-compatible) template engine, called at compile time on skill markdown bodies.

### Surface

```rust
pub fn resolve_template(content: &str, vars: &HashMap<String, Value>) -> String
```

### Supported constructs

| Construct | Meaning |
|---|---|
| `{{ var }}` | Scalar substitution. |
| `{{ obj.field }}` | Dot-path into JSON object. |
| `{% if var %}…{% endif %}` | Truthy conditional. |
| `{% if var == "val" %}…{% else %}…{% endif %}` | Equality + else. |
| `{% for x in arr %}…{% endfor %}` | Iterate array. |

### Configuration

- `UndefinedBehavior::Chainable` — undefined variables and attribute access on undefined return empty string, never errors. Skills degrade gracefully when state hasn't been configured.
- **No loader set** — `{% include %}` and `{% extends %}` are disabled. Skill templates are self-contained; cross-file composition is not supported.

### Fast path

If `content` contains neither `{{` nor `{%`, the function returns the input unchanged — no MiniJinja environment constructed. Hot path when most skills don't use templating.

### Error handling

Template syntax errors emit a `warning:` line to stderr and fall back to returning the original content. Better to deliver un-interpolated text than to refuse the compile. The warning is the signal to fix the template.

## Permissions (`permissions.rs`)

Enforces that every tool a skill declares in its YAML frontmatter `allowed-tools` is granted by the agent referencing the skill.

### Contract

> Every tool in `SkillPermissions.allowed_tools` must be in `agent.permissions.allow` OR the agent has `"*"` in its allow list.

No transitive escalation — a skill only declares tools it directly uses. Skill A referencing skill B doesn't inherit B's tool needs; each skill's `allowed-tools` is a self-declaration.

### Types

```rust
pub struct SkillPermissions {
    pub allowed_tools: Vec<String>,
}

pub struct PermissionViolation {
    pub skill_name: String,
    pub missing_tool: String,
    // ... location info
}
```

### Flow

1. For each skill in `agent.skills`, parse `allowed-tools` from SKILL.md frontmatter (space-delimited string).
2. Compare each tool against `agent.permissions.allow`.
3. Collect violations. Hard error if any — compile fails.

This runs during `resolve`, before the pure `compile` step. Misconfigured agents are rejected at resolve time, not at emit time, so the error message carries skill names and tool names clearly.

## Schemas (`schemas.rs`)

Per-provider upstream schema URLs. Used in three places:

1. **Ship JSON schemas** — `$ref` in `provider_settings` / `provider_defaults` fields so tools like VSCode's JSON language server autocomplete provider fields against the real schemas.
2. **Web app** — CodeMirror autocomplete for `provider_settings`.
3. **CI** — drift detection (`schema_drift_tests.rs`). Periodic checks that the upstream schemas haven't moved.

### Constants

```rust
pub const CLAUDE_SCHEMA_URL:    &str        = "https://www.schemastore.org/claude-code-settings.json";
pub const CODEX_SCHEMA_URL:     &str        = "https://raw.githubusercontent.com/openai/codex/...";
pub const GEMINI_SCHEMA_URL:    &str        = "https://raw.githubusercontent.com/google-gemini/...";
pub const OPENCODE_SCHEMA_URL:  &str        = "https://opencode.ai/config.json";
pub const CURSOR_SCHEMA_URL:    Option<&str>= None;
```

Cursor has no published schema — config is file-based (`.cursor/rules/*.mdc`, `.cursor/mcp.json`, etc.) with no JSON schema to validate against.

### Public API

```rust
pub const PROVIDER_SCHEMAS: &[(&str, Option<&str>)] = &[ ... ];
pub fn schema_url(provider_id: &str) -> Option<&'static str>;
pub const PROVIDER_MANAGED_KEYS: ...   // fields Ship controls per provider
pub fn managed_keys(provider_id: &str) -> &'static [&'static str];
```

`PROVIDER_MANAGED_KEYS` lists keys Ship owns in each provider's config — writers outside Ship should leave those fields alone. `managed_keys(provider)` is the runtime lookup.

### Drift detection

`schema_drift_tests.rs` fetches each URL and fails if Ship's `provider_defaults` / `provider_settings` drift from what the upstream schemas declare. Not run on every commit — it requires network. Run in a scheduled CI job.

## Agent parser (`agent_parser.rs`)

Parses `.claude/agents/*.md` and `.gemini/agents/*.md` into `AgentDef` records. The largest single file in the compiler at 663 lines because it handles provider-specific YAML frontmatter quirks.

### Input formats

- **Claude team agents** (`.claude/agents/*.md`) — markdown with YAML frontmatter declaring name, description, tools, model.
- **Gemini agents** (`.gemini/agents/*.md`) — markdown with YAML frontmatter; different field names than Claude.
- **Legacy Claude team agents** (`agents/teams/claude/*.md`) — pre-standard format, still read for backward compatibility.

Each format produces the same `AgentDef` struct internally, hiding the source format from downstream code.

### Output

```rust
pub struct AgentDef {
    pub name: String,
    pub description: Option<String>,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub system_prompt: String,  // markdown body
    // ... more fields
}
```

Callers (resolve, decompile) don't care which provider authored the file — they see `AgentDef` and move on.

## Per-provider builders (`compile/claude.rs`, etc.)

Inside `compile/`, one file per provider emits the provider-specific bytes:

- `claude.rs` — builds CLAUDE.md, .claude/settings.json patch, .claude/skills/ entries, .claude/agents/ files.
- `codex.rs` — AGENTS.md, .codex/config.toml, .agents/skills/ (shared with Gemini), .codex/agents/.
- `cursor.rs` — .cursor/rules/*.mdc, .cursor/mcp.json, .cursor/skills/, .cursor/agents/. No context file.
- `gemini.rs` — GEMINI.md, .gemini/settings.json (with hooks in Gemini trigger format), .gemini/policies/ship.toml, .agents/skills/ (shared), .gemini/agents/.
- `opencode.rs` — opencode.json (everything in one file).

### Shared modules

- `compile/agents.rs` — `compile_agent_profiles(resolved, provider) -> Vec<AgentFile>`. Translates `AgentDef` into provider-native agent files.
- `compile/context.rs` — the shared context-file generator. Emits CLAUDE.md / AGENTS.md / GEMINI.md from the same `ResolvedConfig.rules` input; provider decides filename.
- `compile/skills.rs` — skill-to-provider-path mapping plus YAML frontmatter generation.
- `compile/mcp.rs` — MCP server serialization. Generic shape with per-provider adjustments (Gemini puts them inside `mcpServers`, Cursor inside `.cursor/mcp.json`, etc.).
- `compile/plugins.rs` — plugin manifest filtered by provider.
- `compile/provider.rs` — `ProviderDescriptor` + feature flags (`supports_mcp`, `supports_hooks`, `supports_tool_permissions`, `supports_memory`).

### Feature flags

Each provider declares capabilities in its `ProviderDescriptor`:

```rust
pub struct ProviderFeatureFlags {
    pub supports_mcp: bool,
    pub supports_hooks: bool,
    pub supports_tool_permissions: bool,
    pub supports_memory: bool,
    pub server_mode: bool,
    // ...
}
```

Compile steps check these before emitting. If `supports_hooks == false` for a provider, hooks configured in Ship config are dropped from that provider's output (with a warning in the compile result). This is the path that lets `.ship/` declare intent and each provider emit only what it can support.

## Related

- [Compilation Pipeline](/reference/compiler/pipeline/) — the three stages in order.
- [Provider Outputs](/reference/compiler/providers/) — what each provider writes.
- [Decompile](/reference/compiler/decompile/) — inverse direction.
- [Manifest & Lockfile](/reference/compiler/manifest-lockfile/) — package identity and versioned dependencies.
