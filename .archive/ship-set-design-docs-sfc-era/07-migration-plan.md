# Migration plan

How we get from the current state (Jinja views + first-cut SFC components) to Sail-only authoring without disrupting in-progress work.

**Important context:** zero views in this codebase have been signed off as production-complete. Existing components and views work but are not load-bearing. The migration is a clean rewrite informed by what we built, not a careful preserve-everything operation.

## Current state inventory

```
.ship/views/                          ← Jinja-rendered routes
├── chat/index.html.j2                  - existing chat (in-progress)
├── pipelines/index.html.j2             - existing (working)
├── pipelines-board/index.html.j2       - new, mounts <app-pipeline-board>
├── mesh/index.html.j2
├── agents/...
├── git/index.html.j2
├── settings/...
└── ... (~20 view directories)

.ship/components/                     ← First-cut SFC components
├── app-agent-modal.set                ← (will move to .set in migration)
├── app-agent-thread.set
├── app-agent-composer.set
├── app-agent-picker.set
├── app-pty-attach.set
├── app-pipeline-board.set
├── app-pipeline-step.set
├── app-typeahead.set                  ← demo
├── ship-tabs.set                      ← stress test
├── ship-tabs.css
├── ship-tab.set
└── ship-tab-panel.set

apps/shipd/src/views/sfc/             ← Current SFC compiler (Datastar bridge)
└── (~2000 lines, recently fixed but architecturally deprecated)

packages/ship-set/                    ← Lit components (stay)
└── (~200 components, no migration)
```

The `app-*` components have file extension `.set` already (per the previous redesign discussion). For this plan, treat them as legacy-shaped Sail-prep — they need real Sail-compiler treatment, but they're closer to the destination than `.html.j2` views are.

## Migration phases

### Phase 0 — ship-set compiler + runtime MVP (week 1-2)

**Goal:** the ship-set compiler exists and produces a working component end-to-end.

- Build `crates/ship-set-compiler/` (Layer 2 per [02-compiler-architecture.md](02-compiler-architecture.md))
- Build `packages/@ship-set/runtime/` (Layer 1 per [03-runtime-architecture.md](03-runtime-architecture.md))
- Build `crates/ship-set-ship-source/` (Layer 3 — Ship's SchemaSource adapter, see [05-server-data-bridge.md](05-server-data-bridge.md))
- One end-to-end Sail component: a fresh `<app-counter>` rewrite of the simplest working example
- Playwright spec proves it renders + reacts in browser
- Daemon serves `/_ship-set/components/<tag>.js` alongside the existing `/_ship/components/<tag>.js` (both endpoints active during transition)

**Done when:** `<app-counter>` is browser-verified, signal updates work, prop binding works, basic event handler works.

**Parallel impl:** Old SFC compiler still serves `.ship`-extension files. ship-set compiler serves `.set` extension files (formally — even though the existing `.set` files were authored against the old shape).

### Phase 1 — Migrate existing components (week 3)

**Goal:** every `app-*` component is a real Sail component, browser-verified.

The migration tool — `cargo run -p shipd migrate-sfc <file>` — does the mechanical translation:

| Old (legacy SFC) | New (Sail) |
|---|---|
| `const $count = signal(0)` | `state count = 0` |
| `const $double = derived(() => ...)` | `derived double = expr` |
| `const $resp = data(({signal}) => fetch(...))` | `data resp = api.endpoint(...)` (or fall back to imperative effect) |
| `effect($x, () => ...)` | `effect on x { ... }` |
| `data-text="$x"` | `{x}` |
| `data-show="$x"` | `{#if x}...{/if}` (or `:show` attribute if simple) |
| `data-for="a in $list"` | `{#for a of list}...{/for}` |
| `data-on:click="fn"` | `on:click={fn}` |
| `data-attr:foo="$x"` | `foo={x}` |
| `data-bind="$x"` | `bind:value={x}` |
| `:host { ... }` | `:host { ... }` (compiler handles same way) |
| `props.X` | `X` (bare identifier in declarations + template) |

The migration tool runs the AST through these rules + emits the new file. ~80% mechanical; ~20% requires hand-finishing.

**Per-component checklist:**
1. Run migration tool: `migrate-sfc app-agent-modal.set` (overwrites or emits .new file)
2. Compile against ship-set compiler: `TODO(Phase 4): cargo run -p ship-set-compiler check app-agent-modal.set`
3. Browser-test via Playwright: `tests/loops/ship-set-agent-modal.ui.spec.mjs`
4. Smoke-test in dev shell: open the modal, send a message, verify

The 8 existing components (~1500 LOC total) take 1-2 days hand-finishing after the auto-migration. We delete the old SFC compiler's per-file path support for these (they're `.set` now, not `.ship`).

**Done when:** all `app-*` components compile + browser-test green via ship-set compiler.

### Phase 2 — Migrate views (week 4-5)

**Goal:** every route is a `.set` file. Jinja `.html.j2` files retire from the views directory.

Two flavors of view file:

**Simple views** (most): mostly Jinja interpolation, light Datastar attributes, no complex client-side reactivity.

```html
<!-- chat/index.html.j2 (legacy) -->
{% from '_ship.html.j2' import head %}
{% set ctx = ship.chat.state(...) %}
<div class="chat-view">
  <h1>{{ ctx.title }}</h1>
  {% for msg in ctx.messages %}
    <div class="msg">{{ msg.text }}</div>
  {% endfor %}
</div>
```

→

```html
<!-- chat/index.set (new) -->
<script lang="ts">
  data state = ship.chat.state({})
</script>

<div class="chat-view">
  <h1>{state.title}</h1>
  {#for msg of state.messages}
    <div class="msg">{msg.text}</div>
  {/for}
</div>
```

Auto-migration:
- Strip `{% from '_ship.html.j2' import ... %}`
- `{% set ctx = ship.x.y(...) %}` → `data state = ship.x.y(...)` declaration
- `{{ x }}` → `{x}`
- `{% if %}/{% else %}/{% endif %}` → `{#if}/{:else}/{/if}`
- `{% for x in y %}` → `{#for x of y}`
- Datastar attributes → Sail bindings (per the Phase 1 mapping)

A migration tool (`migrate-jinja-view`) handles 80%. The remaining 20% (deeply nested macros, dynamic includes, complex conditionals) needs hand attention.

**Complex views** (few): macros with logic, partial includes, agent-rendered Jinja.

`_ship.html.j2`'s ~600 lines of macros become a few utility components or `.set` snippet imports. Many macros (`signal`, `bind`, `on`, etc.) become unnecessary because ship-set's syntax is the equivalent. Some (like the `head` macro that emits the head tag bundle) become a `<ship-set-head>` component or a per-route auto-emission.

**Per-view checklist:**
1. Auto-migrate: `migrate-jinja-view chat/index.html.j2` → emits `chat/index.set`
2. Compile + lint: `ship-set-compiler check chat/index.set`
3. Hand-finish any leftover Jinja constructs
4. Browser-test: route renders, mounts components, behaves correctly

The 20 view directories take 1-2 weeks hand-finishing. Daemon prefers `.set` if both extensions exist for a route.

**Done when:** every route serves from `.set`. No `.html.j2` files in `.ship/views/`. The Jinja renderer continues to compile but no view directory uses it.

### Phase 3 — Retire legacy SFC compiler (week 6)

**Goal:** delete the first-cut SFC framework. Sail is the only component framework.

- Delete `apps/shipd/src/views/sfc/` (~2000 lines)
- Delete `/_ship/components/<tag>.js` route — only `/_ship-set/components/<tag>.js` remains
- Delete Datastar's bundle from production (or shrink to a tiny bundle if any non-component view still uses Datastar morph)
- Delete the legacy `app-*-test.spec.mjs` Playwright tests; replace with `ship-set-*` versions
- Update Justfile: no more `test-loops` distinction between SFC and Sail; everything is Sail
- Update docs: link to spec files, remove references to old framework

**Parallel impl gone.**

**Done when:** `git grep -i datastar` returns only docs/historical notes. `git grep "/_ship/components/"` returns nothing except docs.

### Phase 4 — Sunset Jinja for components (week 7+, optional)

**Goal:** Jinja becomes a non-component-only tool. `_ship.html.j2` macros retire entirely or move to a non-component template library.

If after Phase 2 there are still Jinja-rendered surfaces (admin pages, email templates, generated reports, OpenAPI docs), keep the Jinja renderer for those. The component story is fully on Sail; Jinja persists for non-component templating only.

If everything that used Jinja can be expressed as `.set` files (most cases), delete the Jinja renderer and the `_ship.html.j2` macros.

**Done when:** Jinja is either retired entirely or scoped to non-component contexts only.

## Risk and mitigation

### Risk: migration tool produces broken output

The auto-migration regexes can miss edge cases — nested control flow, complex Jinja macros, weird Datastar attribute combinations.

**Mitigation:**
- Every migrated file goes through Playwright before old version is deleted
- Migration tool emits `.set.new` initially (doesn't overwrite); diff is reviewable
- Bug list as encountered: each broken case becomes a regression test in the migration tool
- We hand-finish where the tool fails; gradually improve the tool

### Risk: ship-set components incompatible with Sail's slot model

ship-set components use shadow DOM + `<slot>`. Sail uses light DOM + a slot resolver.

**Mitigation:**
- Both work — ship-set's shadow slots distribute their own content; Sail's light slots distribute light children. They don't interfere.
- If a Lit component expects to slot-distribute Sail children, the children just sit in the host's shadow tree's slot. Standard browser behavior.
- Schema-emission for ship-set lists slot names; ship-set compiler validates `<el slot="X">` against them.

### Risk: tokens.css migration breaks visual design

The legacy SFC components reference tokens via direct `var(--ship-color-X)`. Sail's `${color.X}` is the new way. Both paths work simultaneously during migration.

**Mitigation:**
- `tokens.css` is unchanged; the `var(--ship-color-X)` form continues to work
- ship-set compiler's token interpolation is additive; existing components keep working
- Visual regression tests via Playwright catch unintended changes

### Risk: agents author against the wrong framework

During the parallel-impl window (weeks 3-5), agents could author `.ship` files instead of `.set`. This wastes work and creates more migration surface.

**Mitigation:**
- CI gate: any new file with `.ship` extension fails the build (post Phase 1)
- Agents see a clear `CLAUDE.md` instruction and a `sail.config.ts` that points to the new framework
- The migration tool is run on any leftover `.ship` files at PR time

### Risk: data declarations don't yet match every existing query

Some legacy views use `ship.x.y()` Jinja calls that may not have manifest entries yet.

**Mitigation:**
- Audit ship's SDK manifests before Phase 2
- Add missing manifest entries as we encounter them
- Fall back: a `data raw = fetchJSON('/api/v1/...')` form for queries without typed manifests

## Schedule

| Week | Phase | Output |
|---|---|---|
| 1-2 | Phase 0: compiler + runtime MVP | First Sail component browser-verified |
| 3 | Phase 1: migrate existing components | All `app-*` components on Sail |
| 4-5 | Phase 2: migrate views | All routes on Sail |
| 6 | Phase 3: retire legacy compiler | Old SFC framework deleted |
| 7+ | Phase 4: Jinja sunset (optional) | Jinja kept only for non-component use |

**Total: 6-7 weeks for full migration.** Worst-case parallel implementation lasts 3-4 weeks (Phases 1-3) when both old SFC and Sail compile.

## Tools we need to build

1. **`migrate-sfc`** — turns a legacy SFC `.ship` file (or `.set` authored against the old shape) into a real Sail component. Lives in `crates/sail-migrate/` or as a subcommand of `shipd`.

2. **`migrate-jinja-view`** — turns a `.html.j2` Jinja view into a `.set` route file. Same crate.

3. **`ship-set-compiler check`** — CLI for the compiler. Used by CI gates and authors verifying changes.

4. **Sail Playwright spec generator** — given a `.set` file, scaffold a Playwright spec that exercises the component's documented surface. Saves time writing per-component browser tests.

## Checklist gates

Each phase has a gate before moving to the next:

**Phase 0 → 1:**
- [ ] ship-set compiler compiles a hand-written test file
- [ ] Sail runtime mounts the component in a real browser
- [ ] Reactivity works (signal change → DOM update)
- [ ] One Playwright spec proving the above

**Phase 1 → 2:**
- [ ] All 8 `app-*` components compile via Sail
- [ ] All 8 components browser-test green
- [ ] Migration tool's auto-migration succeeded for each (manual fixes documented)

**Phase 2 → 3:**
- [ ] All routes serve `.set` (no `.html.j2` in `.ship/views/`)
- [ ] Loop tests still green (substrate paths)
- [ ] Visual smoke-test of every route

**Phase 3 → 4:**
- [ ] Legacy SFC compiler deleted
- [ ] No `apps/shipd/src/views/sfc/` references
- [ ] CI gates that block any new `.ship` file

## After migration

What the codebase looks like:

```
.ship/views/                          ← all .set files
├── chat/index.set
├── pipelines/index.set
├── mesh/index.set
└── ...

.ship/components/                     ← all .set files
├── app-agent-modal.set
├── app-agent-thread.set
└── ...

crates/ship-set-compiler/                 ← OSS, the framework
crates/ship-set-ship-source/              ← Ship's SchemaSource adapter
packages/@ship-set/runtime/               ← OSS, the runtime
packages/ship-set/                    ← Lit components (unchanged)

apps/shipd/src/views/                 ← Ship's view-serving glue
├── sail.rs                             - serves /_ship-set/components/<tag>.js
└── routes.rs                           - filesystem routing for .set routes
```

One framework. One file format. One mental model. The legacy SFC compiler retired; Datastar dropped from production; Jinja kept only where it serves non-component templates.

This is the framework you actually want to write apps in.

## Why this migration is realistic

- The first-cut SFC framework's bugs (`data-on:`, `data-for`, `:host`, camelCase signals) are each a structural mismatch with Datastar — fixing them is the same scope as building Sail's compiler. Rebuilding gets us forward instead of holding pattern.
- The migration tools are tractable (~3-4 days of work for both, including tests).
- The browser-test gate from `tests/loops/agent-modal.ui.spec.mjs` is the template for catching migration regressions.
- Zero production gating — no view is signed off, no user is blocked by the rewrite.
- The Sail framework scales beyond the migration: new components are dramatically easier to author, the patch protocol unlocks the canvas use case, the OSS path opens the framework to broader use.

The migration pays for itself within Ship — the rewrite avoids 4-6 weeks of bridge fixes and unblocks features (typed call sites, patch protocol, server-data integration) that the current framework can't reach. And it gives us an OSS framework as a side effect.
