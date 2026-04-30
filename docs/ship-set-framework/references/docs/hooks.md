---
group: Set Framework
order: 4
title: Hooks — template and controller
description: Two categories of hook with different jobs and different syntax. Template hooks for host-side side effects; controller hooks for UI pattern bundles. When to reach for which.
---

# Hooks

Ship Set has **two** categories of hook with different jobs and different syntax. They are not interchangeable.

| Category | Job | Where it lives | v1 hooks |
|---|---|---|---|
| **Template hook** | Element-scoped lifecycle / side effect, may contribute read-only scope identifiers | Template, as `<use-*>` element | `<use-kbd>` (document keyboard shortcuts), `<use-click-outside>` (dismiss-on-outside for modals / dropdowns / popovers), `<use-shell>` (host context) |
| **Controller hook** | Bundle of state + derived + handlers as a reusable UI pattern slice | Controller, in the `use:` section | `useFilteredList`, `useSelection`, `useForm` |

The v1 lists are deliberately small. New hooks land when a real view needs them — `usePagination` / `useSort` / `<use-focus-trap>` / `<use-clipboard>` and friends are not pre-emptively shipped.

## When to reach for which

Three questions cover almost every "I need something" moment:

1. **Need data from the platform?** → `<data>` element in the template. Not a hook.
2. **Need a side effect tied to the view's lifecycle?** → Template hook (`<use-*>`).
3. **Need a reusable bundle of state, derived values, and handlers?** → Controller hook (`use:` section).

If the answer doesn't fit one of the three, the case is either expressible as a *combination* of these primitives, or it's signaling a gap in the framework that should be filled deliberately — not accommodated with a fourth pattern.

## Template hooks

Element-shaped registrations for host-side side effects and platform context. Live in the template alongside the structure they augment.

### Authoring contract

A template hook is registered as a custom element of the form `<use-<name>>`. The runtime treats it specially:

- The element does not render to the DOM.
- Its **attributes** are bindings: `as="X"` declares an identifier in template scope; `bind:foo={ref}` passes a controller-side reference into the hook. **No expressions in attributes.**
- Its **children** are template fragments. They may contain expressions, control flow, slots, and nested elements. The hook decides what to do with them.

### Three patterns

**Pattern 1 — contribute a scope identifier:**

```html
<use-shell as="shell" />

<!-- shell is now readable in template and any controller section -->
<header class="theme-{shell.theme}">
  Welcome, {shell.currentUser.name}
</header>
```

**Pattern 2 — receive a controller reference:**

```html
<use-focus-trap bind:target={dialogEl} />
```

The `bind:` attribute passes an identifier from controller / template scope to the hook. The compiler statically requires that the value be a bare identifier (no expression).

**Pattern 3 — accept template-fragment configuration:**

```html
<use-title>{agents().value?.items.length ?? 0} agents — {shell.app.name}</use-title>

<use-kbd>
  <on key="/"        action="search" />
  <on key="cmd+n"    action="newAgent" />
  <on key="escape"   action="closeAll" />
</use-kbd>
```

Children are template fragments. Hooks that take richer configuration use child elements (here `<on key="..." action="..." />`) rather than attribute strings. Action references resolve against the controller's `on.*` section — same scope as `on:click={search}`.

### Authoring a template hook

Template hooks are TypeScript modules registered with the framework's hook registry. They expose:

- A name (the `<use-name>` tag)
- An attribute schema (which `as=`, `bind:` and literal attrs are accepted)
- A child schema (which child elements / template fragments are accepted)
- A factory that runs at mount and may register a `cleanup` for unmount

Implementation contract is in active development. See [status.md](status.md).

## Controller hooks

Bundles of `state` / `derived` / `on` / `mount` returned from a factory function. Composed in the controller's `use:` section. Each result is namespaced by its key.

### Authoring contract

A controller hook is a function that returns the controller shape:

```ts
// packages/ship-set-runtime/src/hooks/filtered-list.ts
import type { ControllerHook } from '@ship-set/runtime'

export const useFilteredList = <T>(
  source: () => T[],
  predicate: (item: T, query: string) => boolean,
): ControllerHook<{ query: string }> => ({
  state:   { query: '' },
  derived: { filtered: ({ query }) => source().filter(i => predicate(i, query)) },
  on:      { setQuery: (s, q: string) => { s.query = q } },
})
```

The hook's return shape matches the controller's six-section shape exactly. Use sites:

```ts
// view.ts
import { useFilteredList } from '@ship-set/runtime'

export default ({ ship }) => ({
  use: {
    list: useFilteredList(
      () => /* a signal accessor — see "passing signals" below */,
      (a, q) => a.name.includes(q),
    ),
  },
  // ... rest of the controller
})
```

In the template, the hook's contributions are namespaced by `use:` key:

```html
<input bind:value={list.query} on:input={list.setQuery} />
{#for a of list.filtered key={a.id}}
  <ship-card>{a.name}</ship-card>
{/for}
```

### Passing signals to hooks

**Pass signals directly. Do not wrap them in closures.**

```ts
// good
useFilteredList(agents, (a, q) => a.name.includes(q))

// bad
useFilteredList(() => agents().value?.items ?? [], pred)
```

Signals are first-class reactive references. Inside the hook, `agents()` reads reactively. Closures break that — the hook gets a function that *looks like* a regular getter and authors lose track of what's reactive. Pass the signal; let the hook unwrap.

For live signals with the standard envelope (`{ kind, value: { items } }`), hooks know the shape and unwrap to `.value.items` themselves. See [data-subscriptions.md](data-subscriptions.md#live-envelope).

### Composition rule

Hooks compose by namespacing in `use:`. There is no automatic flat-merge, no spread operator, no precedence rules. If two hooks both contribute a `state.query`, they live as `list.query` and `search.query` — the namespaces keep them apart. This is the trade for never having a name-collision argument again.

## What's NOT a hook

Resist these. They look like hooks but aren't.

- **Subscribing to server data.** That's `<data>` in the template, not a hook. Even if you reuse a subscription across views, repeat the `<data>` element. The author should always see the subscription in the template.
- **Sharing global state.** That's a controller hook instantiated once and imported wherever needed. Same role as any other controller hook; no new primitive. (Yes, this might end up looking like a "store." It's still just a controller hook.)
- **Routing.** Routing is a `shipd` concern, exposed via `<data>` and template route helpers (TBD). Not a hook.
