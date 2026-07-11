---
name: configure-agent
description: Use when setting up a workspace for a specialist agent — selecting agent, permission tier, scope, and worktree path. Ensures agents are productive (not over-restricted) and safe (not under-restricted).
stable-id: configure-agent
---

## Permission Preset Selection

Pick the preset that matches the blast radius of the work, not your anxiety level.
Over-restricting kills productivity — every approval interruption breaks flow.

| Preset | default_mode | Use for |
|--------|-------------|---------|
| `ship-readonly` | `plan` | Reviewer/gate agents, analysis-only, no writes |
| `ship-standard` | `default` | Interactive sessions, commander, orchestration |
| `ship-autonomous` | `dontAsk` | Most specialist workers in worktrees — web, Rust, docs, tests |
| `ship-elevated` | `dontAsk` | CI agents, release automation — unlocks push/publish |

**Default to `ship-autonomous` for dispatched agents.** Agents in worktrees doing scoped feature work should not be interrupted. Use `ship-standard` for interactive sessions where you want confirmation prompts.

## Base Rules (all presets)

Every preset inherits these rules — they cannot be overridden by agents:

- **Always allow:** `Bash(ship *)` (the Connect op surface — `ship ops call`)
- **Always deny:** `Bash(sqlite3 ~/.ship/*)`, `Bash(git push*)`, `Bash(*publish*)`, secrets files (`.env`, `credentials*`)
- **Always ask:** `.ship/` writes

`ship-elevated` unlocks `Bash(git push*)` and `Bash(*publish*)` via `tools_allow_override`.

## The Compound Command Problem

Claude Code matches permission patterns against the FULL command string.
`Bash(ship exec*)` will NOT match `cd .target/bin && ship exec blah`.

Rules:
- Never try to allowlist compound commands with patterns — it won't work
- Use `default_mode` via presets to set the baseline, `tools_ask` only for specific destructive ops
- If a legitimate workflow always chains commands, document the full pattern OR grant the preset that doesn't need to ask

```toml
# Wrong: trying to pattern-match compound commands
tools_ask = ["Bash(ship exec*)"]   # misses: cd dir && ship exec

# Right: set mode, guard only what's genuinely dangerous
[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(rm -rf*)"]
```

## Agent → Preset Mapping

| Agent | Recommended preset | Rationale |
|---------|-------------------|-----------|
| commander | `ship-standard` | Orchestration, needs confirmation for dispatch |
| gatekeeper | `ship-standard` | Gate + review, needs bash for checks, owns merge |
| rust-lane | `ship-autonomous` | Scoped feature work in worktree |
| web-lane | `ship-autonomous` | Scoped feature work in worktree |
| test-writer | `ship-autonomous` | Tests only, low blast radius |
| default | `ship-standard` | Interactive human-paired sessions |

## Scope Constraints


Always set explicit file scope in the job spec. The agent constrains capability
(what the agent knows how to do). Scope constrains authority (what it's allowed to touch).

```
Scope: apps/gateway/src/routes/, apps/gateway/src/lib/
Off-limits: crates/, apps/mcp/, apps/ship-studio-cli/
```

Agents WILL respect explicit scope instructions. Don't rely on preset alone.

## Checklist Before Starting an Agent

- [ ] Correct agent for the domain?
- [ ] Permission preset matched to blast radius (default to ship-autonomous for worktrees)?
- [ ] Job spec includes scope + off-limits?
- [ ] Acceptance criteria written (not just "make it work")?
- [ ] Dependencies noted (what must already be true)?

**Worktree + agent setup are no longer checklist items.** helmd's
`ship.ops.v1.AgentsService/Dispatch` provisions the worktree, branch,
and agent config in one RPC. The dispatch-portable.sh CLI (or the MCP
`dispatch_job` tool, or the Launch UI) is the operator-facing entry
point; from this skill's perspective the worktree path and the
`ship use <agent>` step are helmd's job, not yours.
