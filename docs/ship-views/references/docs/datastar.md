---
group: Views
order: 5
title: Datastar & Signal Conventions
description: The exact Datastar version Ship pins, plus Ship-specific signal conventions layered on top — `_`-prefix client-only signals and the `data-ship-signal-types` schema.
audience: internal
---

# Datastar & Signal Conventions

Ship views use [Datastar](https://data-star.dev/) for client-side
reactivity. Ship pins Datastar exactly so dev, CI, and prod always
render the same behavior, and adds two conventions on top of vanilla
Datastar: a `_`-prefix that marks signals as client-only, and a
`data-ship-signal-types` schema attribute the view linter uses to
catch type mismatches.

## Pinned versions

| Surface                         | Pin               | Where                                                  |
|---------------------------------|-------------------|--------------------------------------------------------|
| Datastar runtime crate (Rust)   | `=0.3.1`          | `apps/shipd/Cargo.toml`                                |
| Datastar JS bundle              | `1.0.0-RC.7`      | `apps/shipd/assets/datastar.js` (vendored)             |

Both surfaces are pinned exactly. Upgrading either is a deliberate
contract change — bump both together, run the full view linter and
browser smoke tests, and update this page in the same commit. The
crate version uses Cargo's strict-equality syntax (`=0.3.1`); a bare
`"0.3.1"` would float across the `0.3.x` range.

## `_`-prefix: client-only signals

Any signal whose name starts with an underscore is **client-only**.
The convention is enforced by author discipline (the view linter
flags references to undeclared signals regardless of prefix), and
the daemon never reads or writes them — they exist only inside the
browser tab.

Use this prefix for:

- Toggle state for transient UI (`_showForm`, `_expanded`, `_menuOpen`).
- Per-tab loading flags driven by `indicator(...)` macros (`_saving`,
  `_loadingItems`).
- Local-only filter or sort selections that should not be persisted.

Anything that would round-trip to the server — collection IDs,
form payloads, identifiers an event handler will POST — must use a
non-underscored name. The two `_`-prefixed signals already shipping
in `_ship.html.j2`'s allowlist are `$_dsRequest` (Datastar's request
channel) and `$_dsResponse` (Datastar's response channel); these are
intrinsics, not user-declared.

Example:

```jinja
{% from '_ship.html.j2' import signal, show, indicator %}
<ship-card
  {{ signal('_showForm', false) }}
  {{ signal('form', {}) }}>
  <ship-button data-on:click="$_showForm = !$_showForm">
    Toggle
  </ship-button>
  <div {{ show('$_showForm') }}>
    <ship-input data-bind="form.title"></ship-input>
  </div>
</ship-card>
```

`$_showForm` never leaves the page. `$form` is sent the moment a
button macro POSTs `data: $form` to the kernel.

## `data-ship-signal-types`: type schema

The `signals(...)` macro emits a JSON schema attribute alongside the
signal declarations:

```jinja
<div {{ signals({'title': '', 'count': 0, 'active': false}) }}>
```

renders to:

```html
<div data-signals:title="''"
     data-signals:count="0"
     data-signals:active="false"
     data-ship-signal-types='{"title":"str","count":"int","active":"bool"}'>
```

The attribute is a flat JSON object mapping signal name → primitive
type tag. Recognized tags:

| Tag    | Source                                                    |
|--------|-----------------------------------------------------------|
| `str`  | String defaults.                                          |
| `bool` | Boolean defaults.                                         |
| `int`  | Integer defaults.                                         |
| `num`  | Non-integer numeric defaults.                             |
| `any`  | `none` defaults or shapes (objects, arrays).              |

Datastar itself ignores `data-ship-signal-types`. The schema is
consumed only by the view linter — the AST type-mismatch rule reads
it to flag assignments like `$count = 'hello'` when `count` was
declared as `int`. Stripping the attribute disables that check; it
does not change runtime behavior.

When you declare a single signal with `signal(name, default)` no type
attribute is emitted, so single-signal views aren't typed-checked.
For coverage, prefer `signals({...})` for any view with more than two
or three signals.

## Lint enforcement

`views-undefined-signal` runs at **error** severity by default. A
view that references `$count` without declaring `count` (via
`signal()`, `signals()`, raw `data-signals:count`, or the
`{#- @lint:signals-ok: count -#}` allowlist) fails `ship lint views`.

The `_`-prefix is not a special case for the linter — `$_showForm`
must still be declared with `signal('_showForm', false)`. The prefix
is a semantic convention for humans and reviewers; the linter
enforces declaration regardless.

## Related

- [Manifest & SDK reference](./reference.md) — `signal()`, `signals()`,
  `show()`, `indicator()` macros.
- [Lint Rule Catalog](./lint-rules.md) — `views-undefined-signal` and
  the AST type-mismatch rule.
- [Datastar upstream docs](https://data-star.dev/) — directives and
  modifiers Ship layers on top of.
