# Harness Reference

Authoritative file paths, config schemas, and compiler support status for each agent harness.

**Rule:** When changing a `HarnessDescriptor` field, update the corresponding row here first.
Add a `<!-- verified: YYYY-MM-DD source: <url> -->` comment when re-verifying a section.

---

## Support Matrix

<!-- Last updated: 2026-03-19 -->

| Feature | Claude Code | Codex CLI | Cursor |
|---|:---:|:---:|:---:|
| MCP servers | ✅ | ✅ TOML `codex_config_patch` + per-server `enabled_tools`, `disabled_tools` | ✅ per-server `envFile` |
| Rules / context file | ✅ `CLAUDE.md` | ✅ `AGENTS.md` | ✅ `rule_files` per-file `.mdc` |
| Skills | ✅ `.claude/skills/` | ✅ `.agents/skills/` | ✅ `.cursor/skills/` |
| Permissions allow/deny | ✅ | — no compatible model | ✅ `cursor_cli_permissions` |
| Permissions ask tier | ✅ `permissions.ask` | — | — |
| defaultMode | ✅ `permissions.defaultMode` | — | — |
| additionalDirectories | ✅ `permissions.additionalDirectories` | — | — |
| Hooks | ✅ `claude_settings_patch` | — | ✅ `cursor_hooks_patch` + raw `cursor_event` bypass |
| Agent limits (cost/turns) | ✅ | — | — |
| Model override | ✅ `model` in settings patch | ✅ `model` in TOML config | — |
| MCP config path | ✅ `.mcp.json` | ✅ `.codex/config.toml` | ✅ `.cursor/mcp.json` |
| Harness-specific settings | ✅ `theme`, `autoUpdates`, `includeCoAuthoredBy`, `claude_settings_extra` | ✅ `approval_policy`, `model_reasoning_effort`, `[agents]` table, `shell_environment_policy`, `notify`, `codex_settings_extra` | ✅ `cursor_environment`, `cursor_settings_extra` |
| Subagent profiles | ✅ `.claude/agents/<id>.md` | ✅ `.codex/agents/<id>.toml` | ✅ `.cursor/agents/<id>.md` |

**Key:** ✅ implemented · ⚠️ partial/not compiled · — not implemented / out of scope

---

## Claude Code

<!-- verified: 2026-03-14 source: https://docs.anthropic.com/en/docs/claude-code -->

| Asset | Path | Format |
|---|---|---|
| Rules / context | `CLAUDE.md` | Markdown. Also `~/.claude/CLAUDE.md` (user-global) |
| Skills | `.claude/skills/<id>/SKILL.md` | Markdown + YAML frontmatter |
| Agents dir | `.claude/agents/<id>.md` | Markdown + YAML frontmatter |
| MCP config | `.mcp.json` | JSON `{ "mcpServers": { ... } }` |
| Settings | `.claude/settings.json` | JSON |

**`.claude/settings.json` — compiled fields:**
```json
{
  "model": "claude-opus-4-6",
  "permissions": {
    "allow": ["Read", "Glob"],          // omit when allow=[*] — strict allowlist otherwise
    "ask":   ["Bash(git push *)"],      // confirm before executing
    "deny":  ["Bash(rm -rf *)"],
    "defaultMode": "acceptEdits",       // "default" | "acceptEdits" | "plan" | "bypassPermissions"
    "additionalDirectories": ["../docs"]
  },
  "hooks": {
    "PreToolUse": [{ "matcher": "Bash", "command": "...", "type": "command" }]
  },
  "maxCostPerSession": 5.0,
  "maxTurns": 20
}
```
Note: `maxCostPerSession` / `maxTurns` are compiled from `AgentLimits` but are not in the official Claude Code settings docs — they may be agent-SDK-only fields. Verify if these are ignored in current Claude Code versions.

**MCP entry (stdio — no `type` field):**
```json
"server-id": { "command": "npx", "args": [...], "env": {} }
```
HTTP/SSE: `"url"` field. No `type` field — transport inferred from field presence.

---

## OpenAI Codex

<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/config-basic -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/mcp -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/skills -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/rules -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/multi-agent -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/guides/agents-md -->

| Asset | Path | Format |
|---|---|---|
| Rules / context | `AGENTS.md` | Markdown. Searched from git root → CWD, each level. Also `~/.codex/AGENTS.md` (global) |
| Approval policy | `~/.codex/config.toml` → `approval_policy` | TOML — `"suggest"`, `"auto-edit"`, or `"full-auto"`. Harness-specific setting. |
| Skills | `.agents/skills/<id>/SKILL.md` | Markdown + YAML frontmatter. Also `~/.agents/skills/` |
| Agents dir | `.codex/agents/<id>.toml` | TOML agent config |
| MCP config | `.codex/config.toml` | **TOML** `[mcp_servers.<name>]` tables |
| Settings | `~/.codex/config.toml` | TOML. Also `.codex/config.toml` (project) |
| Multi-agent roles | `.codex/config.toml` → `[agents.<name>]` | TOML. Role configs at e.g. `agents/explorer.toml` |

**MCP entry (TOML — NOT JSON):**
```toml
[mcp_servers.server-id]
command = "npx"
args = ["-y", "@org/pkg"]
startup_timeout_sec = 10   # note: _sec not _secs
tool_timeout_sec = 60
enabled = true

[mcp_servers.remote]
url = "https://api.example.com/mcp"
```

**TOML output:** `CompileOutput.codex_config_patch` contains the serialised TOML for `.codex/config.toml`.
The JSON `mcp_servers` field is still populated for internal use but Codex consumers should use `codex_config_patch`.

**Codex has no permission model compatible with Ship's allow/deny patterns.**
`approval_policy` (`suggest` / `auto-edit` / `full-auto`) is a harness-specific setting
that belongs in `harness_config`, not in the permissions model.

---

## Cursor

<!-- verified: 2026-03-14 source: https://cursor.com/docs/context/rules -->
<!-- verified: 2026-03-14 source: https://cursor.com/docs/context/skills -->

| Asset | Path | Format |
|---|---|---|
| Rules | `.cursor/rules/<name>.mdc` | Markdown + YAML frontmatter (per-rule file) ✅ compiled |
| Skills | `.cursor/skills/<id>/SKILL.md` | Markdown + YAML frontmatter |
| Agents dir | `.cursor/agents/<id>.md` | Markdown + YAML frontmatter |
| MCP config | `.cursor/mcp.json` | JSON `{ "mcpServers": { ... } }` |
| Permissions | `.cursor/cli.json` | JSON `{ "version": 1, "permissions": { ... } }` ✅ compiled |

**Cross-agent skill paths Cursor also scans:**
`.agents/skills/`, `.claude/skills/`, `.codex/skills/`, `~/.cursor/skills/`

**MCP entry (same shape as Claude — no `type` field):**
```json
"server-id": { "command": "npx", "args": [...] }
"server-id": { "url": "https://..." }   // HTTP or SSE — Cursor does not distinguish at config level
```

**`.cursor/rules/<name>.mdc` frontmatter schema** (all three fields optional):
```yaml
---
description: "When and why this rule applies"  # drives "Apply Intelligently"
globs:                                          # drives "Apply to Specific Files"
  - src/**/*.ts
alwaysApply: true                               # true = inject always (default)
---
```

Rule application modes:

| Mode | `alwaysApply` | `globs` | `description` |
|---|:---:|:---:|:---:|
| Always Apply (default) | `true` | — | — |
| Apply Intelligently | `false` | — | ✅ required |
| Apply to Specific Files | `false` | ✅ | optional |
| Apply Manually (@-mention) | `false` | — | — |

Keep rules under 500 lines. Cursor ignores rules in Inline Edit (Cmd+K) — Agent Chat only.

**Context file behavior:** Cursor uses `rule_files` (individual `.mdc`) instead of a single
`AGENTS.md`. `context_content` is always `None` for Cursor. Other harnesses concatenate all
rules into one file (`CLAUDE.md`, `AGENTS.md`) — keep individual rules concise
to avoid bloating those files.

---

## SKILL.md Frontmatter (all harnesses)

```yaml
---
name: my-skill           # required. Lowercase, hyphens. Must match directory name.
description: >           # required. Used for implicit skill matching/activation.
  Describe exactly when this skill should and should not be used.
# Optional (Cursor adds):
license: MIT
compatibility: "node >= 18"
disable-model-invocation: false   # true = requires explicit /skill-name invocation
---
```

Optional subdirectories inside a skill folder: `scripts/`, `references/`, `assets/`
Codex also supports `agents/openai.yaml` for harness-specific metadata.

---

---

## Permissions Models

Each harness uses a fundamentally different model. Claude's is the primary compilation target; others are best-effort translations.

### Claude Code — allow/deny lists

Already in `.claude/settings.json`. See Claude Code section above.
Translation is lossless — Claude Code uses the same pattern format as Ship's internal model.

### Codex CLI — no Ship-compatible permission model

<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/config-advanced -->

Codex's `approval_policy` (`suggest` / `auto-edit` / `full-auto`) controls how the agent handles tool use, but it is a global approval mode — not a per-tool allow/deny list. It belongs in `harness_config`, not in Ship's permissions model.

Ship does not compile any permission rules for Codex.

### Cursor — typed permissions (IDE agents + CLI)

<!-- verified: 2026-03-14 source: https://cursor.com/docs/cli/reference/permissions -->

Cursor permissions live in `.cursor/cli.json` (project) or `~/.cursor/cli-config.json` (global).
**These apply to both Cursor IDE agents and the Cursor CLI** — not CLI-only.

**Format:**
```json
{
  "permissions": {
    "allow": ["Shell(git *)", "Read(**/*)", "Mcp(ship:*)"],
    "deny":  ["Shell(rm -rf *)", "Mcp(*:delete*)"]
  }
}
```

**Typed pattern syntax:**
- `Shell(cmd)` — shell command (maps from Claude `Bash(cmd)`)
- `Read(glob)` — file read; `Read(**/**)` = any file
- `Write(glob)` — file write
- `WebFetch(domain)` — HTTP fetch
- `Mcp(server:tool)` — MCP tool call

**Translation from Ship's model:**

| Ship (Claude) pattern | Cursor CLI pattern |
|---|---|
| `Bash(cmd)` | `Shell(cmd)` |
| `Bash` (bare) | `Shell(*)` |
| `Read` / `Glob` / `LS` | `Read(*)` |
| `Write` / `Edit` / `MultiEdit` | `Write(*)` |
| `WebFetch` (bare) | `WebFetch(*)` |
| `WebFetch(domain)` | `WebFetch(domain)` |
| `mcp__server__tool` | `Mcp(server:tool)` |
| `mcp__*__delete*` | `Mcp(*:delete*)` |
| `*` (wildcard) | omitted (CLI default) |
| Other tools (e.g. `NotebookEdit`) | omitted (no CLI equivalent) |

**Permissive preset** (`CURSOR_PERMISSIVE_ALLOW` constant in the compiler):
```json
{ "permissions": { "allow": ["Shell(*)", "Read(*)", "Write(*)", "WebFetch(*)", "Mcp(*:*)"] } }
```
This must only be emitted when the user has **explicitly selected** a permissive mode (with UI warnings). The bare `"*"` wildcard is intentionally NOT auto-expanded — Cursor's default without a config file is interactive/prompt, which is the safe default.

**Global permissive** (`~/.cursor/cli-config.json`) must never be written unless the user explicitly selects it at the global level (with additional warnings — it grants full access to all Cursor agents on the machine).

Compiled to `cursor_cli_permissions` field in `CompileOutput`.

---

## Subagent Profiles

Ship compiles `.ship/agents/profiles/*.toml` into harness-native subagent definitions.
Each profile is emitted for every harness listed in its `harnesses` field (or all harnesses if empty).

<!-- verified: 2026-03-17 source: https://code.claude.com/docs/en/sub-agents -->
<!-- verified: 2026-03-17 source: https://cursor.com/docs/context/subagents -->
<!-- verified: 2026-03-17 source: https://developers.openai.com/codex/subagents -->

### Claude Code — `.claude/agents/<id>.md`

YAML frontmatter + Markdown body. Key fields: `name`, `description`, `model` (alias: sonnet/opus/haiku),
`permissionMode`, `disallowedTools`, `mcpServers`, `skills`.

### Cursor — `.cursor/agents/<id>.md`

YAML frontmatter + Markdown body. Key fields: `name`, `description`, `model` (`fast`/`default`/name).

### Codex CLI — `.codex/agents/<id>.toml`

TOML agent config. Key fields: `name`, `description`, `model`, `mcp_servers`.
Referenced from `[agents.<name>]` in `.codex/config.toml`.

---

## Harness-Specific Settings

These settings affect **how the agent runs** (model, sandbox, approval policy, telemetry) rather than what it can do — orthogonal to workspace modes. They require a `harness_config: HashMap<String, Value>` concept in `SpaceLibrary` (not yet implemented).

<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/config-advanced -->

### OpenAI Codex (`.codex/config.toml`)

Key settings relevant to Ship:
```toml
model = "o4-mini"
approval_policy = "auto-edit"      # "suggest" | "auto-edit" | "full-auto"
sandbox_mode = "network-disabled"  # "danger-full-internet" | "network-disabled" | "disabled"
shell_environment_policy = "inherit"
notify = true

[otel]
otlp_endpoint = "http://localhost:4317"
```
`approval_policy` maps to our workspace mode concept.

---

## Adding a New Harness

1. Fetch the harness's official docs for: rules, skills, MCP, settings, hooks
2. Add a verified section above with `<!-- verified: YYYY-MM-DD source: <url> -->` on each source
3. Add a row to the Support Matrix
4. Add a `HarnessDescriptor` entry in `crates/core/compiler/src/compile/harness.rs` (append to the `HARNESSES` static slice)
5. Add `"<id>"` to `normalize_harnesses()` in `crates/core/compiler/src/resolve.rs`
6. Add tests: harness_exists, mcp_format, skill_dir, context_file, no_settings_patch (if applicable)
