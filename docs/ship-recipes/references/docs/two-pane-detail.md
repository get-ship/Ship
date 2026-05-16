---
group: Recipes
title: Two-pane detail view
order: 14
description: Sidebar list + detail pane driven by a single selection signal and layout_sidebar.
---

When you need a master/detail layout, compose `layout_sidebar()` with
a selection signal. Each list tile sets the signal via `on('click')`;
the detail pane reads the signal with `data-text` (or `show()` to
toggle richer content). The whole thing is one view, no routing.

```jinja
{% from '_ship.html.j2' import signal, on, attr, layout_sidebar, layout_end, show %}

<div class="ship-view" {{ signal('selected', '') }}>
  {{ layout_sidebar('280px') }}
    <aside>
      {% for item in ship_items('task') %}
        <sui-list-tile
          {{ on('click', "$selected = '" ~ item.id ~ "'") }}
          {{ attr('data-active', "$selected == '" ~ item.id ~ "'") }}>
          <span slot="title">{{ item.data.title }}</span>
        </sui-list-tile>
      {% endfor %}
    </aside>

    <main>
      <sui-empty-state {{ show('!$selected') }}>
        Select a task on the left.
      </sui-empty-state>
      <sui-card {{ show('$selected') }}>
        <h2>Task <span data-text="$selected"></span></h2>
      </sui-card>
    </main>
  {{ layout_end() }}
</div>
```

**Primitives used:** `signal`, `on`, `attr`, `layout_sidebar`,
`layout_end`, `show`.

**When NOT to use:** if each detail pane is server-side expensive to
render or needs its own URL, fetch the detail via `navigate()` into a
morph target instead.
