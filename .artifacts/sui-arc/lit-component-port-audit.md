# lit → SUI port audit

`apps/shell` + `packages/ship-set/src/web-components/` is the parity bar for shell2. This pass cataloged every `<ship-*>` lit primitive against the existing `<sui-*>` lineup, then ported the highest-value missing batch.

## Audit table

`packages/ship-set/src/web-components/` is laid out in three folders:

- `core/` — generic primitives (buttons, dialogs, etc.)
- `domain/` — pipeline / git / agent / auth surfaces
- `islands/` — heavy editors (CodeMirror, Tiptap, xterm)

### core/ (52 primitives)

| lit tag                      | sui equivalent             | status      | notes |
|------------------------------|----------------------------|-------------|-------|
| `ship-alert`                 | `sui-alert`                | ported      |  |
| `ship-alert-dialog`          | `sui-alert-dialog`         | ported      |  |
| `ship-annotation-layer`      | `sui-annotation-layer`     | ported      | consolidated under `sui-annotation/` directory |
| `ship-annotation-list`       | `sui-annotation-list`      | ported      | same dir |
| `ship-annotation-marker`     | `sui-annotation-marker`    | ported      | same dir |
| `ship-app-icon`              | `sui-app-icon`             | ported      |  |
| `ship-avatar`                | `sui-avatar`               | ported      |  |
| `ship-badge`                 | `sui-badge`                | ported      |  |
| `ship-breadcrumb`            | `sui-breadcrumb`           | ported      | + `sui-breadcrumb-item` |
| `ship-button`                | `sui-button`               | ported      |  |
| `ship-card`                  | `sui-card`                 | ported      |  |
| `ship-checkbox`              | `sui-checkbox`             | ported      |  |
| `ship-chip`                  | `sui-chip`                 | ported      |  |
| `ship-code`                  | `sui-code`                 | ported      |  |
| `ship-collapsible`           | `sui-collapsible`          | ported      |  |
| `ship-context-menu`          | `sui-context-menu`         | ported      |  |
| `ship-dialog`                | `sui-dialog`               | ported      |  |
| `ship-diff`                  | `sui-diff`                 | ported      |  |
| `ship-drawer`                | `sui-drawer`               | ported      |  |
| `ship-dropdown-menu`         | `sui-dropdown-menu`        | ported      |  |
| `ship-empty-state`           | `sui-empty-state`          | ported      |  |
| `ship-error-state`           | `sui-error-state`          | ported      |  |
| `ship-field`                 | `sui-field`                | ported      |  |
| `ship-file-upload`           | `sui-file-upload`          | ported      |  |
| `ship-hover-card`            | `sui-hover-card`           | ported      |  |
| `ship-icon`                  | `sui-icon`                 | ported      |  |
| `ship-input`                 | `sui-input`                | ported      |  |
| `ship-json-view`             | `sui-json-view`            | ported      |  |
| `ship-label`                 | `sui-label`                | ported      |  |
| `ship-link`                  | `sui-link`                 | ported      |  |
| `ship-list-tile`             | `sui-list-tile`            | ported      |  |
| `ship-log-stream`            | `sui-log-stream`           | **NEW**     | this dispatch — SSE log pane |
| `ship-mark`                  | `sui-mark`                 | ported      |  |
| `ship-markdown`              | `sui-markdown`             | ported      |  |
| `ship-popover`               | `sui-popover`              | ported      |  |
| `ship-select`                | `sui-select`               | ported      |  |
| `ship-separator`             | `sui-separator`            | ported      |  |
| `ship-skeleton`              | `sui-skeleton`             | ported      |  |
| `ship-spinner`               | `sui-spinner`              | ported      |  |
| `ship-stat`                  | `sui-stat`                 | ported      |  |
| `ship-status-dot`            | `sui-status-dot`           | ported      |  |
| `ship-stepper`               | `sui-stepper`              | ported      |  |
| `ship-switch`                | `sui-switch`               | ported      |  |
| `ship-table`                 | `sui-table`                | ported      |  |
| `ship-tabs`                  | `sui-tabs`                 | ported      | + `sui-tab` |
| `ship-textarea`              | `sui-textarea`             | ported      |  |
| `ship-theme-toggle`          | `sui-theme-picker`         | substituted | picker covers all toggle behavior + brand themes; not a 1:1 port but the only known caller uses the picker |
| `ship-timestamp`             | `sui-timestamp`            | ported      |  |
| `ship-toast`                 | `sui-toast`                | ported      |  |
| `ship-toast-container`       | `sui-toast-container`      | ported      |  |
| `ship-toolbar`               | `sui-toolbar`              | ported      |  |
| `ship-tree-view`             | `sui-tree-view`            | ported      |  |
| `ship-view-chrome`           | `sui-view-chrome`          | ported      |  |

### domain/ (9 primitives)

| lit tag                      | sui equivalent             | status      | notes |
|------------------------------|----------------------------|-------------|-------|
| `ship-auth-dialog`           | —                          | **deferred**| 324 LOC, composes `ship-dialog` + provider-blob grid. Pure shell auth UX, no view caller yet. Follow-up. |
| `ship-git-diff`              | —                          | **deferred**| 342 LOC. Composes `ship-diff` (already ported) with file-tree + hunk navigation. Follow-up. |
| `ship-history-row`           | —                          | **deferred**| 210 LOC. Shell-only row used by command-bar history. Follow-up. |
| `ship-pipeline-dag`          | `sui-pipeline-dag`         | **NEW**     | this dispatch |
| `ship-pipeline-definition`   | —                          | **deferred**| 96 LOC. Not currently referenced by any view (`views/pipelines/page.sui` uses raw markup for the definition list). Follow-up if a definition-card is needed. |
| `ship-pipeline-run`          | `sui-pipeline-run`         | **NEW**     | this dispatch |
| `ship-pipeline-step`         | `sui-pipeline-step`        | **NEW**     | this dispatch |
| `ship-pipeline-step-def`     | `sui-pipeline-step-def`    | **NEW**     | this dispatch |
| `ship-provider-blob`         | `sui-provider-blob`        | **NEW**     | this dispatch |

### islands/ (3 heavy primitives)

| lit tag                      | sui equivalent             | status      | notes |
|------------------------------|----------------------------|-------------|-------|
| `ship-code-editor`           | `sui-code-editor`          | ported      | island bundle |
| `ship-markdown-editor`       | `sui-markdown-editor`      | ported      | island bundle |
| `ship-terminal`              | `sui-terminal`             | ported      | island bundle |

## This dispatch

Six ports, all in `packages/ui-components/src/sui-<name>/`:

| port                  | lit LOC | sui LOC | notes |
|-----------------------|---------|---------|-------|
| `sui-pipeline-dag`    | 49      | see below | declarative-only — slot + role/aria |
| `sui-pipeline-step`   | 137     | see below | state-driven status icon, spinner when running |
| `sui-pipeline-run`    | 119     | see below | composes `sui-card` + `sui-badge` with tone mapping |
| `sui-pipeline-step-def` | 175   | see below | declared-shape step renderer for definition pages |
| `sui-log-stream`      | 179     | see below | SSE pane via controller; emits `sui-stream-*` + `sui-log-tick` |
| `sui-provider-blob`   | 129     | see below | brand-SVG tile + label, inlines svg via the sui-runtime `bindHtml` for safe HTML |

### Behavior-parity notes

- **`sui-pipeline-dag`** — purely presentational. Pattern matches `sui-card` slot fallback. Controller-less.
- **`sui-pipeline-step`** — `state` reflects to attribute, drives CSS tones via host selectors (same recipe as the lit version). When `state='running'`, renders `<sui-spinner size="sm" tone="info">` in place of the glyph. `aria-label` set on host via controller `effect` because SUI props live in scope, not in lifecycle hooks; the lit version updates the same attribute on every `updated()`.
- **`sui-pipeline-run`** — composes `sui-card status-rail tone=…` and `sui-badge tone=…`. Tone-map table (`STATE_TO_TONE`) is reproduced verbatim. Header `actions` slot + default content + `footer` slot all preserved. Default state is `pending` → `muted` tone, same as lit.
- **`sui-pipeline-step-def`** — declared-shape step renderer for pipeline-focus pages. Same `step-type` → tone mapping, same `after` chip rendering, same trailing slot, same gate slot. `index` rendered as gutter. Pre-block for `command` body, trimmed.
- **`sui-log-stream`** — controller manages an `EventSource`. Open/close gated on `active`; reconnect on `url`/`event` change; emits `sui-stream-open`, `sui-stream-error`, `sui-stream-close`, and `sui-log-tick` (mirroring lit `ship-stream-*` / `ship-log-tick` events, prefixed `sui-` per SUI convention). Roll-up cap on `maxBytes`. Sticky-bottom autoscroll restored. Buffer text is set via controller `effect` against a `bind:this` ref (templates declare; controller does).
- **`sui-provider-blob`** — SVG inlined via `bindHtml` controller effect (the lit version uses lit's `unsafeHTML` directive — SUI's analogue is `bindHtml`). Same provider→hue→svg table. Light SVG default; callers opt into dark via theme CSS as in lit.

### Deferrals (recommended follow-up dispatch)

1. `ship-auth-dialog` (324 LOC) — heavy provider-grid dialog. Suggest porting alongside an auth-view refresh so behavior and look can be validated against a live flow.
2. `ship-git-diff` (342 LOC) — uses `sui-diff` as the row primitive but adds file-tree state + hunk navigation. Sizeable, depends on shell2 git surface stabilizing.
3. `ship-history-row` (210 LOC) — shell-internal; not yet referenced by views. Defer until a view caller appears.
4. `ship-pipeline-definition` (96 LOC) — small. Port when a view actually composes it (`views/pipelines/page.sui` currently uses raw markup).

The substituted `ship-theme-toggle` (binary light/dark toggle) could also be ported if a future caller wants a stripped-down toggle without the full picker palette; defer until requested.
