---
group: Runtime
order: 18
title: Hooks
description: Two different concepts share the name. User-authored HookConfig (lifecycle triggers) vs the RuntimeHooks trait (log-sink injection point).
audience: internal
---

# Hooks

Two things in the Ship codebase are called "hooks" and the distinction matters. This page separates them.

## 1. User hooks — `HookConfig` / `HookTrigger`

The thing most of the codebase means when it says "hook." Shell commands run on lifecycle events. Authored in `.ship/ship.jsonc`:

```jsonc
{
  "hooks": [
    {
      "id": "lint-on-stop",
      "trigger": "Stop",
      "command": "cargo clippy --workspace -- -D warnings",
      "timeout_ms": 30000
    },
    {
      "id": "block-rm",
      "trigger": "PreToolUse",
      "matcher": "Bash",
      "command": "./scripts/check-bash-command.sh",
      "timeout_ms": 5000
    }
  ]
}
```

Types live in `runtime::config::types::{HookConfig, HookTrigger}`.

### Triggers

19 variants in `HookTrigger`:

| Trigger | When it fires |
|---|---|
| `SessionStart` | Session begins. |
| `UserPromptSubmit` | User submits a prompt turn. |
| `PreToolUse` | Before a tool is invoked. |
| `PermissionRequest` | Agent requests tool permission. |
| `PostToolUse` | After a tool returns successfully. |
| `PostToolUseFailure` | After a tool errors. |
| `Notification` | Provider notification event. |
| `SubagentStart` / `SubagentStop` | Sub-agent lifecycle. |
| `Stop` | Agent's turn ends. |
| `PreCompact` | Before context compaction. |
| `BeforeTool` / `AfterTool` | Gemini-style tool events. |
| `BeforeAgent` / `AfterAgent` | Gemini-style agent events. |
| `SessionEnd` | Session closes. |
| `BeforeModel` / `AfterModel` | Gemini-style model invocation events. |
| `BeforeToolSelection` | Before the agent chooses a tool. |

Not every provider supports every trigger. The compiler maps triggers to the active provider's native schema; unsupported triggers get dropped at compile time with a warning in the emit output.

### HookConfig fields

```rust
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    pub matcher: Option<String>,  // glob/regex on tool name
    pub timeout_ms: Option<u64>,
    pub description: Option<String>,
    pub command: String,
}
```

- `matcher` — optional glob/regex. `"Bash"` = exactly Bash. `"mcp__*"` = any MCP tool. Empty/absent = all tools.
- `timeout_ms` — hard cap on command runtime. Exceeded = kill + fail.
- `description` — human-readable intent. Surfaced in editor UX and in the exported provider config as a comment where supported.
- `command` — shell command. Runs with the workspace's env + Ship-specific vars (`SHIP_SESSION_ID`, `SHIP_TOOL_NAME`, etc.) set.

### Where they fire

- **Runtime-side**: `session_lifecycle::run_post_session_hooks` fires `SessionEnd` and `Stop` hooks when `end_workspace_session` runs. These go through the runtime synchronously.
- **Provider-side**: most triggers (`PreToolUse`, `PostToolUse`, `Notification`, etc.) fire inside the provider's execution loop. The compiler emits them into provider-native hook config (e.g. `.claude/settings.json:hooks`), and the provider (Claude Code, Gemini CLI, etc.) runs them directly. Ship doesn't observe these — they run in the agent's process.

### CRUD

`runtime::config`:

- `add_hook(ship_dir, hook)` — append a hook. Writes back atomically.
- `remove_hook(ship_dir, hook_id)` — by id.
- `list_hooks(ship_dir)` — current hooks.

### Example: commit-gating on Stop

```jsonc
{
  "id": "pre-commit-lint",
  "trigger": "Stop",
  "command": "cargo fmt --check && cargo clippy -- -D warnings",
  "timeout_ms": 60000,
  "description": "Gate: block session close if lint fails"
}
```

A non-zero exit from the command signals hook failure. What the agent sees varies by provider (some refuse to exit; some pass the error through). The behavior is deliberately provider-dependent — Ship declares the hook; the provider enforces.

## 2. Runtime hooks — the `RuntimeHooks` trait

The smaller, less-used thing. Lives in `runtime::config::hooks`:

```rust
pub trait RuntimeHooks: Send + Sync {
    fn append_log(&self, ship_dir: &Path, action: &str, details: &str) -> Result<()>;
}

pub struct DefaultRuntimeHooks;

impl RuntimeHooks for DefaultRuntimeHooks {
    fn append_log(&self, ship_dir: &Path, action: &str, details: &str) -> Result<()> {
        log_action(ship_dir, action, details)
    }
}
```

One method. One default implementation. Purpose: let app layers (shipd, tests, alternate hosts) inject a custom sink for the project action log. `log_action` writes to the file-based project log at `.ship/log/`; a test could swap in an in-memory sink.

Most runtime code calls `crate::log_action(ship_dir, action, details)` directly, bypassing the trait. The trait is dormant — ready if an app layer needs to redirect logging, unused in production today.

## Why the naming collision exists

`HookConfig` is the user-facing lifecycle-hook vocabulary borrowed from Claude/Gemini — users know "hooks" as "shell commands on events." `RuntimeHooks` is the internal extension-point terminology common to embedder-friendly Rust crates. Both are entrenched.

When reading code, disambiguate by context:

- `HookConfig`, `HookTrigger`, "the hooks list," `.ship/ship.jsonc` → user lifecycle hooks.
- `RuntimeHooks`, `DefaultRuntimeHooks`, `append_log` → embedder trait.

When writing code, prefer `log_action` for project log writes unless you're specifically building a swappable embedding.

## Invariants

- User hooks are data. They live in `.ship/ship.jsonc` and compile into provider config. Runtime doesn't execute most of them — providers do.
- `SessionEnd` / `Stop` hooks run synchronously during `end_workspace_session`. Long-running hook commands block session close.
- `RuntimeHooks::append_log` has one default impl and no known alternates. Don't design against its plurality today.

## Related

- [Config](/runtime/config/) — where `HookConfig` lives and the file format.
- [Sessions](/runtime/sessions/) — post-session hook firing.
- [Compiler](/compiler/) — hook translation per provider.
