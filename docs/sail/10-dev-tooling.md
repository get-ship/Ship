# Sail dev tooling

What an author sees, hears, and runs when building Sail apps. The dev tooling is part of the framework's value — co-designed with the language, the runtime, and the platform.

## North star

**The author saves a `.sail` file. The browser shows the change. No build to wait for, no error to decode, no terminal to babysit.**

Everything below collapses to that loop.

## Tier 1 — what every author needs (ship with v1)

### LSP (`@sail/lsp`)

A Language Server Protocol implementation that backs every editor. Full-fidelity IDE support is the single biggest agent-authoring win because it eliminates round-trips for "is this prop name right?" / "does this signal exist?" / "what events does this component emit?".

Capabilities:
- **Syntax highlighting** for `.sail` files (script as TS, template as HTML+`{expr}`, style as CSS+`${tokens}`)
- **Completion** for:
  - Prop names + values when typing `<my-component …>`
  - Signal names inside `{expr}` interpolations and bindings
  - Event names in `on:…` (validated against the target component's schema)
  - Token names in `${color.surface}` (validated against the configured token set)
  - `data` query paths (validated against the schema source — `ship.agents.list` autocompletes once `ship.agents.` is typed)
  - Slot names in slot fills
- **Diagnostics** in real time:
  - Unknown signal references → red squiggle, "no `state $foo` declared"
  - Type mismatches in component props
  - Unknown design tokens
  - Unterminated control-flow blocks
  - Schema-source errors ("`ship.agents.list` requires arg `workspace`")
- **Hover info** — type signatures, doc comments, link to spec sections
- **Go-to-definition** — `state foo`, `<my-component>`, `${color.X}` all jump
- **Rename** — symbol-aware refactor across the workspace (rename `state x` → `state count`, all references update)
- **Code actions** — auto-import missing components, "extract to component", "convert to derived"
- **Inline values** during debugging (signal cache value shown next to the declaration)

Implementation: TypeScript, runs as a standard LSP server. Editors connect via the standard protocol. Lives at `crates/sail-lsp/` (Rust thin wrapper that spawns the TS LSP) or `packages/@sail/lsp/` (pure TS). TBD on language; pure TS is faster to build, Rust ties cleaner into the compiler.

The LSP **shares its parser + analyzer with the compiler** — same code, different consumer. The LSP doesn't have its own AST. This means LSP correctness flows from the compiler's correctness.

### CLI (`@sail/cli`)

One binary, common verbs:

```
sail check [path]           # type-check + lint, no emit; CI gate friendly
sail build [path]           # full compile, emit JS/SSR/dts/schema to dist/
sail dev                    # watch mode, hot-reload templates+styles
sail format [path]          # opinionated formatter (Prettier-shaped)
sail migrate                # tools to migrate from .ship/.html.j2/Svelte/React
sail bench                  # run the agent-token benchmark
sail new <template>         # scaffold a new project / component
```

Implementation: TypeScript binary, Cargo binary, or both — the compiler is Rust, the runtime is TypeScript, so the CLI bridges. Probably TypeScript wrapping a `sail-compiler` Rust subprocess (or via napi-rs bindings).

### Hot reload (template + style)

Today's daemon already does this for `.ship` SFCs via Datastar morph. Sail extends:

- File watcher in the daemon (or `sail dev`) detects `.sail` mtime change
- Compiler re-emits the affected component
- Daemon pushes a `component:update` SSE event with the new component module URL
- Browser swaps the `customElements` definition (or, if HMR isn't yet implemented for scripts, falls back to `location.reload()` for script changes only — templates and styles morph in place)

For templates + styles: **<200ms save-to-paint round trip.** Author edits, saves, browser updates without losing scroll, focus, or signal state.

For script changes in v1: page reload. Script HMR (custom-elements re-registration with state preservation) is a v1.5 feature.

### Devtools panel (`<sail-devtools>` component)

A web component that mounts in any Sail app's dev mode. Shows:

- **Component tree** — hierarchical view of mounted Sail components, click to inspect
- **Signal inspector** — per-component, list every signal with current value; toggle to mutate from devtools
- **Event log** — all CustomEvents fired in the page, with payloads
- **Patch monitor** — live SSE patch stream, with filtering by signal name + op type
- **Render counter** — per-component render count, for finding hot spots
- **Token panel** — all design tokens with their current values, contrast checks

Mounts in the existing Ship dock alongside `<ship-devtools-source>` etc., and exports as a standalone web component for non-Ship deployments. ~600 LOC of UI on top of the runtime's introspection API.

### Error reporting

Errors speak Sail's vocabulary, point at file:line:col, link to docs. Three sources:

1. **Compile errors** — emitted by `sail-compiler` as JSON for the LSP, formatted text for the CLI:
    ```
    sail-compiler: error[SAIL/E0042]: prop 'count' expects number, got string
       ╭─ agent-modal.sail:14:35
       │
    14 │   <my-counter count="not a number">
       │                     ^^^^^^^^^^^^^^ — type mismatch
       │
       = help: pass a number expression: `count={5}`
       = note: defined at counter.sail:3:1
    ```
2. **Runtime errors** — `[sail:my-component] script error at line 23: ReferenceError: undefinedVar is not defined`. Source-mapped back to the `.sail` source.
3. **Schema source errors** — when `data ship.agents.list()` is called but the daemon doesn't recognize it: `[sail:data] unknown schema reference 'ship.agents.list' — did you mean 'ship.agents.detail'?`.

Each error code (`SAIL/E0042`, `SAIL/W0011`, etc.) is documented at `docs.sail-framework.dev/errors/E0042` with a longer explanation, examples, and a fix.

## Tier 2 — what makes authoring fun (v1.5 - v2)

### Script HMR

Today: edit `<script>`, page reloads.
v1.5: SSE pushes the new script module, browser unregisters the old custom element, registers the new one, walks all instances and re-mounts each (preserving DOM position via `replaceWith`). Signal state is preserved when names match; new signals get their initial value; removed signals are dropped.

Implementation: ~150 LOC in the runtime + ~50 LOC in the daemon's hot-reload SSE channel.

### Time-travel debugging

Every signal mutation is logged in dev mode. Devtools panel shows the timeline; click a past mutation to "rewind" the signal store to that snapshot. Useful for understanding "what triggered this re-render?" or reproducing flaky bugs.

Implementation: ring buffer of patch logs in the runtime + UI in the devtools panel.

### Visual component inspector

Right-click any DOM node in dev mode → "Inspect Sail component" opens the relevant `.sail` source in your editor at the right line. Bridge: devtools sends a `sail:open-source` message to the dev daemon, which uses the editor's URL handler (`vscode://file/...`).

### Test harness — `@sail/test`

```ts
import { mount, fire, on } from '@sail/test'
import MyButton from './my-button.sail'

it('emits select on click', async () => {
  const { host, on: listen } = mount(MyButton, { variant: 'primary' })
  const events: CustomEvent[] = []
  listen('select', (e) => events.push(e))
  await fire.click(host.querySelector('button')!)
  expect(events.length).toBe(1)
  expect(events[0].detail).toEqual({ id: 'something' })
})
```

`mount(Component, props)` instantiates the component in jsdom (or browser via Playwright), returns a handle for assertion + interaction. Cleanup automatic.

Without this, every component is a liability. With it, refactoring framework internals or migrating from Svelte/React/etc. is bounded work.

### VS Code extension

`sail-vscode` packages the LSP + syntax theme + snippets + commands ("Sail: New Component", "Sail: Migrate from Svelte", "Sail: Run Benchmark"). Published to the VS Code marketplace. JetBrains/Vim/Helix support via standard LSP.

### Storybook-shaped showcase

`sail showcase` scaffolds a per-component playground page. Each `.sail` file gets a `.showcase.sail` sibling (optional) with prop variations. The showcase auto-mounts each variant; designers can browse the catalog visually. Lives at `localhost:9315/_sail/showcase/` in dev mode.

## Tier 3 — production-grade DX (v2+)

### Performance profiler

Per-component render time, signal-update fan-out cost, patch-apply latency. Surfaced in the devtools panel. CI integration: a perf budget per component, regression gate.

### A11y auditor

Built-in axe-core integration. Errors surfaced in the devtools panel + CI gate. "Your `<button>` has no accessible name" type warnings, sourced back to the `.sail` line that emits the offending element.

### Visual regression harness

Per `.sail` component: snapshot a default + variant set; CI diffs against committed images, blocks merges on visual changes without explicit accept.

### Bundle analyzer

`sail build --analyze` produces an interactive treemap: which components contribute how much to the bundle, what's deduplicated, where the wins are.

## Tier 4 — agent-specific tooling (the differentiator)

### `sail bench` — token-efficiency benchmark runner

Per `08-testing-benchmarks.md § Layer 4`. Compares Sail against Solid/Svelte/Next.js/Dash on a fixed corpus. Run locally; published nightly to Sail's leaderboard. **The marketing AND the falsifier.**

### Schema export for agents

`sail schema --format=agent-friendly` emits a compressed manifest of:
- Every component's prop + event + slot schema
- Every available `data ship.x.y(args)` query
- Every design token
- Common patterns (with examples)

Agents (Claude/GPT/whatever) read this manifest before authoring; instructions become "here are your building blocks, compose them" rather than "here's documentation, figure it out". Reduces author tokens dramatically.

### `sail explain <error>` 

`sail explain SAIL/E0042` opens the docs page for that error. `sail explain --tag=ship-button` shows the prop schema + examples. `sail explain --signal=mesh` traces where the `mesh` signal originates and what consumes it. Built for the case where an agent hits an error and needs context fast — equivalent to a human agent grepping the codebase, but instant.

### `sail snapshot <component>` for token-efficient context loading

Given `<component>`, emits a minimal context block that an agent can paste into a new conversation to author against that component. Includes: the component's source, its dependencies' schemas, related component schemas, the relevant ship-set primitives. Agents use this to bootstrap into authoring without ingesting the whole project.

### Agent-aware formatter

`sail format` not only opinionated-formats but **canonicalizes** code shapes that agents commonly produce variantly. Same logic, one shape — minimizes token diff in PRs.

## Implementation phasing

- **Sprint 1 (alongside compiler MVP):** LSP basic syntax + diagnostics + completion. Hot reload for templates+styles. Error reporting framework.
- **Sprint 2:** CLI verbs (`check`, `build`, `dev`, `migrate`). Test harness (`@sail/test`).
- **Sprint 3:** Devtools panel (component tree, signal inspector, event log).
- **Sprint 4:** Script HMR. Visual component inspector. VS Code extension.
- **Sprint 5+:** Tier 3 (perf profiler, a11y, visual regression, bundle analyzer). Tier 4 (agent-specific surfaces).

Each tier is independently shippable. The framework works without any of it, but the experience compounds.

## Why this matters

Most reactive frameworks treat tooling as add-on. Sail treats it as the framework. Three reasons:

1. **Agents need it more than humans do.** A human author can grep, scroll, guess. An agent needs the schema in the prompt and the error in the response. Tooling that pre-digests information is a force multiplier on agent throughput.

2. **The token economy compounds.** Every error caught at compile time is a recovery turn the agent doesn't spend. Every autocompletion is a token of suggestion the agent doesn't have to invent. Every hot-reload is a context-reset the agent doesn't trigger.

3. **The framework's quality is the tooling's quality.** A framework with bad errors and slow feedback loops is a bad framework, regardless of how clever the runtime is. Sail's job is to make the entire authoring experience — language + runtime + tooling — feel like one coherent thing. Co-designed with the daemon. Designed for agents. Built on the web.

That's the dev tooling story.
