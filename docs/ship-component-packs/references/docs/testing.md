---
group: Component Packs
title: Testing
order: 5
description: The minimum every pack component ships — render, attrs, events, keyboard.
audience: internal
---

Every pack component ships with a colocated `.test.ts` file. The bar
is the same as first-party `@ship/set` components; packs do not get a
discount.

## The four tests

Every interactive component has these four at minimum:

| Test | What it verifies |
|---|---|
| Render | Mounts, renders initial content, reflects attrs to the DOM. |
| Attribute reactivity | Changing an attr (`tone`, `active`, `disabled`) re-renders correctly and reflects to the host where declared. |
| Event | User action (click, keydown) dispatches the documented `CustomEvent` with the documented `detail` shape. `bubbles: true, composed: true`. |
| Keyboard | Enter/Space activates; Escape dismisses; Arrow keys navigate — whichever apply. |

Non-interactive components drop the event and keyboard tests and
keep render + attr-reactivity.

## Runner and patterns

The ship-set workspace uses Vitest with jsdom. New pack components
inherit that config — don't introduce a second test runner.

Reference tests in-tree to copy from:

- `packages/ship-set/src/web-components/core/ship-button.test.ts` — an
  interactive primitive with tone / variant / size / disabled / loading
  states and a keyboard test.
- `packages/ship-set/src/web-components/core/ship-list-tile.test.ts` —
  a slot-heavy primitive whose tests cover each named slot and the
  `ship-select` event from click and keyboard.
- `packages/ship-set/src/web-components/core/ship-segmented.test.ts` —
  arrow-key navigation over a group of children.

Prefer patterns from these files verbatim — same helpers, same style —
rather than inventing new setup.

## Running

```bash
pnpm -F @ship/set test                       # full suite
pnpm -F @ship/set test ship-my-thing         # single component
pnpm -F @ship/set test:watch                 # re-run on change
```

A component without tests is incomplete. Don't land one.
