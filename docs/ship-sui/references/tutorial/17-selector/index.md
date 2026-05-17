---
title: "selector — single-source list highlighting"
description: "When one signal drives the highlighted row, two rows re-render. Not N."
group: "Tutorial"
order: 17
slug: "17-selector"
prev: "16-effect-cleanup"
next: "18-untrack-batch"
chapter: "reactivity"
---

A common pattern: a list of rows, one of which is "selected". The
naive shape subscribes every row to the `selectedId` signal, so when
the selection changes **every row re-evaluates**:

```sui
{#for r of rows() key={r.id}}
  <li class:selected={r.id === selectedId()}>{r.label}</li>  <!-- every row reads selectedId -->
{/for}
```

`selector()` flips that — it returns a function `(key) => boolean`
that is O(1) per call and only notifies the **two affected** rows
(the one losing selection, the one gaining it) on change.

```ts
import { signal, selector } from '@ship/sui-runtime'

const selectedId = signal(0)
const isSelected = selector(selectedId)   // keyed reactivity
```

```sui
{#for r of rows() key={r.id}}
  <li class:selected={isSelected(r.id)}>{r.label}</li>
{/for}
```

Under the hood: `selector()` runs a single ambient effect against
the source signal. Each `isSelected(key)` call subscribes the
current effect to a *bucket keyed by the value*. When the source
changes from A → B, only buckets A and B notify — every other
bucket sits cold.

For a 1000-row list with a moving selection: naive = 1000
re-evaluations per selection change; `selector()` = 2. The other
998 rows never wake up.

## Try it

The example renders 200 rows and tracks a selection via arrow keys.
Try:

1. **Watch the DOM**: open devtools' Rendering → "Paint flashing".
   Move the selection with `j/k`. Only two rows flash per move.
2. **Compare with naive**: swap `isSelected(r.id)` for
   `r.id === selectedId()` in the template. Now every row flashes
   per move (a much larger paint area).
3. **Selector over derived**: pass a derived signal (e.g. one that
   normalises `selectedId` modulo `rows().length`) into `selector()`.
   The bucket reactivity still applies — only the affected rows
   re-fire.
