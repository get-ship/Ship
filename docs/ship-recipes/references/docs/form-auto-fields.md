---
group: Recipes
title: Form with auto-fields
order: 6
description: Let the form() macro generate inputs from a collection schema and submit via create().
---

When you need a create form for a known collection, don't hand-roll
fields. Call `form(ship_schema(<collection>), <collection>)` and it
walks the schema to emit typed inputs bound under `$form.<field>`. A
single `create()` button POSTs the whole `$form` object.

```jinja
{% from '_ship.html.j2' import head, styles, signals, form, create, loading, spinner %}

<!DOCTYPE html>
<html>
<head>{{ head('New task') }}{{ styles() }}</head>
<body>
<div class="ship-view" {{ signals({'form': {}, '_creating': false}) }}>
  <ship-card style="max-width: 40rem;">
    <h2>Create task</h2>

    {{ form(ship_schema('task'), 'task') }}

    <ship-button-group>
      <ship-button variant="primary"
                   {{ create('task', '$form') }}
                   {{ loading('_creating') }}>
        Create
      </ship-button>
      {{ spinner('_creating') }}
    </ship-button-group>
  </ship-card>
</div>
</body>
</html>
```

**Primitives used:** `head`, `styles`, `signals`, `form`, `create`,
`loading`, `spinner`.

**When NOT to use:** if you need custom field ordering, conditional
visibility, or field-level validation, hand-write the inputs with
`bind('form.title')`. `form()` is the fast path, not the only path.
