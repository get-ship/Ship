---
name: set-authoring
stable-id: set-authoring
description: Author Ship views and components with the `.html.set` framework — single-file HTML templates, optional paired `.ts` controller, signals, server-driven `<data live>` subscriptions, three-role architecture (template + controller + data). The successor to the j2/Datastar stack; **this is how you write new ship UI**. The full reference doc is `docs/ship-set-framework/`; this skill is the authoring cheat sheet plus the "do / don't" rules an agent reaches for while building.
tags: [ship, set, framework, ui, views, components, html-set, authoring]
authors: [ship]
audience: agent
---

# Set authoring — the cheat sheet

Use this skill while writing or porting `.html.set` views and components. The full reference lives at **[docs/ship-set-framework/SKILL.md](../../docs/ship-set-framework/SKILL.md)**; consult it for the *why* behind any decision below.

## The three-role split (memorize this)

Every authoring decision lands in exactly one of three places:

| Role | File | What goes here |
|---|---|---|
| **Template** | `view.html.set` | Structure + declarative wiring (`<data>`, `<state>`, `<use-*>`, `on:`, `bind:`, `{#if}`, `{#for}`) |
| **Controller** | `view.ts` (optional) | Behavior — `state` / `derived` / `on` / `use` / `mount` / `unmount` |
| **Data subscription** | `<data>` element in template | The bridge to `shipd` (`source="ship.X.Y()" [live]`) |

If you can't express a thing inside one of the three, combine two — never invent a fourth role.

## Template surface (all of it)

```html
<!-- subscriptions (server data) -->
<data name="agents" source="ship.agents.summaries()" live />
<data name="schema" source="ship.schema.read('agents')" />

<!-- local state (trivial) -->
<state name="open" value="false" />

<!-- component declaration (only in component-shaped files) -->
<component name="ship-button">
  <prop name="tone" values="default|primary" default="default" reflect />
  <prop name="disabled" type="bool" default="false" reflect />
</component>

<!-- expressions -->
<p>{count() === 1 ? '1 item' : count() + ' items'}</p>

<!-- control flow -->
{#if cond()}…{:else if x()}…{:else}…{/if}
{#for a of items() key={a.id}}…{/for}
{#match status()}{:case 'idle'}…{:case 'loading'}…{:default}…{/match}
{#await promise}…{:then v}…{:catch err}…{/await}

<!-- bindings -->
<button on:click={() => count.set(count() + 1)}>+</button>
<input bind:value={query} />
<input type="checkbox" bind:checked={enabled} />

<!-- live boundaries -->
<while-loading bind="agents"><p>loading…</p></while-loading>
<while-error bind="agents" let:err><p>{err.message}</p></while-error>

<!-- template hooks (host primitives) -->
<use-shell as="shell" />
<use-kbd>
  <on key="/" action="search" />
  <on key="cmd+n" action="newAgent" />
</use-kbd>
<use-click-outside bind:target={dialogEl} bind:action={close} />

<!-- slots -->
<slot name="leading"></slot>
<slot></slot>
```

## Controller surface (six sections, no others)

```ts
import { controller } from '@ship-set/runtime'

export default controller({
  state:    { /* initial values; one signal per key */ },
  derived:  { /* memos; ({ x }) => x() * 2 */ },
  on:       { /* handlers; ONLY mutator: (s, ...args) => { s.x = ... } */ },
  use:      { /* controller hooks composed by namespace */ },
  mount:    (s, el) => { /* once after render */ },
  unmount:  (s) => { /* once on dispose */ },
})
```

**Three tiers — pick the smallest that fits:**

1. **No controller** — pure `.html.set` with `<state>` and inline handlers. Hello / counter / static info.
2. **Object form** — `export default controller({...})`. Most coordinated views land here.
3. **Function form** — `export default controller(({ ship }) => ({...}))`. Use when you need closure-scoped variables (a stable ID, a once-per-mount handle).

## Available controller hooks (compose via `use:`)

```ts
import { useFilteredList, useSelection, useForm } from '@ship-set/runtime'

use: {
  list: useFilteredList(agents, (a, q) => a.name.toLowerCase().includes(q.toLowerCase())),
  sel:  useSelection(agents, (a) => a.id),
  form: useForm({ initial: () => settings(), validate, submit }),
}
```

Templates reference them namespaced: `list.filtered()`, `list.setQuery`, `sel.selected()`, `sel.select`, `form.values()`, `form.errors().<field>`, `form.submit`, etc.

## Available template hooks (use as elements)

| Hook | Purpose | Shape |
|---|---|---|
| `<use-shell as="X">` | Host context (theme, currentUser, app metadata) | Self-closing; contributes `X` to scope |
| `<use-kbd>` | Document keyboard shortcuts | Children `<on key="..." action="onName" />` resolving to controller `on.*` |
| `<use-click-outside bind:target bind:action>` | Dismiss-on-outside | Self-closing; `bind:target` is an element ref, `bind:action` is a handler |

## Hard rules (non-negotiable)

1. **Mutation lives only in `on.*` handlers.** Never `s.x = v` in a derived or template. Type system enforces.
2. **Subscriptions live in `<data>` elements**, never in controller code. Don't call `ship.*` outside `<data>`.
3. **Templates declare; controllers do.** Anything declarative → template. Anything imperative → controller.
4. **No `<script>` blocks in templates.** Full stop. Logic goes in the paired `.ts`.
5. **No `customElements.define`** in view code. That's primitive territory.
6. **Read signals by call:** `count()` reads, `count.set(n)` writes. Live signals expose `.kind` (`'loading'` / `'ok'` / `'error'`); discriminate before reading `.value`.

## Live data envelope

```ts
type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: Error }
  | { kind: 'ok'; value: T }
```

Pattern:

```html
<while-loading bind="agents"><p>loading…</p></while-loading>
<while-error bind="agents" let:err><p>{err.message}</p></while-error>
{#if agents().kind === 'ok'}
  <!-- happy path: agents().value.X -->
{/if}
```

Or equivalently:

```ts
// in controller
derived: {
  items: ({ agents }) => {
    const s = agents()
    return s.kind === 'ok' ? s.value.items : []
  },
}
```

When passing a live signal to a hook, **pass the signal itself, not a closure**. Hooks know the envelope shape.

## File layout

```
views/<name>/
  view.html.set    # template (required)
  view.ts          # controller (optional)
  view.css         # styles (optional, linked via <link rel="stylesheet">)
```

Components live in `packages/ship-set-components/src/<tag>/` with the same triple. (Today's Lit primitives are still in `packages/ship-set/src/` until they're ported.)

## What's phased out — don't author against these

- **`.html.j2` + Datastar** — the j2/Datastar prod stack (`ship-views` skill / `.ship/views/<name>/`). Still ships current production traffic but every new view goes in `views/<name>/` as `.html.set`.
- **`.ship` SFC three-block format** (template + `<script>` + `<style>` in one file). Dead shape. The `ship-components` skill describes it; ignore for new work.
- **`view()` factory** from `@ship-set/runtime`. Removed. Use `controller()`.

## Porting a Lit primitive to a `.html.set` component

Use this recipe whenever you port a `packages/ship-set/src/web-components/core/<X>.ts` Lit primitive to a `.html.set` component under `packages/ship-set-components/src/<set-X>/`.

### The naming convention

Ported primitives use the **`set-` tag prefix** during the migration. `<ship-button>` (Lit) → `<set-button>` (ported). Both ship side-by-side; `.html.set` views consume the `set-*` versions, j2 views keep using `ship-*`. See **[ship-set-prefix-during-migration ADR](../../docs/ship-adrs/references/docs/ship-set-prefix-during-migration.md)** for the full reasoning.

### The trio of files

```
packages/ship-set-components/src/<tag>/
  <tag>.html.set         # template + <component> directive + scoped <style>
  <tag>.ts               # imports the template and calls defineComponent()
```

Generate the skeleton with the scaffolder:

```sh
node scripts/scaffold-set-component.mjs set-foo
```

Then edit the `.html.set` to match the Lit source's prop list, styles, and body template.

### Authoring checklist

For each port:

1. **Open the Lit source.** Note its prop schema, slot list, and CSS surface — `:host(...)` selectors translate cleanly to scoped shadow CSS in the port.
2. **Decide shadow vs light DOM.** Default to **shadow** for primitives that own a tone-matrix or component-scoped CSS (every primitive ported so far). Use light DOM only when the component is intentionally style-by-class within a parent.
3. **Author the `<component>` block** with `shadow` (for shadow DOM) and one `<prop>` per Lit `static properties` entry. Use `reflect` for any prop that drives `:host([attr=...])` selectors — the framework auto-reflects when you mark it.
4. **Inline the `<style>`** block inside `<component>`. CSS lives in the shadow root.
5. **Author the body template** with `{prop()}` interpolation, `{#if}` / `{#for}` / `<slot>` as needed. Match the Lit `render()` shape.
6. **Wire the `.ts`** to call `defineComponent(template)`. The helper splices in any `opts.styles` (when CSS is loaded from a sibling file via the bundler's text loader) and registers the element.
7. **Add the registration** to `src/index.ts` — both the named export and the call inside `registerAll()`.
8. **Add a smoke row** to `test/all-components.test.ts` (one `{ tag, attr, value, shadowPart }` entry exercises register + reflect + shadow part).
9. **Bundle** — `node scripts/build-bundles.mjs --force` rebuilds the components bundle and rebakes `apps/shipd/assets/ship-set-components.js`.
10. **Verify** — `pnpm --filter @ship-set/components exec vitest run`.

### `defineComponent` options

```ts
import { defineComponent } from '../define'
import template from './set-foo.html.set'
import css      from './set-foo.css'    // when CSS is in a sibling .css file

defineComponent(template, { styles: css })
```

The helper splices `<style>${css}</style>` into the template's `<component>` block. Use this for primitives whose CSS exceeds ~50 lines — keep `.html.set` files focused on structure, push styles to the sibling `.css`.

### What ports look like end-to-end

The first thirteen ports — `set-status-dot`, `set-spinner`, `set-badge`, `set-chip`, `set-card`, `set-link`, `set-divider`, `set-skeleton`, `set-stat`, `set-mark`, `set-kbd`, `set-avatar`, `set-progress` — are the reference set. Read any of them in `packages/ship-set-components/src/` for shape; each is ~30–60 lines of `.html.set` plus a 6-line `.ts`.

### What's NOT ported (yet)

The Lit library has 40+ components. The set above are simple-to-medium primitives. Bigger ones (`ship-button`, `ship-tabs`, `ship-input`, `ship-dialog`) port the same way but cost more — port on demand when a real view needs them, not pre-emptively.

## When to ask vs assume

**Assume** when the answer is in the [framework reference](../../docs/ship-set-framework/SKILL.md) or this cheat sheet.

**Ask** when:
- The pattern doesn't fit the three roles cleanly — that's a discipline conversation, not an authoring choice.
- A primitive (`<ship-*>`) you need doesn't exist in the catalog — do not invent a new component inside a view.
- A hook (`useFilteredList` / `useSelection` / `useForm`) doesn't fit the slice you need — adding a fourth controller hook is a deliberate decision, not an accommodation.

## Implementation status

The framework's surface is firm; some pieces are still landing. See [status.md](../../docs/ship-set-framework/references/docs/status.md) for what's stable vs in active dev. Highlights:

- **Stable**: directives, control flow, bindings, `controller()` with all six sections (incl. mount/unmount), the three controller hooks, the three template hooks, `views/agents/` ported as the reference example.
- **In active dev**: SSR walker, LSP namespace-awareness for `use:` hooks, Rust compiler recognition of `<use-*>`.

The active-dev gaps don't block authoring — the daemon hydrate path covers everything in the stable list end-to-end.
