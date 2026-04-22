---
group: Recipes
title: Loading indicator
order: 10
description: Wrap any action in loading() + spinner() to show a busy state without managing signals by hand.
---

When you need a "working..." indicator on a button, add `loading()` to
the same element that carries the action and drop a `spinner()` next
to it, both referencing the same underscore-prefixed signal. The
action flips the signal while the fetch is in flight; the spinner
shows; the button stays on screen.

```jinja
{% from '_ship.html.j2' import signals, create_job, loading, spinner %}

<div class="ship-view" {{ signals({'_running': false, 'selectedAgent': 'reviewer'}) }}>
  <ship-button-group>
    <ship-button variant="primary"
                 {{ create_job('$selectedAgent', 'review') }}
                 {{ loading('_running') }}>
      Run review
    </ship-button>
    {{ spinner('_running') }}
  </ship-button-group>
</div>
```

**Primitives used:** `signals`, `create_job`, `loading`, `spinner`.

**Common pitfall:** using two different signal names for `loading()`
and `spinner()`. They must match — the spinner watches the signal the
loading macro writes.
