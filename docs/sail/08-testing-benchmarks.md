# Testing and benchmarks

Two test layers and one benchmark that justify the framework. Without these, "Sail is the easiest framework anywhere" is marketing; with them, it's auditable.

## Layer 1: compiler unit tests

Snapshot-style tests in `crates/sail-compiler/tests/corpus/`:

```
tests/corpus/
├── 01-minimal/
│   ├── input.set
│   └── expected/
│       ├── output.js              ← snapshot of compiled JS
│       ├── output.html.tmpl       ← snapshot of SSR template
│       ├── output.d.ts            ← snapshot of types
│       └── output.schema.json     ← snapshot of schema
├── 02-state-and-derived/
│   └── ...
├── 03-template-control-flow/
├── 04-component-composition/
├── 05-typed-props/
├── 06-style-scoping/
├── 07-data-declarations/
├── 08-events-and-bindings/
├── 09-slots/
└── 10-edge-cases/
```

Each test compiles `input.set` and diffs against `expected/`. Drift is caught instantly. Snapshots are updated via `cargo test -- --update-snapshots` after intentional changes.

**Coverage target:** every spec feature has at least one corpus entry. ~50 corpus entries for the v1 spec.

## Layer 2: runtime unit tests

Vitest tests in `packages/@sail/runtime/test/`:

```
test/
├── signal.test.ts                 ← tracking, batching, derived, untrack
├── component.test.ts              ← defineComponent, lifecycle, attribute changes
├── bind.test.ts                   ← every bind* directive
├── on.test.ts                     ← event modifiers, cleanup
├── for.test.ts                    ← keyed reconciliation, edge cases
├── slot.test.ts                   ← fill, fallback, distribution
├── patch.test.ts                  ← every op + JSON path edge cases
├── data.test.ts                   ← SSE subscription, reconnect, cleanup
└── morph.test.ts                  ← idiomorph integration
```

~150 tests at the runtime level. Run as part of CI on every commit. <5s total.

## Layer 3: integration tests (Playwright)

`tests/loops/sail-*.ui.spec.mjs` — browser-rendered specs that prove framework features work end-to-end against a running Sail compiler + daemon.

The pattern is established in `tests/loops/agent-modal.ui.spec.mjs` (just landed). Per feature:

```
tests/loops/
├── sail-reactivity.ui.spec.mjs       ← signal updates trigger DOM updates
├── sail-bindings.ui.spec.mjs         ← every bind directive in a real browser
├── sail-events.ui.spec.mjs           ← on:click, modifiers, cleanup
├── sail-for.ui.spec.mjs              ← keyed reconciliation: insert/remove/move/swap
├── sail-slots.ui.spec.mjs            ← named + default + fallback
├── sail-data-fetch.ui.spec.mjs       ← data declaration, SSR'd value, hydration
├── sail-data-live.ui.spec.mjs        ← SSE subscription, patch application, reconnect
├── sail-patch-canvas.ui.spec.mjs     ← canvas use case: many patches, no flicker
├── sail-component-composition.ui.spec.mjs
├── sail-host-styles.ui.spec.mjs      ← :host rewrite works
├── sail-prop-types.ui.spec.mjs       ← typed prop validation at runtime
└── sail-error-quality.ui.spec.mjs    ← bad input produces good errors
```

Each spec tests one framework feature in a real browser. Specs run via `just test-loops-ui`. Total: ~30 specs, ~5 minutes to run all.

**Coverage target:** every gap I hit during the original SFC build (camelCase signals, `:host`, `data-for`, `data-on:`, `data-bind:value`) has a regression spec. Future bugs add specs as we encounter them.

## Layer 4: token-efficiency benchmark

The differentiated test. The marketing claim. The falsifier of "is this rewrite worth it."

### The corpus

Eight component-spec problems, drawn from real Ship UI patterns but framework-agnostic. Each has:

- **Spec** — ≤200-word natural-language description of what to build
- **Acceptance criteria** — ≤10 browser-testable items (e.g., "component renders without errors", "clicking the button increments the counter", "Escape closes the modal", "list re-renders when items change")
- **Stub schema** — for backends that need it (e.g., a `users.list()` endpoint typed as `User[]`)

```
tests/agent-bench/corpus/
├── 01-static-info-card/
│   ├── spec.md
│   ├── acceptance.json
│   └── stubs.json
├── 02-typeahead/                     ← debounced search, server fetch, keyboard nav, select event
├── 03-modal-with-focus-trap/         ← open/close API, ESC, click-outside, tab trap
├── 04-sortable-paginated-table/      ← column sort, page size, server data
├── 05-live-status-board/             ← long list updating via SSE, status badges, filter
├── 06-form-with-validation/          ← multi-field, async server validation, optimistic submit
├── 07-editable-list/                 ← drag-reorder, inline edit, persist, rollback
└── 08-tabbed-interface/              ← N tabs, keyboard nav, deep-link active tab
```

### The frameworks compared

```
tests/agent-bench/frameworks/
├── sail/                              ← ours (the candidate)
│   ├── runner.ts
│   └── primer.md                      ← framework primer agents see
├── solid/
├── svelte5/
├── nextjs-app-router/
├── htmx-alpine/
├── dash/
└── (each provides a runner.ts that knows how to scaffold + test in that framework)
```

Each framework has a runner that:
1. Sets up a project skeleton
2. Provides the agent with a primer (framework's docs in compressed form)
3. Receives a generated component
4. Tries to compile / run it
5. Runs the acceptance criteria via Playwright

### The experiment

For each (problem × framework × agent):

1. **Author run.** Prompt the agent with `spec.md` + `primer.md`. Measure:
   - Tokens in the agent's prompt (input)
   - Tokens in the agent's output (component code)
   - Total revision rounds before passing OR giving up
   - Total tokens across all rounds
   - Did it ever pass?

2. **Read run.** Give the agent the working component plus a small change request ("add a clear-all button"). Measure tokens-to-correct-edit.

3. **Lint run.** Inject a deliberate error (typo, type mismatch). Measure tokens-to-fix.

4. **Browser verification.** Each result runs through Playwright against the acceptance criteria. Failure modes (compile / runtime / behavior) tracked.

### The agents

- Claude (Opus 4.7, Sonnet 4.6 — multiple sizes)
- GPT-5
- Codex

Each agent runs each framework's primer fresh. We're benchmarking the framework, not the agent.

### Pass criteria

Sail is "easiest" if:
- Median tokens-to-passing ≥30% lower than the next-best framework
- First-try success rate ≥80% (vs whatever the others land at)
- Tokens-to-edit ≥40% lower (read run)
- Tokens-to-fix ≥30% lower (lint run)
- Lint fix correctness ≥95%

If we can't show those numbers, **the native build is unjustified**. We tighten the spec or revert to fixing what we have. **This is the falsification gate.**

### Methodology rigor

- Each (problem × framework × agent) runs ≥10 times. Outliers excluded.
- Prompts are version-controlled. Regen against new agent versions ≥quarterly.
- Browser tests are the gold standard — "passed" means the acceptance criteria are green, not that the agent claimed to be done.
- All runs logged with full transcripts; reproducible.

### What this becomes

The benchmark is open source. Lives at `github.com/sail-framework/agent-bench` (or under Ship's org if simpler). Methodology + corpus + results published on the Sail website.

Other framework teams can submit their own runner implementations. If Solid lands a refinement that closes the gap, we update the leaderboard. **The benchmark is the marketing.** "Sail is 47% fewer tokens than the next-best framework on this published benchmark, here's how to verify" is a much stronger pitch than "Sail is easy because we say so."

## Layer 5: visual + accessibility regression

Less novel but mandatory. Per route + per component:

- Visual regression via Playwright screenshots (Percy, Chromatic, or hand-rolled diff)
- A11y rules via axe-core in headless Chromium
- Keyboard navigation paths
- Color contrast checks

Not Sail-specific; standard web app discipline. The point is: Sail's compile-time guarantees ([01-language-spec.md § Type safety](01-language-spec.md#type-safety)) catch what they catch; visual + a11y regression catches what types can't.

## Layer 6: performance benchmarks

For "is Sail fast" claims:

### js-framework-benchmark

The standard browser-rendered framework benchmark (krausest). Rows: create / update / partial update / select / swap / remove. Tests in 1k and 10k row scales.

Sail's target: within 1.10-1.15x of vanilla. Solid is ~1.05x; Svelte 5 ~1.10x; React 19 ~2-3x. We aim for the Solid/Svelte tier.

Run: weekly on CI, results published on Sail's docs site.

### Patch protocol latency

For the canvas use case, end-to-end patch latency:

```
tests/perf/patch-latency.spec.mjs
- Spawn a daemon
- Open a Sail component subscribed to a `data live` signal
- Daemon emits a patch
- Browser applies the patch
- Measure: emit timestamp → DOM update timestamp
```

Target: <16ms p99 (one frame at 60Hz). Ideally <5ms p50.

Run: weekly. Regression alerts if median exceeds budget.

### Memory + bundle

- @sail/runtime: <13KB minified+gzipped
- Per-component module: <2KB minified+gzipped for typical components
- Total runtime + 10 components for a non-trivial page: <30KB
- Memory: <5MB heap for a 100-component page

Run: per-release, posted with release notes.

## What this all gives us

A defensible, auditable, reproducible claim that Sail is the easiest reactive web framework for agent-authored apps:

- **Compiler tests** prove the language spec is implemented correctly
- **Runtime tests** prove reactive primitives work correctly
- **Integration tests** prove features work in a real browser
- **Token benchmark** proves the framework wins on the agent-authoring criterion that matters most
- **Visual + a11y tests** catch what types can't
- **Performance benchmarks** prove "fast" claims with numbers

The benchmark is the central piece. Without it, every claim is asserted. With it, the claim is a number against a public corpus that any agent / framework can be re-run against. **That's how we win the architecture argument.**

## Implementation phasing

- **Week 1-2:** compiler corpus tests as the compiler is built (covers spec implementation)
- **Week 3:** runtime unit tests as the runtime is built
- **Week 4:** integration Playwright specs (one per spec section) — written alongside migration of existing components
- **Week 5-6:** token-efficiency benchmark — corpus, runners, multi-agent harness
- **Week 7:** perf benchmarks (krausest, patch latency)
- **Week 8+:** visual + a11y regression as views migrate

The benchmark is the highest-leverage piece for marketing/positioning. The compiler + runtime tests are mandatory before any release.

## Non-negotiables

- **Every spec feature has a Playwright spec.** No exceptions. The first SFC framework's pain came from features that compiled but were broken in browser.
- **The token benchmark runs before announcing.** If we can't show 30% wins, we don't ship.
- **The benchmark methodology is public.** No private numbers. Any framework team should be able to challenge or improve the corpus.
- **Performance regressions block releases.** A perf test gate prevents accidental slow-downs.

This is the test plan that turns Sail from "an interesting design" into "a measurably-better framework."
