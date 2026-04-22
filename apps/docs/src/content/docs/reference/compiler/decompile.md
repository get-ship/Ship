---
title: "Decompile"
description: "Inverse direction — parse provider-native configs back into a ProjectLibrary. Per-provider parsers, detection, merge."
sidebar:
  label: "Decompile"
  order: 4
---
`crates/core/compiler/src/decompile/`. The compiler's inverse: read provider-native files on disk, produce a `ProjectLibrary` that, if re-compiled, would emit equivalent bytes.

Used by `ship convert .` to migrate existing provider configs into `.ship/` shape without hand-translation.

```
decompile/
  mod.rs           — public API, DetectedProviders, detect_providers, decompile_all
  claude.rs        — parse CLAUDE.md + .claude/ + .mcp.json
  claude_mcp.rs    — Claude MCP config helpers
  codex.rs         — parse AGENTS.md + .codex/config.toml
  cursor.rs        — parse .cursor/rules + .cursor/mcp.json
  cursor_helpers.rs
  gemini.rs        — parse GEMINI.md + .gemini/settings.json + .gemini/policies
  gemini_policies.rs
  opencode.rs      — parse AGENTS.md + opencode.json
  opencode_agents.rs
  tests.rs, roundtrip_tests.rs
```

## Public API

```rust
pub fn detect_providers(project_root: &Path) -> DetectedProviders
pub fn decompile_all(project_root: &Path) -> ProjectLibrary
pub fn decompile_claude(project_root: &Path) -> ProjectLibrary
pub fn decompile_codex(project_root: &Path) -> ProjectLibrary
pub fn decompile_cursor(project_root: &Path) -> ProjectLibrary
pub fn decompile_gemini(project_root: &Path) -> ProjectLibrary
pub fn decompile_opencode(project_root: &Path) -> ProjectLibrary
```

Each per-provider function is independent. `decompile_all` calls each and merges the results. Callers wanting a single-provider import (unlikely in practice) use the per-provider variants.

## Detection

```rust
pub struct DetectedProviders {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub cursor: bool,
    pub opencode: bool,
}
```

`detect_providers(project_root)` probes for each provider's canonical directory/file:

| Provider | Detected by |
|---|---|
| claude | `.claude/` dir exists |
| codex | `.codex/` dir exists |
| gemini | `.gemini/` dir exists |
| cursor | `.cursor/` dir exists |
| opencode | `opencode.json` exists |

`DetectedProviders::any()` returns true if at least one was found. `as_list()` returns a `Vec<&'static str>` of the detected provider IDs — used to print "detected: claude, cursor" in the convert CLI.

Detection is file-system presence only. A provider directory with garbage inside will still be detected; decompile is tolerant of missing or malformed files.

## decompile_all

Orchestrator. Calls every per-provider decompiler, merges the results. Merging is additive — if both Claude and Codex declare an MCP server with the same id, later wins but both entries are preserved if they're meaningfully different.

The caller (typically `ship convert`) writes the merged `ProjectLibrary` into `.ship/*.jsonc` files and generates `.ship/.gitignore` so future compiles own the output.

## Per-provider responsibilities

### decompile_claude

Parses:

- `CLAUDE.md` — header extraction (model, cost limits), body as rules/instructions.
- `.claude/settings.json` — `permissions.allow/deny/ask`, `hooks`, `model`, `maxCostPerSession`, `maxTurns`, `env`, `availableModels`, `theme`, etc.
- `.claude/skills/*/SKILL.md` — skill definitions with YAML frontmatter.
- `.claude/agents/*.md` — subagent profiles.
- `.mcp.json` — MCP server list. Transport inferred from field presence (`command` stdio, `url` HTTP/SSE). Helpers in `claude_mcp.rs`.

### decompile_codex

Parses:

- `AGENTS.md` — shared rules file (also used by opencode).
- `.codex/config.toml` — provider settings in TOML.
- `.agents/skills/*/SKILL.md` — shared skills dir with Gemini.
- `.codex/agents/*` — subagent profiles.

### decompile_cursor

Parses:

- `.cursor/rules/*.mdc` — rule files with frontmatter.
- `.cursor/mcp.json` — MCP config.
- `.cursor/skills/*/SKILL.md` — skills.
- `.cursor/agents/*` — subagent profiles.

No context file (Cursor doesn't use one). Permission translation is in `cursor_helpers.rs` — Cursor's permission model differs enough from the others to get its own helpers.

### decompile_gemini

Parses:

- `GEMINI.md` — context file.
- `.gemini/settings.json` — hooks (Gemini's trigger format), `mcpServers` (transport inferred from field: `command`, `url`, `httpUrl`), per-server `trust/includeTools/excludeTools/timeout`, `defaultApprovalMode`, `maxSessionTurns`, `tools.sandbox`, pass-through settings.
- `.gemini/policies/ship.toml` — `tool_policies` arrays (allow/deny/ask_user per tool kind). Helpers in `gemini_policies.rs`.
- `.agents/skills/*/SKILL.md` — shared with Codex.
- `.gemini/agents/*.md` — subagent profiles with YAML frontmatter.

### decompile_opencode

Parses:

- `opencode.json` — everything in one file: model, providers, skills list, MCP (inline), hooks. Helpers in `opencode_agents.rs`.
- `AGENTS.md` — shared with Codex.

## json_string_array helper

Shared by multiple decompile modules. Extracts a JSON array into `Vec<String>`, filtering non-strings. Returns empty vec on `None` or non-array input.

```rust
pub(super) fn json_string_array(val: Option<&Json>) -> Vec<String>
```

Used for things like permission lists, skill id lists, env var name lists — anywhere a provider config stores an array of strings.

## Tolerance vs strictness

Decompile is deliberately permissive. Missing files are OK. Malformed JSON produces empty sections rather than errors. The reasoning: users may have partial provider configs (migrated halfway, hand-edited) and `ship convert` should do the best it can rather than refuse.

Strict mode would be a separate API if the need arises. Today there's one path — best-effort.

## Roundtrip tests

`roundtrip_tests.rs` verifies the invariant:

```
ProjectLibrary → compile(provider) → CompileOutput
   → (write to disk)
   → decompile(provider) → ProjectLibrary'
   → (assert fields match)
```

A roundtrip match means the compile/decompile pair preserves user intent for that provider. Fields that can't roundtrip (because the provider's native format loses information) are documented per-provider. The tests are the forcing function that catches when a compile change breaks decompile.

## Not roundtrip-clean

Some things don't survive a roundtrip:

- **Comments in JSONC** — provider configs are JSON (not JSONC), so user comments in `.ship/ship.jsonc` are lost after a compile-decompile cycle.
- **Plugin metadata** — some plugin fields are compiler-internal and don't emit to provider configs. A decompiled `ProjectLibrary` won't have them.
- **Agent profile ordering** — provider configs don't always preserve declaration order.

Callers that need high-fidelity preservation should operate on `.ship/` directly, not round-trip through provider configs.

## Related

- [Compiler Overview](/reference/compiler/) — forward direction.
- [Compilation Pipeline](/reference/compiler/pipeline/) — stages in detail.
- [Provider Outputs](/reference/compiler/providers/) — what compile writes, per provider.
