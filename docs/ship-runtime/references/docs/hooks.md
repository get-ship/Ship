---
group: Runtime
order: 18
title: Hooks
description: Two different concepts share the name. Compiler lifecycle hooks (HookConfig, compiled into harness config) vs platform reactive triggers (HookDef, .ship/triggers/*.jsonc).
audience: internal
---

# Hooks

Two things in the Ship codebase are called "hooks" and the distinction matters. This page separates them.

## 1. Compiler lifecycle hooks — `HookConfig` / `HookTrigger`

Shell commands run on agent lifecycle events. Authored as part of agent config and **compiled into harness-native config** — Ship doesn't run them itself, the harness does.

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

Types live in `ship_platform::config::types::{HookConfig, HookTrigger}` (re-exported through `ship_platform::config`). The compiler has its own narrower `HookTrigger` in `crates/core/compiler/src/types/config.rs` — only 6 variants (`PreToolUse`, `PostToolUse`, `Notification`, `Stop`, `SubagentStop`, `PreCompact`) — that the platform triggers map down to per harness.

### Triggers

19 variants in `ship_platform::config::types::HookTrigger`:

| Trigger | When it fires |
|---|---|
| `SessionStart` | Session begins. |
| `UserPromptSubmit` | User submits a prompt turn. |
| `PreToolUse` | Before a tool is invoked. |
| `PermissionRequest` | Agent requests tool permission. |
| `PostToolUse` | After a tool returns successfully. |
| `PostToolUseFailure` | After a tool errors. |
| `Notification` | Harness notification event. |
| `SubagentStart` / `SubagentStop` | Sub-agent lifecycle. |
| `Stop` | Agent's turn ends. |
| `PreCompact` | Before context compaction. |
| `BeforeTool` / `AfterTool` | Gemini-style tool events. |
| `BeforeAgent` / `AfterAgent` | Gemini-style agent events. |
| `SessionEnd` | Session closes. |
| `BeforeModel` / `AfterModel` | Gemini-style model invocation events. |
| `BeforeToolSelection` | Before the agent chooses a tool. |

Not every harness supports every trigger. The compiler maps triggers to the active harness's native schema; unsupported triggers get dropped at compile time with a warning in the emit output.

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
- `description` — human-readable intent. Surfaced in editor UX and in the exported harness config as a comment where supported.
- `command` — shell command. Runs with the workspace's env + Ship-specific vars (`SHIP_SESSION_ID`, `SHIP_TOOL_NAME`, etc.) set.

### Where they fire

All triggers are **harness-side**. The compiler emits them into harness-native hook config (e.g. `.claude/settings.json:hooks`), and the harness (Claude Code, Gemini CLI, etc.) runs them directly inside its execution loop. Ship doesn't observe or execute these — they run in the agent's process. Triggers a harness doesn't support are dropped at compile time with a warning in the emit output.

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

A non-zero exit from the command signals hook failure. What the agent sees varies by harness (some refuse to exit; some pass the error through). The behavior is deliberately harness-dependent — Ship declares the hook; the harness enforces.

## 2. Platform reactive triggers — `HookDef` / `HookTrigger`

A different system that also lives under the "hooks" name — the public surface is **triggers** (the Rust types still carry the `Hook*` prefix; the module path and JSONC directory have been renamed). The code lives in `crates/core/ship-platform/src/triggers/`. These are reactive automation: a JSONC document in `.ship/triggers/*.jsonc` matches a Ship event (or an external source) and runs actions. Ship executes these itself — they are not compiled into harness config.

```rust
// crates/core/ship-platform/src/triggers/def.rs
pub struct HookDef {
    pub name: String,
    pub description: Option<String>,
    pub trigger: HookTrigger,
    pub actions: Vec<HookAction>,
}

pub struct HookTrigger {
    /// Internal event type (e.g. "pipeline.run.failed"). Mutually exclusive with `source`.
    pub event: Option<String>,
    /// External trigger source (webhook, file watch, cron). Mutually exclusive with `event`.
    pub source: Option<TriggerSourceDef>,
    /// Optional dot-path filter on the event payload.
    pub filter: Option<serde_json::Value>,
}
```

Exactly one of `event` or `source` is set. Event triggers match a kernel event type directly; source triggers (`Webhook`, `File`, `Cron`) are surfaced as synthetic `external.<kind>.<name>` events. The `HookEngine` (`triggers/engine.rs`) loads the documents, watches for changes, and dispatches matching events to the actions. Distinct from the compiler lifecycle hooks above — those are an agent/harness concern, these are a runtime automation concern.

## Why the naming collision exists

Both systems were named "hooks" independently:

- `HookConfig` / `config::types::HookTrigger` — agent lifecycle hooks, the Claude/Gemini-style "shell commands on events" vocabulary. Compiled into harness config.
- `HookDef` / `triggers::def::HookTrigger` — reactive triggers matching Ship events. Runtime-executed. The filesystem and module path use **triggers**; the Rust types retain the `Hook*` prefix for historical reasons.

When reading code, disambiguate by context:

- `HookConfig`, `config::types::HookTrigger`, agent config, "compiled into harness config" → compiler lifecycle hooks.
- `HookDef`, `triggers::def::HookTrigger`, `HookEngine`, `.ship/triggers/*.jsonc` → platform reactive triggers.

## Invariants

- Compiler lifecycle hooks are data. They compile into harness config; Ship never executes them — harnesses do.
- Platform reactive triggers are runtime-executed. The `HookEngine` matches events and runs actions.
- The two `HookTrigger` types are unrelated — one is a lifecycle enum, the other an event/source matcher struct.

## Related

- [Config](/runtime/config/) — where `HookConfig` lives and the file format.
- [Compiler](/compiler/) — lifecycle-hook translation per harness.
