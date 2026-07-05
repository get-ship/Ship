---
name: sui-eval
stable-id: sui-eval
audience: internal
description: Run the SUI authoring eval. Dispatch lane agents to build a view from a spec, watch their tmux panes for idle (not commits), grade compile + lint + structure, record scores to the sui-eval-runs collection. Off-pipeline — uses dispatch-portable.sh → helmd, not pipelines.
tags: [eval, sui, dispatch, grading]
audience: internal
authors: [ship]
---

# SUI authoring eval

Measures how well an agent config authors a SUI view from a spec. One lane per
model. No pipeline orchestrator — `dispatch-portable.sh` POSTs each lane to
helmd (boots helmd first if it's down), helmd spawns it in its own worktree +
tmux tab, a poll loop waits for lanes to idle, then `grade.ts` scores each
worktree and `record.ts` makes the scores queryable in `views/eval`.

Why off-pipeline: the pipeline fanout path is unreliable (the supervisor sweep
can murder live lanes). helmd's bare single-agent dispatch survives daemon
hiccups and is fully observable in a real PTY.
## Files

- `scripts/dispatch-eval.sh` — fork N lanes (one per model) from the current branch
- `scripts/lane-watch.sh` — peer into each lane's tmux pane, detect idle, auto-grade
- `scripts/grade.ts` — compile + lint + structural rubric → `.artifacts/sui-eval-runs/<run-id>.jsonl`
- `scripts/record.ts` — jsonl → `collections.create_item` into `sui-eval-runs`
- `task-spec.md` + `mockups/` — the authoring task the lanes build

## Run it

**1. Dispatch.** Haiku-only is the default (cheap smoke). Add models for a live run:

```bash
bash .ship/skills/sui-eval/scripts/dispatch-eval.sh                       # haiku
bash .ship/skills/sui-eval/scripts/dispatch-eval.sh --models haiku,sonnet # live A/B
```

Note the printed `RUN_ID`. Lanes open as wt tabs, each in a tmux session named
after its slug (`eval-<model>-<RUN_ID>`), and work autonomously off `task-spec.md`.

**2. Watch + grade — one command, no commit race.** Completion is detected by
peering into the tmux pane, NOT by waiting for a commit. Agents routinely finish
the work and never commit; a commit-based watcher reads that as "no work" and the
cohort gets cleaned up. The pane is ground truth — when Claude Code stops printing
and its "esc to interrupt" indicator is gone, the lane is done:

```bash
bash .ship/skills/sui-eval/scripts/lane-watch.sh <RUN_ID>
```

It blocks until every lane is idle (120s quiet + no spinner) or the 40m timeout,
then grades each worktree and records — no external `/loop`, no `git` dependency.
Run it in a tmux pane of your own (or background it) so it survives. Pass
`--no-grade` to only signal (writes `.artifacts/sui-eval-runs/.watch/<RUN_ID>/<slug>.done`).

If a lane goes idle with **uncommitted work** on disk — the case that burns you
downstream (idle agent, real work, no commit → cleanup assumes nothing happened) —
the watcher `send-keys` a nudge to commit and gives it another round (up to 2
prods) before grading the dirty tree anyway. Grading reads files from disk, so an
uncommitted lane still scores; the prod is about leaving a clean artifact, not
about the score.

A lane that's idle but doesn't compile still gets graded — grade is a measurement,
not a gate. Low score is signal, not error.

**3. Read results** in `views/eval` (grouped by run_id) or the raw jsonl under
`.artifacts/sui-eval-runs/`.

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
SUI_EVAL_BEHAVIOR_SPEC=.ship/skills/sui-eval/tasks/authority/authority.behavior.jsonc \
bun .ship/skills/sui-eval/scripts/grade.ts
```

`SUI_EVAL_BEHAVIOR_SPEC` adds the behavioral dimension: `behavior-grade.ts` runs
the spec's scenarios against the candidate in jsdom (vitest + the production WASM
compiler via `@ship/test-setup`'s `runScenarios`) and the composite reweights to
compile 0.30 / lint 0.10 / structural 0.20 / behavior 0.40. Without it the legacy
0.40/0.20/0.40 split applies. `behavior-grade.ts` also runs standalone:

```bash
bun .ship/skills/sui-eval/scripts/behavior-grade.ts <lane-dir> <view-path> <spec.behavior.jsonc>
```

## Cleanup

After scoring, remove lane worktrees + branches:
```bash
for wt in ~/dev/Ship/worktrees/eval-*-<id>; do git worktree remove --force "$wt"; done
git branch | grep "job/eval-.*-<id>" | xargs -r git branch -D
```
