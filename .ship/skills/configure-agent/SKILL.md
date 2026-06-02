---
name: configure-agent
audience: internal
stable-id: configure-agent
description: Use when setting up a workspace for a specialist agent — selecting agent, permission tier, scope, and worktree path. Ensures agents are productive (not over-restricted) and safe (not under-restricted).
tags: [configuration, agents, permissions, worktree]
authors: [ship]
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

## Worktree Setup

Canonical path: read `~/.ship/machine/local.jsonc:worktrees.dir`.
Default if not set: `~/dev/<project-name>-worktrees/`

```bash
# worktree path
WORKTREE_BASE=$(ship config get worktrees.dir 2>/dev/null || echo ~/dev/$(basename "$PWD")-worktrees)
WORKTREE_PATH="$WORKTREE_BASE/<slug>"

git worktree add "$WORKTREE_PATH" -b job/<job-id>
cd "$WORKTREE_PATH"
ship use <agent>
```

Never use `../<branch>` — always use the configured path.
Never use Claude Code's native worktree UI — it won't respect the config.

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
- [ ] Worktree created at canonical path?
- [ ] `ship use <agent>` run IN the worktree?
- [ ] Job spec includes scope + off-limits?
- [ ] Acceptance criteria written (not just "make it work")?
- [ ] Dependencies noted (what must already be true)?
