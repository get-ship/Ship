---
group: Recipes
title: Infinite-scroll list
order: 15
description: A sentinel element at the bottom uses data-on:intersect to fetch the next page. One of the rare sanctioned raw-Datastar cases.
---

When you need page-after-page loading as the user scrolls, put a
sentinel element at the bottom of the list with `data-on:intersect`
firing a `@get` for the next page. The daemon returns the appended
rows and replaces the sentinel with a new one pointing at the new
last id.

This is **the** sanctioned raw-Datastar case in a Ship view: no macro
wraps `intersect` today because the ergonomics haven't settled. Every
other Datastar binding in the framework has a macro — flag it if you
reach for raw `data-*` for anything else.

```jinja
{% from '_ship.html.j2' import signal, live %}

<div class="ship-view" {{ signal('lastId', '') }}>
  <div {{ live('tasks') }}>
    <div id="tasks-content">
      {% for item in ship_items('task') %}
        <ship-list-tile><span slot="title">{{ item.data.title }}</span></ship-list-tile>
      {% endfor %}
    </div>

    {% if has_more %}
      <div id="scroll-sentinel"
           data-on:intersect="@get('/api/runtime/view/tasks?after=' + $lastId)">
        Loading more...
      </div>
    {% endif %}
  </div>
</div>
```

**Primitives used:** `signal`, `live`. Plus the sanctioned raw
`data-on:intersect`.

**Common pitfall:** forgetting to replace or remove the sentinel on
each page. If the daemon returns new rows but leaves the old sentinel
in place, it will immediately re-fire as the user scrolls — thrash.
The SSE patch must swap the sentinel for a new one (or drop it when
`has_more` is false).
