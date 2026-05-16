---
group: Recipes
title: Debounced search
order: 4
description: The search macro gives you two-way binding plus 300ms debounce plus SSE refresh in a single call.
---

When you need a search input that waits for the user to stop typing
before refiring the query, use the `search` macro. It binds the input
to a signal, adds a debounce modifier, and refreshes the view's SSE
stream in one go. The server consumes the signal when rebuilding the
list.

```jinja
{% from '_ship.html.j2' import signal, live, search %}

<div class="ship-view" {{ signal('q', '') }}>
  <sui-input {{ search('q', 'tasks', '500ms') }}
              placeholder="Search tasks..."
              aria-label="Search"></sui-input>

  <div {{ live('tasks') }}>
    <div id="tasks-content">
      {% for item in ship_items('task', q=q) %}
        <sui-list-tile>
          <span slot="title">{{ item.data.title }}</span>
        </sui-list-tile>
      {% endfor %}
    </div>
  </div>
</div>
```

**Primitives used:** `signal`, `live`, `search`.

**Common pitfall:** forgetting to declare the query signal on the root
element. The `search` macro only binds — it does not declare. If `$q`
isn't declared upstream, the first keystroke writes to a
runtime-created signal that doesn't round-trip cleanly.
