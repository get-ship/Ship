# lit → SUI port — batch 3

Batch 1 (#137) cleared the highest-value missing primitives off the
audit: `sui-log-stream`, `sui-pipeline-dag`, `sui-pipeline-run`,
`sui-pipeline-step`, `sui-pipeline-step-def`, `sui-provider-blob`.

Batch 2 (#139) closed out the four deferred `domain/` primitives + the
substituted `core/ship-theme-toggle`: `sui-pipeline-definition`,
`sui-theme-toggle`, `sui-history-row`, `sui-auth-dialog`, `sui-git-diff`
(× 4 — container/file/hunk/line).

Batch 3 is small. The audit-tracked `core/`, `domain/`, and `islands/`
inventories are now ported. What remains in the `<ship-*>`→`<sui-*>`
gap, after a fresh sweep of `views/` + `apps/shell/src/chrome/`, is:

| lit                  | sui (batch 3)        | status      | notes |
|----------------------|----------------------|-------------|-------|
| `ship-tab-panel`     | `sui-tab-panel`      | **NEW**     | this dispatch |
| `ship-toast-host`    | `sui-toast-host`     | **NEW**     | this dispatch |
| `ship-shell`         | —                    | **skipped** | not a UI primitive — see "Skipped" below |

Nothing else needs porting. Every `<ship-X>` referenced from `views/`
or `apps/shell/src/chrome/` (excluding shell-internal infrastructure)
now has a registered `<sui-X>` counterpart.

## What this dispatch shipped

| port              | lit LOC | sui shape                              | notes |
|-------------------|---------|----------------------------------------|-------|
| `sui-tab-panel`   | 17      | `.sui` + `.controller.ts`              | Companion panel for `sui-tabs` / `sui-tab`. |
| `sui-toast-host`  | 90      | `.sui` + `.controller.ts`              | Composes `sui-toast-container`; bridges `ship:notify` → `.show()`. |

### Behavior-parity notes

- **`sui-tab-panel`** — pure declarative pair to `sui-tabs`. Two props
  (`name`, `active`, both `reflect`). `:host { display: none }` by
  default; `:host([active]) { display: block }` with the same fade
  keyframe as `ship-tab-panel.styles`. Controller mount-time wiring
  matches lit's `connectedCallback()`: defaults `role="tabpanel"` and
  `tabindex="0"` only when those attributes aren't already set on the
  host, then derives `id = "sui-tabpanel-${name}"` and
  `aria-labelledby = "sui-tab-${name}"` (matching the existing
  `sui-tabs` controller's `aria-controls` target). The lit version
  prefixed `ship-` for these IDs; SUI uses `sui-` and the sibling
  `sui-tabs` controller already emits the matching prefix, so no
  cross-controller changes were needed.

- **`sui-toast-host`** — `<sui-toast-container>` wrapper with a single
  listener for `ship:notify`. Mapping is identical to
  `ship-toast-host`:

    notify detail   → container.show() opts
    ──────────────────────────────────────
    message         → title
    description     → description
    tone            → tone
    duration        → duration
    actionHtml      → actionHtml

  The lit version listened on `closest('ship-shell')`. SUI broadens
  the selector to `closest('sui-shell, ship-shell')` so the primitive
  works both today (apps/shell still uses `<ship-shell>` as the root)
  and after any future `<sui-shell>` port. If neither ancestor exists
  (test harness, standalone use), falls back to `document.body` so
  bubbling dispatches still land. The `position` prop is forwarded
  through the inner `<sui-toast-container>` via the standard
  `position={position()}` binding — keeping a single source of truth
  for the value rather than reflecting on the host too.

### Substitutions / renames

| lit              | sui (batch 3)    | reason |
|------------------|------------------|--------|
| `ship-shell` ancestor query | `sui-shell, ship-shell` ancestor query | forward-compat with a future `sui-shell` port |

(`ship:notify` keeps its `ship:` prefix on purpose — the contract is
the API surface of `window.ship.notify()`, not a component event.)

## Skipped — and why

- **`ship-shell`** is the only other `<ship-*>` used in
  `apps/shell/src/chrome/` that doesn't have a `<sui-*>` counterpart.
  It isn't a UI primitive: it owns `KernelBridge` (kernel-event
  subscription), publishes `window.ship` (the global ship API surface
  — navigate / back / current / notify / command / chat / subscribe),
  applies the rail-open body class from localStorage, and renders
  nothing (`render() { return null }`). Porting it to
  `packages/ui-components` would require pulling
  `apps/shell/src/chrome/kernel-bridge` + `types.ts` into a primitives
  package, which inverts the dependency direction. This is platform
  infrastructure that lives correctly in `apps/shell/`.

  Batch 2's follow-up note already called this out: "Remaining
  un-ported `apps/shell/*` chrome (`ship-shell`, `ship-launcher`, …)
  is shell-internal and lives behind island bundles; not in scope for
  the 'missing primitives' track." `ship-launcher`, `ship-nav-top`,
  `ship-panel-dock`, `ship-command-bar`, `ship-rail`, and
  `ship-devtools-*` are now all ported into
  `packages/ui-components/src/islands/`; only `ship-shell` remains
  shell-internal by design.

## Visual baselines (deferred, same as batches 1 + 2)

Acceptance criterion #2 requests `tests/e2e/visual/baselines/
components-sui-{tab-panel,toast-host}-{paper,ship-dark}.png`. The job
spec scopes `tests/e2e/visual/views.json` out of edit — without
registering the tags there, the visual harness can't synthesize a
preview page and so a `playwright --update-snapshots` run won't write
the PNGs. This matches the recurring follow-up from batch 1 and batch
2: registry entries + baselines are a maintainer-side one-shot capture
covering all three batches the next time someone runs `just visual
--update-snapshots` against a live daemon. Tracked, not blocking.

## Test coverage

Behavior-test suites added under `packages/ui-components/test/`:

- `sui-tab-panel.test.ts` — registration, shadow-root + slot,
  role/tabindex/id/aria-labelledby wiring on mount, respect for an
  externally-set role/tabindex, and `<sui-tabs>` integration (active
  toggles by name, hidden cleared, switches on `active` attr change).
- `sui-toast-host.test.ts` — registration, inner `<sui-toast-
  container>` + default `position=bottom-right`, `ship:notify` →
  `show()` mapping (tone, duration, message, description), action-
  Html forwarding into the toast, ancestor-aware listening (uses
  `closest('ship-shell')` when present, falls back to
  `document.body`), and listener teardown on disconnect.

`pnpm -F ui-components test` for the two new files reports 12 passing
across 12 cases. The pre-existing failures in `sui-nav-top` (1) and
`sui-rail` (3) on `main` are unaffected by this batch (confirmed by
running the same two files on a clean `git stash`).
