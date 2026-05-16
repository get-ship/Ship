---
group: Recipes
title: Optimistic update
order: 8
description: Persist an edit the moment the user stops typing with bind + put_on_change.
---

When you need a settings-style input that saves without an explicit
Save button, pair `bind()` with `put_on_change()`. `bind` keeps the
signal mirrored to the input; `put_on_change` fires a debounced PUT on
every committed change event. The UI reflects the new value
immediately; the server catches up on the next debounce window.

```jinja
{% from '_ship.html.j2' import signal, bind, put_on_change %}

<div class="ship-view" {{ signal('theme', theme) }}>
  <sui-field label="Theme">
    <sui-select {{ bind('theme') }}
                 {{ put_on_change('/api/settings', '{theme: $theme}') }}>
      <option value="system">System</option>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
    </sui-select>
  </sui-field>
</div>
```

**Primitives used:** `signal`, `bind`, `put_on_change`.

**When NOT to use:** for high-stakes fields (billing, permissions),
require an explicit Save. Optimistic update is for preferences, not
commitments.
