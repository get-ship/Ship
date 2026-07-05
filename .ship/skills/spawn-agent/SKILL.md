---
name: spawn-agent
audience: internal
stable-id: spawn-agent
description: Dispatch a specialist agent through Helm — helmd compiles, provisions the worktree, and launches the harness. `dispatch-portable.sh` is the thin CLI client; it boots helmd locally if the daemon is down.
tags: [commander, dispatch, worktree]
authors: [ship]
---

# Dispatch

Dispatch a specialist agent to a job. **helmd owns dispatch** — it compiles the agent, provisions the worktree + branch, and launches the harness into a tmux tab you can attach to. Commander, the Launch UI, and `dispatch-portable.sh` are all clients of it.

## Hard rule for dispatched agents

**NEVER push to a remote.** Agents commit locally to their branch. The orchestrator gates by reading the local worktree state. `git push` is a side-effect on a shared remote that requires explicit human authorization — never assume it. If a job spec ever says "push", treat it as a typo and stop at commit.

helmd installs a shared pre-push guard at `<main-repo>/.git/hooks/pre-push`
during worktree provisioning: it blocks any push of `refs/heads/job/*` and
can only be bypassed with `SHIP_ALLOW_PUSH=1 git push ...` (deliberate,
one-shot). Agents running with bypassPermissions cannot silently push.

## How to dispatch

**Primary — through Helm.** Call the `dispatch_job` MCP tool (or the Launch UI, or `dispatch-portable.sh` — all POST helmd). helmd compiles the agent, provisions the worktree, and launches the harness. You never create worktrees or compile configs yourself.

`dispatch-portable.sh` boots helmd locally if it isn't already running
(looks for `target/debug/helmd` relative to the repo root, else `helmd` on
PATH), so the same command works from a fresh shell or after a daemon bounce.

The job params (MCP tool fields, or CLI flags on `dispatch-portable.sh`):

| Flag | What |
|------|------|
| `--slug <name>` | Worktree dir and branch name (required) |
| `--agent <agent>` | Ship agent profile (required) |
| `--spec <file>` | Path to job spec (required) |
| `--base <branch>` | Advisory only — helmd applies its own `job/<slug>` convention. Accepted for back-compat. |
| `--harness <cli>` | Override the harness binary: `claude` (default), `omp`, `codex`, etc. The harness must be authenticated in the launching terminal's environment. |
| `--model <id>` | Override model for the spawned agent |
| `--dir <path>` | Worktree base dir (informational; helmd resolves its own) |
| `--dry-run` | Resolve + print the plan; do NOT POST helmd / spawn |

Flags accepted for back-compat with the old in-script spawn (no-op on the
RPC path): `--no-open`, `--skill <id>`, `--confirm`, `--batch <name>`,
`--no-bootstrap`, `--ship-bootstrap`. They're silently swallowed so existing
callers don't break — don't add new code that depends on them.
### Environment variables

These are personal configuration — set in your shell profile, not in project files.

| Variable | What |
|----------|------|
| `SHIP_DEFAULT_TERMINAL` | Force terminal: `tmux`, `wt`, `iterm`, `vscode`, `warp`, `manual` |
| `SHIP_WORKTREE_DIR` | Base directory for worktrees (default: `~/dev/Ship/worktrees`) |
| `SHIP_GLOBAL_DIR` | Ship global data dir (default: `~/.ship`) |
| `SHIP_AGENT_MODEL` | Default model for spawned agents |
| `SHIP_DISPATCH_CONFIRM` | Set to `1` to always prompt before dispatch |
| `SHIP_PROVIDER_CLI` | Override provider binary: `claude`, `codex`, `gemini`, `opencode` |
| `WT_PROFILE` | Windows Terminal profile name to launch panes under (default: `$WSL_DISTRO_NAME`). Set to your preferred profile (e.g. `Ubuntu`) so panes inherit your font/colors instead of wt's default. |

## Workflow

1. Write the job spec — pass it to `dispatch_job`; for a working file use the mission scratch (`write_mission_scratch`), not a loose path.
2. Call `dispatch_job` (or the CLI). helmd provisions the worktree + branch, compiles the agent config, and launches the harness.
3. Watch it: `tmux attach -t ship-<slug>`, or in Mission Control. The agent picks up its spec and works autonomously.

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
- (Operator rationale and architecture decisions are not agent payload — constraints travel; rationale doesn't.)

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
bash .ship/skills/spawn-agent/scripts/dispatch-portable.sh --slug auth-tests --agent test-writer \
  --spec .artifacts/job-spec-auth-tests.md

# Job 2: implementation (after tests complete and gate passes)
bash .ship/skills/spawn-agent/scripts/dispatch-portable.sh --slug auth-impl --agent rust-lane \
  --spec .artifacts/job-spec-auth-impl.md
```

The test spec scopes to test files only. The impl spec references the test files as its contract and must not modify them.

{% if runtime.agents %}
## Available Agents

{% for a in runtime.agents %}- **{{ a.id }}**{% if a.description %} — {{ a.description }}{% endif %}
{% endfor %}
{% endif %}

## Environment

All pre-launch verification (CLI on PATH, `ship use <agent>`, `.mcp.json`
registration) happens server-side in helmd. A dispatch that fails any
check returns an error from the `Dispatch` RPC; no agent is launched.
Watch the response: it tells you which check failed.

Override the worktree base via `SHIP_WORKTREE_DIR` (default: `~/dev/Ship/worktrees`).
Override the helmd endpoint via `HELMD_URL=http://host:port` or the shorthand
`HELMD_ADDR=host:port`. `dispatch-portable.sh` falls back to a local helmd
(boots `target/debug/helmd` from the repo root or `helmd` on PATH) when
`HELMD_URL` / `HELMD_ADDR` is unset and the default port is unreachable.

{% if terminal == "auto" %}
Terminal: auto-detected (`$TMUX` → tmux, `$WT_SESSION` → wt, `$TERM_PROGRAM` → iterm/vscode).
{% else %}
Terminal: **{{ terminal }}**
{% endif %}

Worktree base: **{{ worktree_dir }}**

```bash
ship vars set dispatch worktree_dir <path>
```

## Cleanup

After gate passes:
```bash
git worktree remove {{ worktree_dir }}/<slug>
git branch -d job/<slug>
```
