---
title: "Template hooks — <use-*>"
description: "Element-shaped lifecycle / side effect, declared in the template."
group: "Tutorial"
order: 19
slug: "19-template-hooks"
prev: "18-untrack-batch"
next: "20-controller-hooks"
chapter: "composition"
---

A **template hook** is a custom element of the form `<use-name>`.
It doesn't render to the DOM — it attaches a host-side side effect
(or contributes a scope identifier) and tears down on unmount.

```sui
<use-title bind:text={pageTitle} />

<use-kbd>
  <on key="/"      action="toggleSearch" />
  <on key="cmd+n"  action="newAgent" />
  <on key="escape" action="clearSelection" />
</use-kbd>

<dialog bind:this={dialogEl}>…</dialog>
<use-click-outside bind:target={dialogEl} action="close" />
```

Three patterns:

| Pattern | Example | What it does |
|---|---|---|
| **Receive controller ref** | `<use-click-outside bind:target={ref} />` | Hook gets a handle to a controller-side element / value. |
| **Template-fragment children** | `<use-kbd><on key="…" /></use-kbd>` | The hook walks its children for declarative configuration. |

The compiler treats `<use-*>` as a registered hook lookup — unknown
hook names are a compile error, not a silent no-op. Today's hooks:

- `<use-title>` — bind document.title
- `<use-kbd>` — document-scoped keyboard shortcuts
- `<use-click-outside>` — close-on-outside-click
- `<use-focus-trap>` — trap tab focus inside a region
- `<use-clipboard>` — copy / paste plumbing
- `<use-scroll-anchor>` — preserve scroll position across re-renders
- `<use-intersection>` — IntersectionObserver wrapper

## Try it

The example uses `<use-title>` to bind the browser tab's title and
`<use-kbd>` for keyboard shortcuts. Try:

1. **Open a new tab**: the title should reflect `count()`. Increment
   the counter — the tab title updates.
2. **Press the shortcuts**: `j`/`k` to move the count, `r` to reset.
   The bindings live in the template, not in a `useEffect` /
   `addEventListener` you wrote by hand.
3. **Add `<use-clipboard />`** somewhere and try the copy hook —
   another platform-bridge hook that contributes a controller-callable
   helper without manual setup.
