---
group: Recipes
title: Fetch-error surfacing
order: 12
description: Catch fetch failures on a specific action and render the message in a sui-alert via on_fetch_error.
---

When you need user-visible error reporting for a single action, attach
`on_fetch_error()` to the trigger element. The macro scopes a
document-level listener to that element and patches a signal with the
failure message. Render the signal through `show()` + `data-text` in a
`<sui-alert>`.

```jinja
{% from '_ship.html.j2' import signals, bind, post, on_fetch_error, show %}

<div class="ship-view" {{ signals({'payload': {}, 'err': ''}) }}>
  <sui-input {{ bind('payload.title') }} placeholder="Title"></sui-input>

  <sui-button variant="primary"
               {{ post('/api/specs', '$payload') }}
               {{ on_fetch_error('err') }}>
    Submit
  </sui-button>

  <sui-alert tone="destructive" {{ show('$err') }}>
    <span data-text="$err"></span>
  </sui-alert>
</div>
```

**Primitives used:** `signals`, `bind`, `post`, `on_fetch_error`,
`show`.

**When NOT to use:** for expected validation errors, server-patch the
form with field-level messages instead. `on_fetch_error` is for
unexpected transport failures (4xx/5xx), not form validation UX.
