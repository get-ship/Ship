---
group: Recipes
title: Modal create
order: 13
description: Reuse confirm_dialog as a full create form host — the confirm button carries the create() action.
---

When you want a create flow that overlays the current view instead of
navigating away, repurpose `confirm_dialog()`. The fourth argument
(`action_attrs`) accepts any action macro's output, so pass
`create('task', '$form')` there and put the form inputs inside the
dialog body. The Cancel button closes; the Confirm button creates and
closes.

```jinja
{% from '_ship.html.j2' import signals, bind, create, confirm_dialog, confirm_trigger %}

<div class="ship-view" {{ signals({'form': {}}) }}>
  <sui-button variant="primary" {{ confirm_trigger('new-task') }}>
    New task
  </sui-button>

  {{ confirm_dialog(
       'new-task',
       'New task',
       'Give it a title and priority.',
       create('task', '$form')) }}

  <sui-alert-dialog-slot slot="body" for="confirm-new-task">
    <sui-field label="Title">
      <sui-input {{ bind('form.title') }}></sui-input>
    </sui-field>
    <sui-field label="Priority">
      <sui-select {{ bind('form.priority') }}>
        <option value="low">Low</option>
        <option value="normal">Normal</option>
        <option value="high">High</option>
      </sui-select>
    </sui-field>
  </sui-alert-dialog-slot>
</div>
```

**Primitives used:** `signals`, `bind`, `create`, `confirm_dialog`,
`confirm_trigger`.

**Common pitfall:** putting the form inputs outside the dialog. They
still bind to `$form` wherever they live, but users expect the inputs
to be inside the overlay — keep them slotted into the dialog body.
