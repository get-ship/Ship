---
name: sui-eval
stable-id: sui-eval
description: Run the SUI authoring eval. Dispatch lane agents to build a view from a spec, poll until they finish, grade compile + lint + structure, record scores to the sui-eval-runs collection. Off-daemon — uses dispatch.sh, not pipelines.
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
- `scripts/grade.ts` — compile + lint + structural rubric → `.artifacts/sui-eval-runs/<run-id>.jsonl`
- `scripts/record.ts` — jsonl → `collections.create_item` into `sui-eval-runs`
- `task-spec.md` + `mockups/` — the authoring task the lanes build

## Run it

**1. Dispatch.** Haiku-only is the default (cheap smoke). Add models for a live run:

```bash
bash .ship/skills/sui-eval/scripts/dispatch-eval.sh                       # haiku
bash .ship/skills/sui-eval/scripts/dispatch-eval.sh --models haiku,sonnet # live A/B
```

Note the printed `RUN_ID` and lane slugs. Lanes open as wt tabs and work
autonomously off `task-spec.md`.

**2. Loop until lanes finish.** Each lane is done when it has committed past the
base branch. Poll — don't babysit. Use the `loop` skill:

```
/loop 5m check eval RUN_ID=<id>: for each ~/.ship/worktrees/eval-*-<id>,
  `git -C <wt> rev-list HEAD ^<base> --count` > 0 means that lane committed.
  When ALL lanes have committed (or after 40m), stop looping and run step 3.
```

A lane that committed but doesn't compile still gets graded — grade is a
measurement, not a gate. Premature commits score low; that's signal, not error.

**3. Grade + record.** dispatch-eval.sh prints the exact commands for the run.
Per lane, then once:

```bash
(cd ~/.ship/worktrees/eval-haiku-<id> && SHIP_PIPELINE_RUN_ID=<id> bun <repo>/.ship/skills/sui-eval/scripts/grade.ts)
SHIP_PIPELINE_RUN_ID=<id> bun <repo>/.ship/skills/sui-eval/scripts/record.ts
```

`grade.ts` discovers lanes by `-haiku`/`-sonnet` suffix under `SHIP_WORKTREE_DIR`
(default `~/.ship/worktrees`). Override the target view + rubric for a different
task:

```bash
SUI_EVAL_VIEW_PATH=views/smoke-counter SUI_EVAL_REQUIRED_FRAGMENTS='on:click' bun .../grade.ts
```

**4. Read results** in `views/eval` (grouped by run_id) or the raw jsonl under
`.artifacts/sui-eval-runs/`.

## Cleanup

After scoring, remove lane worktrees + branches:

```bash
for wt in ~/.ship/worktrees/eval-*-<id>; do git worktree remove --force "$wt"; done
git branch | grep "job/eval-.*-<id>" | xargs -r git branch -D
```
