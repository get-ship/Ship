---
title: "Event modifiers"
description: "preventDefault, stopPropagation, once, key filters."
group: "Tutorial"
order: 7
slug: "07-event-modifiers"
prev: "06-bind-value"
next: "08-sui-primitives"
chapter: "templating"
---

`on:event` attributes accept pipe-separated modifiers that tweak how
the handler is registered or what it receives — saving you a line of
`e.preventDefault()` and friends inside every handler body.

```sui
<form on:submit|preventDefault={handle}>…</form>
<button on:click|once={initOnce}>Init</button>
<div on:click|stopPropagation={onCardClick}>…</div>
<input on:keydown|enter={submit} />
```

The modifiers SUI ships out of the box:

| Modifier | What it does |
|---|---|
| `preventDefault` | Calls `e.preventDefault()` before your handler runs |
| `stopPropagation` | Calls `e.stopPropagation()` before your handler runs |
| `once` | Removes the listener after the first dispatch |
| `capture` | Registers the listener in capture phase |
| `passive` | Registers the listener as passive (touch/scroll perf) |
| `self` | Only fires when `e.target === currentTarget` |
| `enter` / `escape` / `tab` / `space` | Key filters on `keydown` / `keyup` / `keypress` |

Modifiers compose: `on:click|preventDefault|once|stopPropagation`
gives you all three. They're checked at compile time — typos like
`on:click|prevenDefault` are a compile error, not a silent no-op.

## Try it

The chat input fires on Enter via `on:keydown|enter`. Try:

1. **Click-outside to dismiss.** The "edit" button reveals an inline
   text box. Wrap the outer card in `on:click|self={() => dismiss()}`
   so clicking the card (but not its children) closes the editor.
2. **One-shot init.** Add an `on:click|once={firstAction}` button that
   only fires once. Click it twice — the second click should do
   nothing (and the button should visibly disable).
3. **Escape closes.** Add `on:keydown|escape={dismiss}` on the edit
   input so pressing escape exits without saving.
