---
group: Recipes
title: Filter chip row
order: 2
description: Faceted filtering for a list view using chip + chip_all + search, all feeding the same SSE stream.
---

When you need to narrow a list by a known facet (category, status,
owner) plus freeform keyword, compose `chip`, `chip_all`, and `search`
against the view's own SSE stream. Each macro sets a signal and
refreshes `/api/view/<view>/sse`; the server re-runs the query
with the new filter and patches the list region.

The pattern: declare the filter signals at the root, wrap the list
region in `live()`, wire chips to the category signal and the input to
the keyword signal. No custom click handlers, no hand-written fetches.

```jinja
{% from '_ship.html.j2' import signals, live, chip, chip_all, search %}

<div class="ship-view" {{ signals({'status': '', 'q': ''}) }}>
  <ship-toolbar>
    <ship-chip {{ chip_all('status', 'tasks') }}>All</ship-chip>
    <ship-chip {{ chip('status', 'active', 'tasks') }}>Active</ship-chip>
    <ship-chip {{ chip('status', 'blocked', 'tasks') }}>Blocked</ship-chip>
    <ship-chip {{ chip('status', 'done', 'tasks') }}>Done</ship-chip>
    <ship-input {{ search('q', 'tasks') }} placeholder="Search tasks..."></ship-input>
  </ship-toolbar>

  <div {{ live('tasks') }}>
    <div id="tasks-content">Loading...</div>
  </div>
</div>
```

**Primitives used:** `signals`, `live`, `chip`, `chip_all`, `search`.

**Common pitfall:** passing a different view name to `chip()` than the
one on the surrounding `live()`. Both must target the same view slug —
otherwise the chip refreshes a stream no one is listening to.
