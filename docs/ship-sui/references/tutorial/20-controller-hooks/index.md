---
title: "Controller hooks — use: section"
description: "Reusable bundles of state + derived + handlers."
group: "Tutorial"
order: 20
slug: "20-controller-hooks"
prev: "19-template-hooks"
next: "21-forms"
chapter: "composition"
---

A **controller hook** is a reusable slice — state + derived + action
handlers — composed into a controller's `use:` section, namespaced
by a key.

```ts
import { controller, signal, useFilteredList, useSelection } from '@ship/sui-runtime'

export default controller<Scope>({
  scope: () => ({
    items: signal([{ id: 1, label: 'one' }, { id: 2, label: 'two' }, …]),
  }),
  use: {
    list: useFilteredList(
      ({ items }) => items(),
      (item, q) => item.label.toLowerCase().includes(q.toLowerCase()),
    ),
    sel:  useSelection(({ items }) => items(), (i) => i.id),
  },
  on: {
    refresh: () => ship.items.refresh(),
  },
})
```

Reading them in the template:

```sui
<input bind:value={list.query} placeholder="filter…" />

{#for item of list.filtered() key={item.id}}
  <li
    class:selected={sel.isSelected(item.id)}
    on:click={() => sel.toggle(item.id)}
  >
    {item.label}
  </li>
{/for}

<p>{sel.count()} selected of {list.filtered().length}</p>
```

The hook's identifiers (`list.query`, `list.filtered`, `sel.toggle`)
are namespaced under their `use:` key. Three hooks ship today:

| Hook | What it bundles |
|---|---|
| `useFilteredList(source, predicate)` | `state.query`, `derived.filtered`, `on.setQuery`, `on.clear` |
| `useSelection(source, getKey)` | `state.selected: Set`, `derived.count`, `on.toggle`, `on.clear`, `derived.isSelected` |
| `useForm(initial, validate)` | `state.values`, `state.errors`, `state.touched`, `on.set`, `on.touch`, `on.submit`, `derived.valid` |

You can author your own — anything matching `ControllerHook<Scope>`
shape: `{ state?, derived?, on? }`. Each section is merged into the
scope under the namespace key.

## Try it

The example combines `useFilteredList` + `useSelection` over a small
agent catalog. Try:

1. **Filter**: type in the input — the list narrows reactively.
2. **Select**: click rows to add them to the selection; `sel.count()`
   updates.
3. **Author a hook**: write your own `useToggle()` that bundles
   `{ state: { on: false }, on: { toggle: (s) => { s.on = !s.on } } }`.
   Add it to `use:` and read `toggle.on()` in the template.
