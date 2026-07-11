---
name: sui-eval
description: Run the SUI authoring eval. Dispatch lane agents to build a view from a spec via dispatch-portable.sh → helmd, watch their tmux panes for idle (not commits), grade compile + lint + structure, record scores into the sui-eval-runs collection. Off-pipeline — uses helmd's bare single-agent dispatch, not pipelines.
stable-id: sui-eval
---

# SUI authoring eval

Measures how well an agent config authors a SUI view from a spec. One lane per
model. No pipeline orchestrator — `dispatch-portable.sh` POSTs each lane to
helmd (boots helmd first if it's down), helmd spawns it in its own worktree +
tmux tab, a poll loop waits for lanes to idle, then `grade.ts` scores each
worktree and `record.ts` makes the scores queryable in `views/eval`.

**One dispatch path, no fallback.** Always use `dispatch-portable.sh` — the
`dispatch.sh` legacy spawner was deleted. The CLI is a thin
client of helmd's `ship.ops.v1.AgentsService/Dispatch`; every consumer
(this skill, the SPA, the MCP `dispatch_job` tool) routes through it.

Why off-pipeline: the pipeline fanout path is unreliable (the supervisor sweep
can murder live lanes). helmd's bare single-agent dispatch survives daemon
hiccups and is fully observable in a real PTY.

## Files

- `scripts/dispatch-eval.sh` — fork N lanes (one per model) from the current branch
- `scripts/lane-watch.sh` — peer into each lane's tmux pane, detect idle, auto-grade
- `scripts/grade.ts` — compile + lint + structural rubric; writes scores to
  the run's home (mission scratch when in scope, else `data/runs/sui-eval/<run-id>.jsonl`)
- `scripts/record.ts` — jsonl → `collections.create_item` into `sui-eval-runs`
- `task-spec.md` + `mockups/` — the authoring task the lanes build

## Where run output lives

Eval runs are **derived, gitignored** — the durable record is the
`sui-eval-runs` collection row, not a per-run artifact file. The local
JSONL the scripts write is a working copy that `record.ts` consumes.

When a mission is in scope, the JSONL lands in the mission scratch
(`${SHIP_MISSION_SCRATCH}/sui-eval-runs/<run-id>.jsonl`). Otherwise the
canonical derived-run home is `data/runs/sui-eval/<run-id>.jsonl` (the
ship-meta spec calls `data/runs/` out as the observed-run root). The
historical `.artifacts/sui-eval-runs/` location is retired — nothing
writes there any more.

## Run it

**1. Dispatch.** Haiku-only is the default (cheap smoke). Add models for a live run:

```bash
bash skills/sui-eval/scripts/dispatch-eval.sh                       # haiku
bash skills/sui-eval/scripts/dispatch-eval.sh --models haiku,sonnet # live A/B
```

Note the printed `RUN_ID`. Lanes open as wt tabs, each in a tmux session named
after its slug (`eval-<model>-<RUN_ID>`), and work autonomously off `task-spec.md`.

**2. Watch + grade — one command, no commit race.** Completion is detected by
peering into the tmux pane, NOT by waiting for a commit. Agents routinely finish
the work and never commit; a commit-based watcher reads that as "no work" and the
cohort gets cleaned up. The pane is ground truth — when Claude Code stops printing
and its "esc to interrupt" indicator is gone, the lane is done:

```bash
bash skills/sui-eval/scripts/lane-watch.sh <RUN_ID>
```

It blocks until every lane is idle (120s quiet + no spinner) or the 40m timeout,
then grades each worktree and records — no external `/loop`, no `git` dependency.
Run it in a tmux pane of your own (or background it) so it survives. Pass
`--no-grade` to only signal (writes the per-run done file alongside the jsonl).

If a lane goes idle with **uncommitted work** on disk — the case that burns you
downstream (idle agent, real work, no commit → cleanup assumes nothing happened) —
the watcher `send-keys` a nudge to commit and gives it another round (up to 2
prods) before grading the dirty tree anyway. Grading reads files from disk, so an
uncommitted lane still scores; the prod is about leaving a clean artifact, not
about the score.

A lane that's idle but doesn't compile still gets graded — grade is a measurement,
not a gate. Low score is signal, not error.

**3. Read results** in `views/eval` (grouped by run_id) or the raw jsonl under
the run's home.

### Manual / headless grade (override the task)

`grade.ts` discovers lanes via `git worktree list` by `-haiku`/`-sonnet`/`-opus`
suffix; with zero lane worktrees it falls back to the cwd as one anonymous lane,
so the whole grading pipeline runs headless against any candidate on disk — no
dispatch, no tmux, no daemon. Compile/lint use the prebuilt
`target/debug/sui-compiler` (override with `SUI_COMPILER=`), never `cargo run`.

```bash
SHIP_PIPELINE_RUN_ID=<run-id> \
SUI_EVAL_VIEW_PATH=views/authority \
SUI_EVAL_REQUIRED_FRAGMENTS='{#if,{#for,key={,on:click,{#while-loading,{#while-error' \
SUI_EVAL_BEHAVIOR_SPEC=skills/sui-eval/tasks/authority/authority.behavior.jsonc \
bun skills/sui-eval/scripts/grade.ts
```

`SUI_EVAL_BEHAVIOR_SPEC` adds the behavioral dimension: `behavior-grade.ts` runs
the spec's scenarios against the candidate in jsdom (vitest + the production WASM
compiler via `@ship/test-setup`'s `runScenarios`) and the composite reweights to
compile 0.30 / lint 0.10 / structural 0.20 / behavior 0.40. Without it the legacy
0.40/0.20/0.40 split applies. `behavior-grade.ts` also runs standalone:

```bash
bun skills/sui-eval/scripts/behavior-grade.ts <lane-dir> <view-path> <spec.behavior.jsonc>
```

## Cleanup

helmd's dispatch lifecycle owns lane teardown: worktrees, branches, and
the post-run `.artifacts/` were never yours to clean up. If a lane is
still alive at the end of a run, `tmux kill-session -t <slug>` is the
only operator-side cleanup; everything else is helmd's.
