---
group: Recipes
title: Delete with confirm
order: 7
description: Gate destructive collection actions behind a typed sui-alert-dialog via confirm_dialog + confirm_trigger.
---

When you need a destructive action gated by a typed, Ship-styled
dialog, pair `confirm_dialog()` with `confirm_trigger()`. The dialog
renders once per item, pre-wired with the `delete()` macro; the
trigger button just names the dialog id.

For a quick native browser prompt instead, use `confirm_delete()` —
same effect, no typed UI.

```jinja
{% from '_ship.html.j2' import delete, confirm_dialog, confirm_trigger %}

{% for item in ship_items('task') %}
  <sui-list-tile>
    <span slot="title">{{ item.data.title }}</span>
    <sui-button slot="actions" variant="ghost"
                 {{ confirm_trigger('delete-' ~ item.id) }}>
      Delete
    </sui-button>
  </sui-list-tile>

  {{ confirm_dialog(
       'delete-' ~ item.id,
       'Delete this task?',
       'This cannot be undone.',
       delete('task', item.id)) }}
{% endfor %}
```

**Primitives used:** `delete`, `confirm_dialog`, `confirm_trigger`.

**Common pitfall:** reusing the same dialog `id` across rows. Each
item needs a unique id suffix (`'delete-' ~ item.id`) or the trigger
will always open the first one.
