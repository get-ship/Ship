---
group: Recipes
title: Command palette entry
order: 16
description: Register a view-specific action for the shell's command palette by listening for keydown.meta.k and dispatching a CustomEvent.
---

When your view wants to contribute entries to the shell's command
palette, bind `keydown.meta.k` with the `on()` macro and dispatch a
`CustomEvent` that the shell's palette listens for. The expression
form of `on()` lets you run any JS snippet — this is the framework's
official escape hatch for "I need to talk to the shell."

```jinja
{% from '_ship.html.j2' import on %}

<div class="ship-view"
     {{ on('keydown.meta.k',
           "evt.preventDefault();
            window.dispatchEvent(new CustomEvent('ship:command-palette:open',
              { detail: { from: 'tasks', actions: [
                { id: 'new-task', label: 'New task', hint: 'N' },
                { id: 'archive-done', label: 'Archive done' }
              ]}}))") }}>
  <!-- rest of the view -->
</div>
```

**Primitives used:** `on`.

**When NOT to use:** if you only need a keyboard shortcut that stays
local to the view (toggle a panel, focus an input), handle it with
`on('keydown.ctrl.f', "$showSearch = !$showSearch")` and skip the
CustomEvent entirely. The palette dance is for entries the shell owns.
