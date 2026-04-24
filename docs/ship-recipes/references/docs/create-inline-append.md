---
group: Recipes
title: Create + inline-append
order: 9
description: POST a new collection item with create(), then let the live() region re-render from SSE.
---

When you need to add an item without reloading the page, use `create()`
against your collection and let the view's `live()` subscription
deliver the new row. The collection write emits an event, the view's
SSE stream re-queries, and the list region patches in place. You
author no client-side append logic.

```jinja
{% from '_ship.html.j2' import signals, bind, live, create, loading, spinner %}

<div class="ship-view" {{ signals({'form': {}, '_creating': false}) }}>
  <ship-card>
    <ship-input {{ bind('form.title') }} placeholder="New task title"></ship-input>
    <ship-button variant="primary"
                 {{ create('task', '$form') }}
                 {{ loading('_creating') }}>
      Add
    </ship-button>
    {{ spinner('_creating') }}
  </ship-card>

  <div {{ live('tasks') }}>
    <div id="tasks-content">
      {% for item in ship_items('task') %}
        <ship-list-tile><span slot="title">{{ item.data.title }}</span></ship-list-tile>
      {% endfor %}
    </div>
  </div>
</div>
```

**Primitives used:** `signals`, `bind`, `live`, `create`, `loading`,
`spinner`.

**Common pitfall:** trying to mutate the list signal yourself after
`create()`. The SSE stream is the source of truth — let it patch.
Local-append plus server-patch produces duplicates.
