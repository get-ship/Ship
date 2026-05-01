---
group: Ship UI
order: 3
title: The controller surface
description: The optional `.ts` paired with a template. Three tiers, five sections, scope rules, mutation discipline.
---

# The controller surface

A controller is an **optional** TypeScript file paired with a template by filename. `views/agents/view.sui` ↔ `views/agents/view.ts`. No config, no import statement in the template, no manifest entry — convention is the wiring.

The controller is the view's view-model: derived values, event handlers, lifecycle, and composable hook bundles. **State always lives in the template** via `<state name="X" value="..." />` directives. No subscriptions, no host-side side effects — those live in the template.

## Three tiers

Graduated complexity. Use the smallest tier that fits.

### Tier 1 — no controller

Pure declarative views. The template uses `<state>` for any local signals, `<data>` for any subscriptions, and inline expressions for anything computed.

```html
<!-- views/counter/view.sui -->
<state name="count" value="0" />

<p>{count()}</p>
<button on:click={() => count.set(count() + 1)}>+</button>
<button on:click={() => count.set(0)}>reset</button>
```

No `view.ts`. Most simple views land here.

### Tier 2 — object-literal controller

For views that need named handlers and derived values, but no instantiation-time work. State stays in the template:

```html
<!-- views/settings/view.sui -->
<state name="theme" value="'system'" />
<state name="accent" value="'blue'" />
```

```ts
// views/settings/view.ts
import { controller } from '@SUI/runtime'

export default controller({
  on: {
    setTheme:  (s, v: 'light' | 'dark' | 'system') => { s.theme = v },
    setAccent: (s, v: string)                      => { s.accent = v },
  },
})
```

Plain object. Static analysis is trivial — every section is data. Sections can also be functions when they need scope:

```ts
export default controller({
  derived: {
    filtered: ({ agents, query }) =>
      agents().value?.items.filter(a => a.name.includes(query())) ?? [],
  },
  on: {
    setQuery: (s, q: string) => { s.query = q },
    refresh:  ({ ship })     => ship.agents.refresh(),
  },
})
```

Per-section functions are how the controller pulls from the [shared view scope](index.md#the-shared-view-scope). The runtime calls each with `{ ship, ...templateScope, ...derivedScope }`; sections destructure what they need.

### Typing the template scope (one declaration, every section gets it)

The cheapest way to avoid annotating every section's destructure: declare the template-side identifiers once and pass them as the `controller<Scope>` generic. Sections inherit the type:

```ts
import type { LiveSignal, Signal } from '@SUI/runtime'
import type { GitOverview } from '@SUI/sdk-client'

/** Everything the .sui declares — `<data>`, `<state>`,
 *  `<use-* as=>` identifiers — that this controller reads.
 *  Writable signals (template `<state>`) become MutableState keys. */
type Scope = {
  overview: LiveSignal<GitOverview>
  tab: Signal<'changes' | 'history' | 'branches'>
}

export default controller<Scope>({
  derived: {
    branch:  ({ overview }) => /* overview is typed automatically */,
    clean:   ({ overview }) => /* same */,
    // ...15 more, none with per-section annotations
  },
  on: {
    showHistory: (s) => { s.tab = 'history' },  // s typed as MutableState<Scope>
  },
})
```

`MutableState<Scope>` is computed from `Scope`: it picks every key whose value is a `Signal<T>` and exposes that key as an assignable `T`. There's no separate state-shape generic to declare — the template's `<state>` directives are the single source of truth.

The compiler emits a `<file>.sui.d.ts` declaration alongside each compiled template, exporting a `Scope` interface derived from the template's `<data>` / `<state>` / `<use-* as=>` elements. Import it as the controller generic to replace the manual `type Scope = { … }` declaration.

### Tier 3 — function controller

For views that need to close over instantiation-time variables, share computed references across sections, or compose multiple controller hooks:

```html
<!-- views/agents/view.sui -->
<state name="searchOpen" value="false" />
```

```ts
// views/agents/view.ts
import { controller, useFilteredList, useSelection } from '@SUI/runtime'

export default controller(({ ship }) => ({
  use: {
    list: useFilteredList(/* see hooks.md */),
    sel:  useSelection(/* see hooks.md */),
  },
  on: {
    toggleSearch: (s) => { s.searchOpen = !s.searchOpen },
    refresh:      ()  => ship.agents.refresh(),
  },
}))
```

The outer function receives `{ ship }` (and any other instantiation-time context) and returns the controller object. Sections still destructure their own scope as in tier 2.

## The five sections

| Section | Type | Purpose |
|---|---|---|
| `derived` | record of `({ scope }) => value` | Pure memos. Auto-tracked dependencies. |
| `on` | record of `(state, ...args) => void` or `({ scope }, ...args) => void` | Event handlers. The **only** section that mutates state. |
| `use` | record of hook results, OR `({ scope }) => ({ name: hookResult })` | Composition of controller hooks. Each result is namespaced by key. |
| `mount` | `(state, el) => void` | Runs once after the DOM is in place. For wiring third-party libs, observers, computed-initial state assignments, or any one-shot imperative setup. |
| `unmount` | `(state) => void` | Runs on disconnect. Tear down whatever `mount` set up. |

State is **not** a section. Declare it in the template via `<state name="X" value="..." />`. Computed-initial cases (`crypto.randomUUID()`, an SDK call, a derivation over `route`) move to `mount`:

```html
<state name="id" value="''" />
```

```ts
export default controller({
  mount: ({ state }) => {
    state.id = crypto.randomUUID()
  },
})
```

No other sections exist. No `effect`, no `watch`, no `computed`, no `data`, no `live`, no `task`. If your case doesn't fit one of the five, see [the discipline](index.md#the-discipline) — the answer is almost always a combination of the existing sections, not a new section.

## Scope rules

Each section is invoked with the [shared view scope](index.md#the-shared-view-scope) as its argument. The shape of that argument differs slightly by section:

| Section | Argument shape | Mutation |
|---|---|---|
| `derived` | `{ ship, ...allSignals }` | **read-only** state proxy |
| `on` (handler) | `(stateMutator, ...args)` OR `({ ship, ...allSignals }, ...args)` | **writable** state proxy in the first arg |
| `use` | `{ ship, ...allSignals }` | hooks are factories — see [hooks.md](hooks.md) |
| `mount` / `unmount` | `(stateMutator, el?)` | writable state proxy |

The mutation rule is a static guarantee: in `derived` and templates, every `<state>` key is callable (`count()`); in `on` / `mount` / `unmount`, the same key is assignable (`s.count = …`). **You cannot mutate state from a derived or a template.** Every state change has a name (the `on.*` key, or the lifecycle hook) and a single call site.

## What's NOT in the controller

For clarity — these go in the template, never the controller:

- **Local state**: declare it in the template with `<state name="X" value="..." />`. The controller reads it through scope and mutates it through handlers, but the declaration site is always the template.
- Server data subscriptions (use `<data>` in the template).
- Host-side side effects like keybindings, document title, focus traps (use `<use-*>` template hooks).
- DOM structure (templates own structure, full stop).

If you find yourself wanting to construct DOM in `mount`, you almost certainly want a template hook or a child component instead.

## File layout

Most common shape:

```
views/<name>/
  view.sui    # template (required)
  view.ts          # controller (optional)
  view.css         # styles (optional)
```

Component files use the same convention. The file IS the component; `<prop>` directives at the top declare the element's API; a `<tag>.sui.json` metadata file names the tag and shadow flag:

```
packages/SUI-components/src/ship-button/
  ship-button.sui      # template + <prop> directives
  ship-button.sui.d.ts # compiler-emitted Scope declaration
  ship-button.ts       # controller (only if needed)
  ship-button.css
  ship-button.sui.json # { "component": "ship-button", "shadow": true }
```

The compiler pairs by filename; no manifest entry is needed. Adding a `view.ts` next to a `view.sui` is the only signal the framework requires.

## Tier-1 / tier-2 threshold rule

The compiler enforces condition (1) of the threshold rule statically. A **multi-statement** `on:event` handler body in a tier-1 template (no paired `.ts`) is a build-time error:

```html
<!-- ✗ fails without view.ts -->
<button on:click={() => { x.set(1); y.set(2) }}>Save</button>
```

```
view.sui: line 1 col 30: [on:click handler]
multi-statement handler — paired `<view>.ts` controller required for tier-2 work
```

A single-expression or single-statement block is fine at tier 1:

```html
<!-- ✓ OK — single expression -->
<button on:click={() => count.set(count() + 1)}>+</button>

<!-- ✓ OK — single-statement block -->
<button on:click={() => { count.set(count() + 1) }}>+</button>
```

Once you add a paired `view.ts` the check is skipped entirely — the controller owns complex handler logic. Conditions 2–5 of the threshold rule (multi-signal data thunks, `use:` compositions, lifecycle hooks, multi-statement `<state>` mutations) are not yet enforced statically but follow the same architectural boundary.
