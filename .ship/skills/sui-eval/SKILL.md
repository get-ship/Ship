---
name: sui-eval
stable-id: sui-eval
description: Run the SUI authoring eval. Dispatch lane agents to build a view from a spec, watch their tmux panes for idle (not commits), grade compile + lint + structure, record scores to the sui-eval-runs collection. Off-daemon — uses dispatch.sh, not pipelines.
tags: [eval, sui, dispatch, grading]
authors: [ship]
---

# SUI authoring eval

Measures how well an agent config authors a SUI view from a spec. One lane per
model. No pipeline, no daemon fanout — `dispatch.sh` spawns each lane in its own
worktree + wt tab, a poll loop waits for them to commit, then `grade.ts` scores
each worktree and `record.ts` makes the scores queryable in `views/eval`.

Why off-daemon: the daemon fanout path is unreliable (supervisor sweep murders
live lanes — #189 recurred; installed binary hardcodes :9315). dispatch.sh lanes
survive daemon hiccups and are fully observable in a real PTY.

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

### Manual grade (override the task)

`grade.ts` discovers lanes by `-haiku`/`-sonnet` suffix under `SHIP_WORKTREE_DIR`
(default `~/.ship/worktrees`). Override the target view + rubric for a different
task:

```bash
SUI_EVAL_VIEW_PATH=views/smoke-counter SUI_EVAL_REQUIRED_FRAGMENTS='on:click' bun .../grade.ts
```

## Cleanup

After scoring, remove lane worktrees + branches:

```bash
for wt in ~/.ship/worktrees/eval-*-<id>; do git worktree remove --force "$wt"; done
git branch | grep "job/eval-.*-<id>" | xargs -r git branch -D
```
