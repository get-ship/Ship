# Job Spec: migrate `views/pipelines/history/page.sui` to `{#async}`

**Branch:** `job/pipelines-history-async`
**Agent:** ship-designer
**Mode:** autonomous
**Context:** The kill-legacy-async pass landed `{#async}` directive support in Rust. 13 view files were migrated mechanically; one was carved out: `views/pipelines/history/page.sui` — the rationale in commit `7e130c266` was *"renders content unconditionally above its banners — wrapping in `:ok` would change semantics."* Now that the directive is settled, revisit with care.

## Goal

Migrate the history view to `{#async}` if possible. If the structural objection still holds after a real-eye review, restructure the view to fit the directive (the audit's preferred path) — most likely by splitting the unconditional content from the conditional content.

This is the **last** while-loading + while-error + if-ok triple in `views/`. Removing it closes the directive migration.

## File scope

```
views/pipelines/history/page.sui       # the view
views/pipelines/history/controller.ts  # may need adjustment for restructured signals
views/pipelines/history/style.css      # minor adjustments if layout shifts
```

DO NOT touch:
- Any other view (audit is closed for them).
- Compiler / runtime (the directive itself is settled).
- Apps / shell.

## What to change

Two acceptable outcomes:

### Outcome A — directive applies after all

If a closer read shows the unconditional content can sensibly live inside `{:ok}` (e.g., the always-on content is actually loading-state-aware), apply `{#async}` as the other 13 views did.

### Outcome B — restructure to fit

If outcome A genuinely doesn't apply, split the view into two parts:
- An always-rendered shell (filters, header, controls)
- An `{#async runs as v}{:ok}...{/async}` block for the run list itself

This is the structurally honest move: the `{#async}` block scopes only the data-driven content; the chrome above doesn't need it.

### Forbidden outcome

Outcome C — leave the legacy 3-block pattern in place — is NOT acceptable. Either A or B lands.

## Architectural context

- Directive is shipped (commit `00a008e1d`). 13 views already use it.
- Closing the last consumer means we can eventually drop the legacy `{#while-loading}` / `{#while-error}` directives from the compiler (separate, later cleanup — not in scope here).

## Acceptance criteria

1. `grep -rln "while-loading" views/` returns 0 matches.
2. `views/pipelines/history/page.sui` either uses `{#async}` (outcome A) or has its content split into chrome + scoped-async-block (outcome B).
3. Behavior is preserved: loading shows a loading state, error shows an error, ok shows the runs. No regression for any of the three states.
4. `bun run build` passes. `bun packages/deck/bundler/index.ts --all` succeeds.
5. Test (if any exists for this view) passes.
6. Visual QA artifact at `.artifacts/web-qa/pipelines-history-async/<timestamp>/`. Cover loading, error, populated states at ship-light. (Use mocked SDK responses to force each state if needed.)

## Out of scope

- Removing the legacy `{#while-loading}` directive from the compiler (the cleanup job comes AFTER this one closes).
- Polishing other aspects of the history view.
- Migrating to `/studio/pipelines/history` (route migration is a separate later job).

---

> Execute. Smallish job.
>
> When done: complete_workspace + mesh_broadcast.
