---
name: spawn-agent
audience: internal
stable-id: spawn-agent
description: Dispatch a job to a specialist agent in a git worktree. Write the spec, create the worktree, launch the agent.
tags: [commander, dispatch, worktree]
authors: [ship]
---

# Dispatch

Launch a specialist agent in a git worktree to execute a job. One command does everything.

## Hard rule for dispatched agents

**NEVER push to a remote.** Agents commit locally to their branch. The orchestrator gates by reading the local worktree state. `git push` is a side-effect on a shared remote that requires explicit human authorization — never assume it. If a job spec ever says "push", treat it as a typo and stop at commit.

## Usage

```bash
bash .ship/skills/spawn-agent/scripts/dispatch.sh \
  --slug <name> \
  --agent <agent> \
  --spec <path-to-spec>
```

| Flag | What |
|------|------|
| `--slug <name>` | Worktree dir and branch name (required) |
| `--agent <agent>` | Ship agent profile (required) |
| `--spec <file>` | Path to job spec (required) |
| `--base <branch>` | Branch to fork from (default: current branch) |
| `--model <id>` | Override model for the spawned agent |
| `--skill <id>` | Extra skill to inject at compile time, repeatable (e.g. `--skill web-qa --skill tdd`) |
| `--no-open` | Print launch command instead of opening terminal |
| `--dry-run` | Show what would happen |
| `--confirm` | Show spec and ask y/n before launching |
| `--batch <name>` | Group this dispatch into a shared wt window `ship-batch-<name>`. First call opens the window; each subsequent call with the same name adds a split-pane. Lets you monitor 4-5 parallel agents in ONE wt window instead of N tabs. Lock expires after 24h. Currently honored on the `wt` terminal path. |
| `--harness <cli>` | Override the provider binary launched in the worktree: `claude` (default), `omp`, `codex`, etc. The harness must be authenticated in the launching terminal's environment. |

### Environment variables

These are personal configuration — set in your shell profile, not in project files.

| Variable | What |
|----------|------|
| `SHIP_DEFAULT_TERMINAL` | Force terminal: `tmux`, `wt`, `iterm`, `vscode`, `warp`, `manual` |
| `SHIP_WORKTREE_DIR` | Base directory for worktrees (default: `~/dev/ship-worktrees`) |
| `SHIP_GLOBAL_DIR` | Ship global data dir (default: `~/.ship`) |
| `SHIP_AGENT_MODEL` | Default model for spawned agents |
| `SHIP_DISPATCH_CONFIRM` | Set to `1` to always prompt before dispatch |
| `SHIP_PROVIDER_CLI` | Override provider binary: `claude`, `codex`, `gemini`, `opencode` |
| `WT_PROFILE` | Windows Terminal profile name to launch panes under (default: `$WSL_DISTRO_NAME`). Set to your preferred profile (e.g. `Ubuntu`) so panes inherit your font/colors instead of wt's default. |

## Workflow

1. Write the job spec to `.artifacts/job-spec-<slug>.md`
2. Run dispatch: `bash scripts/dispatch.sh --slug <slug> --agent <agent> --spec <path>`
3. Dispatch creates worktree, compiles agent config, opens terminal
4. Agent picks up `.artifacts/job-spec.md` on start and works autonomously

## Job Spec Template

```markdown
# Job Spec: <title>

**Branch:** job/<slug>
**Agent:** <agent>
**Mode:** autonomous

## Goal

<one-paragraph outcome>

## File scope

<directories/files the agent may modify>

## What to change

<specific instructions>

## Constraints
- Key constraints the agent must honor: <hard rules: never weaken guards/tests, never push, gate commands — or "none">
- (Charter 2026-06-11: ADR/architecture-decision content is operator material — do NOT inject it into agent payloads. Constraints travel; rationale doesn't.)

## Acceptance criteria

<numbered verifiable outcomes>

---
> Execute this job yourself. Do not delegate the implementation to a subagent — subagents are only for read-only parallel research, never for writing the code this job asks for.
>
> If you notice a bug or problem outside your file scope, jot it as a one-line note in the commit body (`note(out-of-scope): ...`) and continue your work — don't widen scope to fix it.
>
> **When all acceptance criteria are met:** commit cleanly. **DO NOT push the branch.** Pushing to remote is forbidden without the human's explicit say-so. The gate-side process reads the local worktree to discover your work — remote is irrelevant.
>
> (If you're invoked as a pipeline step — `SHIP_ACTOR_ID` is set, you're inside `ship run` — then ALSO call `pipelines.step_done` over the daemon RPC surface to advance the step monitor:
> ```
> ship ops call pipelines.step_done --json '{"slug":"'$SHIP_ACTOR_ID'","summary":"<one-liner>"}'
> ```
> The CLI is a thin transport; everything flows through `ship ops call`. There is no special completion CLI.)
```

## Test/Impl Split

For feature work, dispatch as two sequential jobs:

```bash
# Job 1: tests only
bash .ship/skills/spawn-agent/scripts/dispatch.sh --slug auth-tests --agent test-writer \
  --spec .artifacts/job-spec-auth-tests.md

# Job 2: implementation (after tests complete and gate passes)
bash .ship/skills/spawn-agent/scripts/dispatch.sh --slug auth-impl --agent rust-lane \
  --spec .artifacts/job-spec-auth-impl.md
```

The test spec scopes to test files only. The impl spec references the test files as its contract and must not modify them.

{% if runtime.agents %}
## Available Agents

{% for a in runtime.agents %}- **{{ a.id }}**{% if a.description %} — {{ a.description }}{% endif %}
{% endfor %}
{% endif %}

## Environment

Dispatch verifies before launching:
1. `ship` CLI is on PATH
2. `ship use <agent>` succeeds in the worktree
3. `.mcp.json` exists and registers the `ship` MCP server (written by `ship use`)

If any check fails, the agent is not launched.

{% if terminal == "auto" %}
Terminal: auto-detected (`$TMUX` → tmux, `$WT_SESSION` → wt, `$TERM_PROGRAM` → iterm/vscode).
{% else %}
Terminal: **{{ terminal }}**
{% endif %}

Worktree base: **{{ worktree_dir }}**

```bash
ship vars set dispatch terminal <auto|tmux|wt|iterm|vscode|manual>
ship vars set dispatch worktree_dir <path>
```

## Cleanup

After gate passes:
```bash
git worktree remove {{ worktree_dir }}/<slug>
git branch -d job/<slug>
```
