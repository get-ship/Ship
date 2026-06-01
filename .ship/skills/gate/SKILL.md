---
name: gate
stable-id: gate
description: Gate a completed job branch before merge. The objective bar is machine-proven via `just gate`; the operator judges only taste. Run inline or as subagent.
tags: [gate, review, quality]
authors: [ship]
---

# Gate Review

A merge gate has two layers. **Do not conflate them.**

1. **Objective bar — machine-proven, not claimed.** `just gate` runs the full check/lint/drift/test surface. If it's green, the mechanical bar is met. You never hand-grep what a command can prove.
2. **Taste bar — human/agent judgment.** The one thing no command catches: is this the right abstraction, thin enough, idiomatic, did it solve the right problem. A diff *skim* for shape, not a line-read for correctness.

A gate is PASS only when **both** pass. Green-but-ugly is a FAIL.

## Protocol

```
1. Read the job spec (acceptance criteria + file scope).
2. Run `just gate` in the worktree. It MUST exit 0.
   - It composes: check + lint (clippy + check-raw) + gen-*-check drift + workspace tests.
   - Never trust the agent's "tests pass" — run it yourself.
3. Verify the spec's explicit acceptance criteria the gate doesn't cover
   (e.g. "no sqlite types in the crate public surface", "old module untouched",
   additive-only). Grep for them.
4. TASTE SKIM the diff: right abstraction? thin? matches codebase idiom?
   over-built? a parallel implementation that should have replaced something?
5. Both green → PASS. Either fails → FAIL with specific evidence.
```

### Locked-test discipline (the trust anchor)

A passing test the **implementer wrote** proves only self-consistency. The trustworthy signal is a test written by **not-the-implementer** (test-writer agent or operator) that the implementer **could not edit**. When a spec ships locked tests, "green" means "met a bar it couldn't move." When it doesn't, weight the test result less and skim the impl harder.

If a refactor *replaces* an old impl, the new crate's coverage must reach **parity with what the old tests covered** before the old is deleted — deleting tested code for under-tested code is a FAIL even if green.

## Verdict format

```
GATE PASS: <job> · job/<slug>
  ✓ just gate green (check + lint + drift + tests)
  ✓ <acceptance criterion> — <evidence>
  ✓ taste: <one line — abstraction/idiom judgment>
  Ready to merge.

GATE FAIL: <job> · job/<slug>
  ✓ just gate green
  ✗ <criterion> — expected X, got Y · <command output>
  ✗ taste: <what's wrong — over-built / parallel-impl / wrong seam>
  What needs to change: <actionable>
```

## The merge (on PASS)

Gate owns the merge; the dispatcher never touches git mid-job.

```bash
cd <main-repo> && git merge --no-ff job/<slug> -m "merge: <slug> — <goal>"
just gate                      # re-verify on the merged tree
# CLEANUP (mandatory — additive crates are parallel impls until cutover; never leak worktrees):
tmux kill-session -t <slug> 2>/dev/null
git worktree remove ~/dev/ship-worktrees/<slug> --force
git branch -d job/<slug>
```

Target branch is the current integration branch (today `feat/window-actors`), not `main`. Record the verdict in the job's tracking doc.

## Replacement discipline (cutover, not accretion)

The #1 agentic failure is building-over instead of replacing. An additive new crate is only the FIRST half of a move; the cutover that deletes the old impl is the committed second half. When gating a service rewrite:
- New impl proven (parity tests green) → the old impl + its competing proto + REST remnant go on `.artifacts/services/_deprecations.md` with an owner.
- A FAIL if the diff *adds* a third way to do something instead of replacing.

## Notes
- `just gate` is the source of truth for the objective bar. If a check is missing from it, add the check to xtask/scripts/lint and compose it into the recipe — never hand-grep in the gate instead.
- UI surfaces still require a fresh `web-qa` artifact (see `web-qa` skill) — that gate is unchanged.
