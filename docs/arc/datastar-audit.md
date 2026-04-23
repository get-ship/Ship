# Datastar Usage Audit

Snapshot of Datastar coupling across view templates. Tracks macro coverage so the decoupling seam from the shell spec stays honest.

**Taken at:** commit b3e37c04 (after shell phase 1, before phase 2)

## Raw Datastar attributes by view

283 occurrences total. Sorted:

| Count | File |
|------:|------|
| 53 | `collections/index.html.j2` |
| 19 | `agents/partials/panel_permissions.html.j2` |
| 19 | `agents/create/index.html.j2` |
| 17 | `overview/index.html.j2` |
| 15 | `setup/partials/step-provider.html.j2` |
| 15 | `pipelines/focus/[name]/index.html.j2` |
| 10 | `git/index.html.j2` |
| 9 | `pipelines/index.html.j2` |
| 8 | `git/partials/panel_changes_diff.html.j2` |
| 8 | `agents/partials/picker_dialog.html.j2` |
| 7 | `agents/[id]/settings/index.html.j2` |
| 6 | `setup/partials/step-workspace.html.j2` |
| 6 | `pipelines/partials/dispatch_dialog.html.j2` |
| 6 | `pipelines/focus/partials/edit_card.html.j2` |
| 6 | `dashboard/partials/quick_links.html.j2` |

Grep pattern: `data-on:|data-bind=|data-signals|data-show=|data-class:|data-attr:|data-init=`

Additional coupling:
- **151** `$signal` expressions inline in attributes
- **47** `@get`/`@post`/`@put`/`@patch`/`@delete` action shorthand calls

## Existing macro coverage

**73 macro uses** across all views. Macros exported from `_ship.html.j2`:
- `signal(name, initial)` — `data-signals:name="initial"`
- `live(view)` — SSE subscribe attrs
- `show(expr)` — `data-show`
- `attr(name, expr)` — `data-attr:name`
- `loading(signal)` — `data-attr:disabled="$signal"`
- `tab_panel(signal, value)` — tab-mode panel visibility

**Coverage ratio:** 73 macro uses / (73 + 283) raw = ~21% covered.

## Macros to add (from highest-frequency raw patterns)

| Macro | Replaces | Frequency |
|-------|----------|----------:|
| `on_click(action)` | `data-on:click="@get(...)"` / `data-on:click="$x = y"` | ~80 |
| `on_submit(action)` | `data-on:submit.prevent="@post(...)"` | ~15 |
| `on_change(action)` | `data-on:ship-change="..."` | ~20 |
| `bind(signal)` | `data-bind="signal"` | ~15 |
| `class_if(expr, cls)` | `data-class:foo="$x"` | ~25 |
| `attr_if(expr, name)` | `data-attr:disabled="$x"` | (merge with existing `attr`) |
| `init(action)` | `data-init="@get('.../sse')"` | ~8 |

These ~7 macros would absorb the majority of the 283 raw attributes.

## Non-coverage accepted

The `@get`/`@post` action syntax **inside** `data-on` attributes stays inline — wrapping it in macros destroys inline readability. Accepted Datastar-specific seam.

## Other coupling seams

1. **SSE patch format** (`views::routed_sse`) — emits `event: datastar-patch-elements` directly. No indirection layer today. Spec calls this out as a seam to add: a `shell_sse::emit_patch(selector, html)` primitive.
2. **Bundled as a core asset** (`ship-elements.js` + `datastar.js` load eagerly in `index.html`). Swapping would also change bundle loading.
3. **`packs:` manifest field** and pack injection have no Datastar-specific logic. Swappable.

## What this enables

If macro coverage gets to >90%, Datastar becomes replaceable at three seams:
1. Rewrite ~15 `_ship.html.j2` macro bodies
2. Replace the SSE emit primitive
3. Swap the bundled script

Without this discipline, swap cost is O(views × attributes). With it, O(macros).

## Next

Phase 2 of shell extraction is rail + nav — unrelated to macro coverage. But adding the 7 macros above could be a parallel pass (small, mechanical, test-friendly) before or after. Recommend: land the 7 macros, then sweep templates in a follow-up that brings coverage from 21% to ~90%.
