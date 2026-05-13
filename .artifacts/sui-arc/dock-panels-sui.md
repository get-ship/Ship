# Dock panels — flesh out terminal/source/signals/lint/schema; swap Components to SUI inventory

## Why

User report: "terminal/source etc panels do not render much" and "just
components and that's the dated lit components". `<sui-panel-dock>` on
shell2 opened with six tabs, but most surfaces were empty and the
Components tab was reading the old `ship-*` lit catalog instead of the
SUI inventory.

## Findings (before)

| Tab        | State                                                                                  |
| ---------- | -------------------------------------------------------------------------------------- |
| Terminal   | Slot empty. `apps/shell2/index.html:120` carried a comment saying terminal loads on demand, but nothing actually injected a `<sui-terminal>`. |
| Source     | Wired to `/api/v1/view-source/...`. Renders + lints + saves. Honest empty state when no view. |
| Signals    | Polled `window._ds.signals` (Datastar legacy). shell2 is SUI-only, so the tree was always `0 / 0`. |
| Lint       | Wired to `POST /api/v1/lint/view`. Clean / loading / error states all present.         |
| Schema     | Wired to `/api/v1/ship/schema` + `/api/v1/view-source/...`. Empty state when no `ship.*` refs. |
| Components | Fetched `/api/assets/custom-elements.json` (the lit `<ship-*>` CEM). Wrong catalog.     |

## Changes

### 1. Terminal slot → auto-mount `<sui-terminal>` via dock controller

`packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.controller.ts`

The dock now fills its own `terminal` slot if no host provided one. On
mount, `ensureTerminalSlotFilled(host)`:

- Bails if `host.querySelector(':scope > [slot="terminal"]')` already
  resolves to a child — outer shells (or tests) can still inject a
  custom engine.
- Otherwise appends `<sui-terminal slot="terminal" fit>` as a light-DOM
  child and lazy-loads `/_sui/islands/sui-terminal/island.js` via a
  dedup-tagged `<script type="module" data-sui-terminal-island>`.

`index.html` did not need to change. shell2's existing inline comment
("terminal loads as island bundle on demand") was finally honored.

### 2. Signals → read SUI scope, not Datastar

`packages/ui-components/src/islands/sui-devtools/sui-devtools-signals.controller.ts`

`window.__suiCurrentScope` is now the source. `apps/shell2/src/main.ts`
publishes it after every successful `mountRoute()` and clears it on
landing/dispose:

```ts
function publishScope(scope: Record<string, unknown> | null): void {
  window.__suiCurrentScope = scope
  document.dispatchEvent(new CustomEvent('sui:scope:change', { detail: { scope } }))
}
```

The signals controller:

- Subscribes to `document/'sui:scope:change'` so it re-samples on
  navigation without polling delay.
- Walks the scope object, calls `__peek()` on signal getters (the
  canonical non-tracking read — keeps the readout out of any ambient
  observer), skips framework helpers (`ship`, `route`, `navigate`) and
  plain method members.
- Keeps the 500ms poll while the tab is active for live values.

This is the minimal scope-exposure surface — no runtime API change, just
a `window.__suiCurrentScope` field + a `sui:scope:change` event. ~14 LOC
in `main.ts`, well under the 30-LOC budget the job spec allowed.

### 3. Components → swap to SUI inventory

`packages/ui-components/src/islands/sui-devtools/sui-devtools-components.controller.ts`

Single URL change: `/api/assets/custom-elements.json` →
`/api/assets/sui-custom-elements.json`. The latter is the CEM emitted
by `sui-build` for every `<sui-*>` in `packages/ui-components/src` (see
`apps/shipd/src/routes.rs:136`). The lit catalog is unchanged and still
served at the old URL for `apps/shell` (j2).

Loading copy adjusted to mention the SUI source and the `just build`
hint when the catalog is empty.

### 4. Source / Lint / Schema — left alone

All three already wire to real daemon endpoints and have honest empty
states. Verified in code; nothing to change. The user report was
dominated by Terminal (empty slot), Signals (wrong source), and
Components (wrong catalog).

## Tests

`packages/ui-components/test/sui-panel-dock.test.ts`
- Added: auto-fill mounts a `<sui-terminal slot="terminal" fit>` light-DOM child.
- Added: a host-provided terminal slot child is preserved (no double-mount).

`packages/ui-components/test/sui-devtools.test.ts`
- Updated: `Loading components` → `Loading sui-* components` to match new copy.
- Updated: signals filter test now stubs `window.__suiCurrentScope` with
  fake signal getters (callable + `__peek`) instead of Datastar `_ds`.
- Added: components fetch hits `/api/assets/sui-custom-elements.json`,
  not the legacy lit CEM.

Note: the worktree's vitest setup currently fails to import
`packages/ship-set/src/web-components/core/lucide-icons.generated`
(generated file missing in this checkout). That's a pre-existing
environment gap unrelated to these changes; `sui-build` compiled clean
on the modified files.

## Files touched

- `apps/shell2/src/main.ts`
- `packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.controller.ts`
- `packages/ui-components/src/islands/sui-devtools/sui-devtools-components.{sui,controller.ts}`
- `packages/ui-components/src/islands/sui-devtools/sui-devtools-signals.{sui,controller.ts}`
- `packages/ui-components/test/sui-panel-dock.test.ts`
- `packages/ui-components/test/sui-devtools.test.ts`

## Out of scope (deferred)

- A dedicated `sui-devtools-terminal` sub-tab that owns shell/PTY wiring
  — current behavior is to mount a bare `<sui-terminal>` engine; piping
  it to a daemon-side shell is a separate job.
- Signals tab: nested-object expansion / type chips. Today it stringifies
  the peeked value. Good enough for the user-reported gap.
