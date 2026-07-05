---
name: gate
audience: internal
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

## The merge (on PASS) — Land, never raw git merge

The merge is owned by helmd's `GitService/Land` RPC. **No raw `git merge`.**
A raw merge can never hit the integration branch correctly (it would have
to know the operator's current integration target, and a manual
`git worktree remove` + `git branch -D` would race with helmd's own
dispatch lifecycle). Land is the one path that squash-merges onto the
integration branch, deletes the source branch (it is LANDED by
definition), and removes the worktree if clean — all atomic, all logged.

```bash
# Resolve the integration branch (ask the operator or check the worktree
# you were dispatched from — never assume `main`).
TARGET=$(git -C <main-repo> symbolic-ref refs/remotes/origin/HEAD 2>/dev/null \
    | sed 's@^refs/remotes/origin/@@' || echo "feat/next")

# Land the job onto the integration branch. On a clean Land the source
# branch + worktree are already gone; on conflict the RPC reports the
# conflicting paths and writes nothing.
ship ops call git.land --json "$(cat <<EOF
{
  "repo_path": "<main-repo>",
  "source_branch": "job/<slug>",
  "target_branch": "$TARGET",
  "message": "<slug> — <one-line goal>"
}
EOF
)"

# Re-verify on the merged tree (the integration branch tip).
just gate
```

Target branch is the **CURRENT integration branch** (typically `feat/next`),
not `main`. `main` is a release cut, not an integration point. The
`Release` RPC (separate from `Land`) is the squash that promotes
integration → main; the operator runs it at release time, not at gate
time. Record the verdict in the job's tracking doc after Land.

**Why no `git worktree remove` + `git branch -d` here:** Land does the
cleanup itself as part of the same RPC. Doing it by hand first is a race
(the cmdr might be dispatching another job into that worktree) and is
almost always wrong — Land's cleanup is gated on a CLEAN worktree, so
uncommitted work is preserved exactly once.

## Replacement discipline (cutover, not accretion)

The #1 agentic failure is building-over instead of replacing. An additive
new crate is only the FIRST half of a move; the cutover that deletes the
old impl is the committed second half. When gating a service rewrite:
- New impl proven (parity tests green) → file a `kind: annotation` claim
  (non-blocking) or `kind: observation` (must-fix) against the affected
  Subsystem/Feature. The deprecation is tracked in the claim inbox, not
  in a `.artifacts/services/_deprecations.md` file (the artifacts concept
  is retired — everything goes through Helm).
- A FAIL if the diff *adds* a third way to do something instead of
  replacing.

## Banned-vocabulary gate

The banned-vocabulary gate is a regression fence against **deleted concepts**
crawling back into the tree via a stale test or a careless edit. It is wired
into `just lint-vocab` (folded into `check-raw` → `lint` → `gate`) and runs
`scripts/lint/check-banned-vocab.sh`, which ripgreps every banned term across
hand-authored source, proto, and **tests** (tests are deliberately in scope —
a test reintroducing a dead word must fail).

**Ban-list:** `.ship/lint/banned-vocab.txt`. One entry per line:

```
<regex-or-literal><TAB><reason>
```

Field 1 is a ripgrep pattern (literal or regex); a single TAB separates it
from the reason, which is printed on a hit. `#` comments and blank lines are
ignored. An **empty list is a no-op** — the gate exits 0 and prints
`banned-vocab: 0 terms, clean`. On a hit it prints
`BANNED VOCAB: <term> — <reason>` with the offending `file:line` and exits
non-zero.

**THE RULE — never ban a term that still compiles.** A refactor step appends a
term to the ban-list only *once its last legitimate use is deleted*. The ban is
the closing act of an eradication, not the opening one: delete the real uses,
prove the tree is clean, then add the fence so the concept can't return.
Banning a live term breaks the build for everyone — that is a process error,
not a caught regression.

## Notes
- `just gate` is the source of truth for the objective bar. If a check is missing from it, add the check to xtask/scripts/lint and compose it into the recipe — never hand-grep in the gate instead.
- UI surfaces still require a fresh `web-qa` artifact (see `web-qa` skill) — that gate is unchanged.
- The cmdr skill's Gate Protocol section is the companion to this one; it owns the operator-facing merge step (Land), this skill owns the objective+taste verdict that authorizes it.
