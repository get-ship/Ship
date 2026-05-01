---
name: sui-authoring
stable-id: sui-authoring
description: Author Ship views and components with SUI — `.sui` single-file templates, optional paired `.ts` controller, signals, server-driven `<data live>` subscriptions. AOT-compiled, attach()-on-SSR. This is how you write new Ship UI.
tags: [ship, sui, ui, views, components, authoring]
authors: [ship]
audience: agent
---

# SUI authoring — the cheat sheet

Use this skill while writing or porting `.sui` views and components. Full reference: `docs/ship-ui/references/docs/`.

## The three-role split

| Role | File | What goes here |
|---|---|---|
| **Template** | `view.sui` | Structure + declarative wiring (`<data>`, `<state>`, `<use-*>`, `on:`, `bind:`, control flow) |
| **Controller** | `view.ts` (optional) | Derived values, handlers, lifecycle — `derived` / `on` / `use` / `mount` / `unmount` |
| **Data subscription** | `<data>` in template | Bridge to `shipd` (`source="ship.X.Y()" [live]`) |

File IS the component — no wrapper element. State lives in `<state>` template directives only; controllers have no `state:` section.

## Template surface

```html
<!-- subscriptions -->
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- reactive args — re-subscribes when selectedId changes -->
<state name="selectedId" value="'abc'" />
<data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />

<!-- local state -->
<state name="open" value="false" />

<!-- props (component .sui files only; tag name comes from <tag>.sui.json) -->
<prop name="tone" values="default|primary" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />

<!-- mutations (write-side complement to <data>) -->
<form mutation="ship.agents.create">
  <input name="name" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>Create</button>
</form>

<!-- control flow (block syntax) -->
{#if loading()}…{:else if error()}…{:else}…{/if}
{#for a of items() key={a.id}}…{/for}
{#match status()}{:case 'idle'}…{:default}…{/match}

<!-- live-data loading state (element form) -->
<while-loading bind="agents"><p>Loading…</p></while-loading>
<while-error bind="agents" let:err><p>{err.message}</p></while-error>

<!-- bindings -->
<button on:click={() => count.set(count() + 1)}>+</button>
<input bind:value={query} />
<input type="checkbox" bind:checked={enabled} />
<div class:active={isActive()} style:color={color()} attr:href={url()}>
  {count() === 1 ? '1 item' : count() + ' items'}
</div>

<!-- template hooks -->
<use-shell as="shell" />
<use-kbd><on key="/" action="search" /></use-kbd>
<use-click-outside bind:target={dialogEl} bind:action={close} />

<!-- slots (components only) -->
<slot name="leading"></slot><slot></slot>
```

## Controller (five sections, no `state:`)

```ts
import { controller } from '@ship/ui-runtime'
import type { Scope } from './view.sui'   // compiler-emitted .sui.d.ts

export default controller<Scope>({
  derived: { total: ({ agents }) => unwrap(agents)?.items.length ?? 0 },
  on:      { newAgent: ({ navigate }) => navigate('/agents/create') },
  use:     { list: useFilteredList(agents, (a, q) => a.name.includes(q)) },
  mount:   ({ state }, el) => { /* runs after first DOM attach */ },
  unmount: ({ state })     => { /* runs on dispose */ },
})
```

**Three tiers — pick the smallest that fits:**

1. **Tier 1 — no controller.** Pure `.sui` with `<state>` and single-expression inline handlers.
2. **Tier 2 — object-literal controller.** Most coordinated views. Multi-statement handlers require a paired `.ts`.
3. **Tier 3 — function-form controller.** When setup needs closure-scoped variables.

**Threshold rule.** A multi-statement `on:event` handler body in a tier-1 file (no `.ts`) is a compile error: `multi-statement handler — paired controller required for tier-2 work`. Enforced by `crates/sui-compiler/src/check/handler_complexity.rs`.

## Controller hooks (compose via `use:`)

```ts
import { useFilteredList, useSelection, useForm } from '@ship/ui-runtime'
```

Templates reference them namespaced: `list.filtered()`, `sel.selected()`, `form.values()`, `form.submit`.

## Template hooks

| Hook | Purpose |
|---|---|
| `<use-shell as="X">` | Host context (theme, currentUser, app metadata) |
| `<use-kbd>` | Document keyboard shortcuts — children `<on key="..." action="onName" />` |
| `<use-click-outside bind:target bind:action>` | Dismiss-on-outside |
| `<use-focus-trap>` | Keyboard focus containment for dialogs |
| `<use-clipboard>` | Click-to-copy |
| `<use-title>` | Reactive `document.title` |
| `<use-scroll-anchor>` | Smooth-scroll to hash targets |
| `<use-intersection>` | IntersectionObserver callback |

## AOT + attach() pipeline

**Compiler** (`crates/sui-compiler/`) emits one ESM per `.sui` plus a `bindings` manifest exported alongside `default`. No client-side parser on the production path.

**SSR walker** (`shipd`) renders the template to HTML with initial values inlined. Markers:

| Template construct | HTML marker |
|---|---|
| `{expr}` text interpolation | `<!--sui:text:bN-->value<!--/sui-->` |
| `on:click={fn}` | `on:click="bN"` attribute preserved |
| `bind:value={sig}` | `bind:value="bN"` attribute preserved |
| `class:active={expr}` | `class:active="bN"` attribute preserved |
| `style:color={expr}` | `style:color="bN"` attribute preserved |
| `attr:href={expr}` | `attr:href="bN"` attribute preserved |
| `{#if}` / `{#for}` / `{#match}` active branch | `<sui-if key="bN">…</sui-if>` (or `<sui-for>`, `<sui-match>`) |
| Inactive branch | `<sui-if key="bN" hidden></sui-if>` |
| Initial scope values | `<script type="application/json" id="__set_init__">…</script>` |

Zero `data-*` attributes in framework-emitted HTML.

**`attach()`** from `@ship/ui-dom` walks the SSR DOM, reads the `__set_init__` JSON island, wires bindings against the manifest, runs lifecycle.mount, returns a disposer:

```ts
import { attach } from '@ship/ui-dom'
const dispose = attach(root, { module, ship, route, navigate, viewName })
```

Never import `@ship/ui-dom/legacy` in production — it contains the parser (~5 KB gzip).

## Hard rules

1. **Mutations in `on.*` handlers only.** `s.x = v` in a derived or template expression is a type error.
2. **Subscriptions in `<data>` elements.** Never call `ship.*` outside `<data>`.
3. **Templates declare; controllers do.** Declarative → template; imperative → controller.
4. **No `<script>` blocks in templates.** Logic in paired `.ts`.
5. **No `customElements.define` in view code.**
6. **Read signals by call:** `count()` reads, `count.set(n)` writes.

## Live data envelope

```ts
type LiveState<T> = { kind: 'loading' } | { kind: 'error'; error: Error } | { kind: 'ok'; value: T }

// Helpers from @ship/ui-runtime:
unwrap(sig)               // T | undefined  — undefined when loading/error
unwrapOr(sig, fallback)   // T              — returns fallback when loading/error
```

```html
<while-loading bind="agents"><p>Loading…</p></while-loading>
<while-error bind="agents" let:err><p>{err.message}</p></while-error>
{#if agents().kind === 'ok'}
  {#for a of agents().value.items key={a.id}}…{/for}
{/if}
```

## File layout

**First-party views** (baked into `shipd`):
```
views/<name>/
  view.sui    # template (required)
  view.ts     # controller (optional)
  view.css    # styles (optional; daemon auto-links)
```

**SUI components** (`packages/ship-set-components/src/<tag>/`):
```
<tag>.html.set    # template + <prop> directives + <style>
<tag>.sui.json    # { "component": "<tag>", "shadow": true }
<tag>.ts          # import './<tag>.compiled.js'  (side-effect only)
```

## What's phased out — don't author against these

- **`view()` factory** from `@ship/ui-runtime`. Removed. Use `controller()`.
- **`<component>` wrapper element.** Gone. File IS the component.
- **Client-side parser / `hydrate()`** in production. AOT + `attach()` only.
- **`.html.j2` + Datastar stack.** Fix-only for `.ship/views/`. New views go in `views/<name>/` as `.sui`.

## Reference docs

- `docs/ship-ui/references/docs/template.md` — full template syntax + renderable subset
- `docs/ship-ui/references/docs/controller.md` — controller surface + tier rules
- `docs/ship-ui/references/docs/data-subscriptions.md` — `<data>` + live + mutations
- `docs/ship-ui/references/docs/ssr-hydration.md` — AOT + attach pipeline, marker contract
- `docs/ship-ui/references/docs/status.md` — stable vs in-dev
