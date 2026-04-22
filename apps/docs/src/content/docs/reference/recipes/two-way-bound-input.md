---
title: "Two-way bound input"
description: "The hello-world of Ship reactivity — a ship-input bound to a signal with a data-text echo."
sidebar:
  label: "Two-way bound input"
  order: 5
---
When you need a form field whose value is read elsewhere in the view,
declare the signal at the root, `bind()` it on the input, and read it
with `data-text` (or any other Datastar-aware attribute) wherever you
need the live value. No submit step, no controller.

```jinja
{% from '_ship.html.j2' import head, signal, bind %}

<!DOCTYPE html>
<html>
<head>{{ head('Hello') }}</head>
<body>
<div class="ship-view" {{ signal('name', 'world') }}>
  <ship-card>
    <h1>Hello, <span data-text="$name"></span>!</h1>
    <ship-input {{ bind('name') }} placeholder="Type your name"></ship-input>
  </ship-card>
</div>
</body>
</html>
```

**Primitives used:** `head`, `signal`, `bind`.

**Common pitfall:** using `data-text="name"` without the `$` — that
renders the literal string "name". Datastar expressions reference
signals with a leading `$`.
