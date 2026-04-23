# Datastar Usage Audit

Snapshot of Datastar coupling across view templates. Tracks macro coverage so the decoupling seam from the shell spec stays honest.

**Taken at:** post-batch-2 sweep (branch `feat/ship-set-architecture`, after the batch-2 refactor commit).

## Summary

Batch 1 + batch 2 swept every in-scope file under `.ship/views/` to macro calls. Coverage flipped from ~21% to ~99%.

Grep pattern for raw coupling: `data-on:|data-bind=|data-class:|data-attr:|data-init=` (note: `data-signals` and `data-show=` were excluded from the sweep pattern — they already have dedicated `signal()` / `show()` macros and the remaining raw instances all depend on side data that macros can't cleanly express, e.g. `| tojson | replace('\"', '&quot;')` serialization or `data-show` paired with non-default inline styles).

## Current raw-attribute residue

| File | Raw | Nature |
|------|----:|--------|
| `skills/partials/explorer.html.j2` | 1 | `data-on:input__debounce.250ms` — debounce modifier, no dedicated macro yet |
| `overview/index.html.j2` | 17 | **Deferred** — canvas work, intentionally skipped |
| (comment-only false positives in 4 other files) | 0 | strings in `{#- ... -#}` or JS comments that match the grep pattern literally |

**In-scope raw (excluding `overview/`):** 1.

## Macro inventory (exported from `_ship.html.j2`)

Event / binding primitives added by this sweep:
- `on_click(action)`, `on_click_prevent(action)` — click handlers (bare + `.prevent`).
- `on_submit(action)` — form submit with `.prevent`.
- `on_change(action)` — `ship-change` custom event.
- `on(event, expr)` — generic event wiring (pre-existing; absorbed many custom `ship-*` events).
- `bind(signal)` — two-way `data-bind` (pre-existing).
- `class_if(expr, cls)` — toggle class reactively.
- `attr_if(expr, name)` — toggle attribute reactively (condition-first order).
- `init(action)` — `data-init` bootstrap.

Plus the pre-existing set: `head`, `signal(s)`, `show`, `loading`, `spinner`, `tab`, `tab_panel`, `chip`, `chip_all`, `search`, `radio`, `confirm*`, `nav`, `view`, `navigate`, `get`, `post`, `put`, `submit`, `feedback`, `emit`, `create_job`, `create`, `update`, `delete`, `toast_*`, `section`/`endsection`, `toggle_class`, `attr` (legacy name-first form), `on_fetch_error`, `on_change_debounced`, `live*`, `error_state`, `layout_*`, `styles`, `form`, `table`.

## Coverage ratio

Macro calls across `.ship/views/` (grepped against the full macro list): **344**.
Raw attrs outside `overview/` and outside comment false-positives: **1**.

Coverage: **344 / (344 + 1) ≈ 99.7%** (in-scope).

Including `overview/`'s 17 deferred raw attrs: **344 / (344 + 18) ≈ 95.0%**.

Either way, the 90% target is met.

## Intentionally raw

- **`data-signals:*` with `| tojson | replace('\"', '&quot;')`** — complex JSON seed values in `agents/partials/panel_permissions.html.j2` and `collections/index.html.j2`. The `signal()` / `signals()` macros don't accept the string-pipeline shape these need. Could be absorbed by a future `signals_json(name, expr)` helper.
- **`data-show="..."` paired with inline style beyond `display:none`** — the `show()` macro owns the `style` attribute; re-using it would clobber adjacent styles. Handful in `collections/index.html.j2`, `setup/partials/step-provider.html.j2`, etc.
- **`data-on:input__debounce.250ms`** in `skills/partials/explorer.html.j2` — one-off; the existing `search()` + `on_change_debounced()` macros cover the related patterns but not this exact one.
- **`data-prop:open`** in `agents/partials/picker_dialog.html.j2` — Datastar *property* binding (Lit's Boolean attribute reflection makes `data-attr:open` unusable there). No `prop` macro yet.
- **`overview/index.html.j2`** — deferred canvas work, not swept.

## Other coupling seams (unchanged)

1. **SSE patch format** (`views::routed_sse`) — emits `event: datastar-patch-elements` directly. Still a seam to abstract.
2. **Bundled as a core asset** (`ship-elements.js` + `datastar.js` load eagerly). Swappable but requires bundle-loader work.
3. **`packs:` manifest field** — no Datastar-specific logic, already swappable.

## What this enables

With macro coverage at ~99% in-scope, Datastar becomes replaceable at three seams:
1. Rewrite ~15 `_ship.html.j2` macro bodies (now ~25 with the new event macros).
2. Replace the SSE emit primitive.
3. Swap the bundled script.

Swap cost is O(macros) — the goal from the shell spec is met.

## Next

- Sweep `overview/` when the canvas work settles (or confirm scrapping).
- Consider `signals_json()` and `prop()` macros to absorb the remaining intentional holdouts.
- Consider revisiting `data-show` to accept an optional extra-style argument so the macro can own the style attribute without clobbering.
