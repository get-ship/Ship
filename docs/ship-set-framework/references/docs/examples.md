---
group: Set Framework
order: 7
title: Canonical examples
description: Three example shapes to anchor the framework's discipline. Counter (no controller), agents (function controller + two hooks), form (controller hook + two-way binding).
---

# Canonical examples

These three examples cover the discipline boundaries. Each tier has a representative; each example exercises a different combination of primitives.

## Tier 1 — counter (no controller)

Pure declarative. Local state in the template. No `.ts` file.

```html
<!-- views/counter/view.html.set -->
<state name="count" value="0" />

<div class="counter">
  <p>Count: <span data-test="value">{count()}</span></p>
  <button on:click={() => count.set(count() + 1)}>+</button>
  <button on:click={() => count.set(0)}>reset</button>
</div>
```

Files: just `view.html.set`. The framework infers tier 1 from the absence of `view.ts`.

What this exercises: `<state>`, expression interpolation, `on:click` with a call-only arrow body.

What this avoids: a controller for two pieces of state and two handlers. Trivial views stay trivial.

## Tier 2 — agents (object controller + a derived bundle)

The actual working view in the repo today (`views/agents/`). Server data via `<data>`, two memos, one navigation handler. No hooks yet — the hook-based version is below.

```html
<!-- views/agents/view.html.set (excerpt) -->
<data name="agents" source="ship.agents.summaries()" live />

<header class="agents-head">
  <h1>Agents</h1>
  <p>{total()} agent{total() === 1 ? '' : 's'}</p>
  <ship-button tone="primary" on:click={newAgent}>New agent</ship-button>
</header>

<while-loading bind="agents">
  <p>Loading agents…</p>
</while-loading>

<while-error bind="agents" let:err>
  <p>Error loading agents: {err.message}</p>
</while-error>

{#if agents().kind === 'ok'}
  {#if items().length === 0}
    <ship-empty-state>No agents yet.</ship-empty-state>
  {:else}
    <div class="agents-grid">
      {#for a of items() key={a.id}}
        <ship-list-tile navigable={'/agents/' + a.id}>{a.name}</ship-list-tile>
      {/for}
    </div>
  {/if}
{/if}
```

```ts
// views/agents/view.ts
import { controller } from '@ship-set/runtime'
import type { LiveSignal } from '@ship-set/runtime'
import type { AgentsSummariesResponse, AgentSummary } from '@ship-set/sdk-client'

export default controller({
  derived: {
    items: ({ agents }: { agents: LiveSignal<AgentsSummariesResponse> }): AgentSummary[] => {
      const s = agents()
      return s.kind === 'ok' ? s.value.items : []
    },
    total: ({ items }: { items: () => AgentSummary[] }) => items().length,
  },
  on: {
    newAgent: ({ navigate }: { navigate: (path: string) => void }) =>
      navigate('/agents/create'),
  },
})
```

What this exercises:

- `<data live>` subscription — daemon-streamed data, declared in the template
- Object-form controller (no closure-scoped vars; `controller(def)` enough)
- `derived` reading template-declared scope (`agents`, then `items`)
- `on` handler reading framework-injected `navigate` via the proxy fallthrough
- `<while-loading>` / `<while-error>` boundaries

What this avoids: a `live:` section in the controller, a `view()` wrapper, calling `live()` from JS.

## Tier 3 — agents with hooks (target shape, lands when hooks ship)

The same view, scaled with the v1 hooks (`useFilteredList`, `useSelection`, `<use-shell>`, `<use-kbd>`). Use this once the hooks are shipped — see [status.md](status.md) for what's currently available.

```html
<!-- views/agents/view.html.set -->
<data name="agents" source="ship.agents.summaries()" live />

<use-shell as="shell" />
<use-kbd>
  <on key="/"        action="toggleSearch" />
  <on key="cmd+n"    action="newAgent" />
  <on key="escape"   action="clearSelection" />
</use-kbd>

<header class="agents-head">
  <h1>Agents — {filter.filtered().length}</h1>
  {#if shell.currentUser}
    <span class="agents-user">{shell.currentUser.name}</span>
  {/if}
  <ship-button tone="primary" on:click={newAgent}>New agent</ship-button>
</header>

{#if searchOpen()}
  <input
    bind:value={filter.query}
    placeholder="Search agents…"
  />
{/if}

{#if agents().kind === 'ok'}
  <ul class="agents-grid">
    {#for a of filter.filtered() key={a.id}}
      <li>
        <ship-list-tile
          tone={sel.selected()?.id === a.id ? 'primary' : 'default'}
          on:click={() => sel.select(a.id)}
        >{a.name}</ship-list-tile>
      </li>
    {/for}
  </ul>
{/if}
```

Add a `<state>` line to the template for the search-toggle (state lives in the template per the [declarative-state ADR](../../../ship-adrs/references/docs/ship-set-declarative-state.md)):

```html
<state name="searchOpen" value="false" />
```

```ts
// views/agents/view.ts
import { controller, useFilteredList, useSelection } from '@ship-set/runtime'

export default controller({
  use: ({ agents }) => ({
    filter: useFilteredList(agents, (a, q) => a.name.toLowerCase().includes(q.toLowerCase())),
    sel:    useSelection(agents, (a) => a.id),
  }),
  on: {
    toggleSearch:    (s)              => { s.searchOpen = !s.searchOpen },
    newAgent:        ({ navigate })   => navigate('/agents/create'),
    clearSelection:  ({ sel })        => sel.clear(),
  },
})
```

What this exercises beyond tier 2:

- Template hooks (`<use-shell>`, `<use-kbd>`) for host context and document keybindings
- Controller hooks (`useFilteredList`, `useSelection`) for UI pattern bundles
- Per-section scope destructure (`use: ({ agents }) => ...`)
- Hook namespacing in templates (`filter.filtered`, `sel.selected`)
- Action references inside `<use-kbd>` resolving to `on.*`

## Tier 3 — settings form (object controller + useForm hook)

Forms exercise two-way binding, validation, and the `useForm` controller hook. Lands once `useForm` ships — see [status.md](status.md) for the current hook surface.

```html
<!-- views/settings/view.html.set -->
<data name="settings" source="ship.settings.read()" />

<form on:submit={form.submit}>
  <label>
    Theme
    <select bind:value={form.values.theme}>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
      <option value="system">System</option>
    </select>
  </label>

  <label>
    Accent color
    <input bind:value={form.values.accent} />
  </label>
  {#if form.errors().accent}
    <p class="error">{form.errors().accent}</p>
  {/if}

  <button type="submit" disabled={form.pending() || !form.dirty()}>
    {form.pending() ? 'Saving…' : 'Save'}
  </button>
  <button type="button" on:click={form.reset} disabled={!form.dirty()}>
    Reset
  </button>
</form>
```

```ts
// views/settings/view.ts
import { controller, useForm } from '@ship-set/runtime'

export default controller({
  use: ({ ship, settings }) => ({
    form: useForm({
      initial:  () => settings(),
      validate: (v) => ({
        accent: /^#[0-9a-f]{6}$/i.test(v.accent) ? null : 'must be a hex color',
      }),
      submit:   (v) => ship.settings.write(v),
    }),
  }),
})
```

What this exercises:

- Object-form `controller(def)` (no closure-scoped vars needed)
- `useForm` controller hook bundling `state` (values, errors, pending, dirty), `derived` (errors per field), `on` (change, submit, reset)
- `bind:value` two-way binding on a `<select>` and an `<input>`
- `on:submit` on the form element

What this avoids: hand-rolled form state with five state variables and four handlers. The pattern lives in the hook.

## What these examples establish

| Decision | Where it lives |
|---|---|
| Server data | `<data>` in the template |
| Local state (any) | `<state>` in the template |
| Computed-initial state (`crypto.randomUUID()`, etc.) | `<state>` placeholder + assignment in `mount` |
| Computed values | Controller `derived:` (named) or inline expressions (one-off) |
| Event handlers | Controller `on:` |
| UI pattern bundles | Controller hook in `use:` |
| Host primitives | Template hook (`<use-*>`) |
| Lifecycle | Controller `mount` / `unmount` |

Three examples, three tiers, all five controller sections, both hook categories, every binding form. If a new view doesn't fit one of these shapes, it is *almost certainly* combinable from these primitives. If it isn't, escalate to a discipline conversation — see [the discipline](index.md#the-discipline).
