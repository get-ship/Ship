---
group: Recipes
title: Toast after action
order: 11
description: Fire a styled toast from any click using toast_container + toast_action.
---

When you need transient success/error feedback, mount one
`toast_container()` on the page and call `toast_action()` on any
button. Multiple actions can chain in a single click expression by
separating them with semicolons — pair it with the action macro you
actually want, and the toast fires alongside.

```jinja
{% from '_ship.html.j2' import signal, bind, toast_container, toast_action %}

<div class="ship-view" {{ signal('value', '') }}>
  <ship-input {{ bind('value') }} placeholder="Anything"></ship-input>
  <ship-button {{ toast_action('Saved!', 'success', 2500) }}>
    Pretend save
  </ship-button>
</div>

{{ toast_container() }}
```

**Primitives used:** `signal`, `bind`, `toast_container`,
`toast_action`.

**Common pitfall:** mounting `toast_container()` more than once.
`toast_action` dispatches by the fixed id `ship-toasts`; duplicate
containers cause duplicate toasts.
