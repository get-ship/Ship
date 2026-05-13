# cmd-bar palette navigation wired end-to-end

## Symptom

`⌘K` opened the command bar; typing a view and pressing Enter / clicking a
result updated the URL silently and the page kept showing the old view.
The launcher (views picker) had the same regression — clicking a row in
`<sui-launcher>` was a no-op visually.

## Diagnosis

The cmd-bar palette's `activateNavItem(path)` already called `navigateTo(path)`,
which routes through `window.ship.navigate(path)` (set up by
`apps/shell2/src/main.ts`). `window.ship.navigate` did:

```ts
history.pushState({}, '', path)
document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
```

…but nothing in shell2 listened for that event to actually call
`mountRoute()`. The launcher had the same pattern — pushState +
dispatch `sui:navigate` with `detail.to` — and the same broken hop.

`sui:navigate` was being consumed by `<sui-rail>`, `<sui-nav-top>`, and the
devtools tabs purely for highlight state. The shell-level consumer that
remounts the view was missing.

## Fix

### 1. `apps/shell2/src/main.ts`

Added a document-level `sui:navigate` listener that calls `mountRoute(next)`
when the event carries `detail.to` and the resolved route differs from
`currentRoute`. The sameView guard mirrors `onAnchorClick`'s pattern and
prevents a remount loop from `mountRoute` → `updateNav`'s own dispatch.

### 2. `packages/ui-components/src/islands/sui-command-bar/sui-command-palette.controller.ts`

Added a `viewItems` derived signal that exposes installed views from
`bar.views()` (loaded from `/api/v1/views` at bar mount time) as a second
navigation section. Dedupes against `NAV_DESTINATIONS` paths so the
curated nav and dynamic view list don't both show "Dashboard". Indices
are offset past `findItems().length` so the shared `selectedIdx` walk in
DOM order stays consistent — `getPaletteItems()` queries `.cb-item` once.

### 3. `packages/ui-components/src/islands/sui-command-bar/sui-command-palette.sui`

Split the find-mode body into two sibling sections ("Jump to" + "Views")
guarded by their own length checks, with a single empty-state when both
are empty.

## Probe

`packages/ui-components/test/sui-command-bar.test.ts` — added two tests
under `navigation contract — shell listens for sui:navigate`:

- Clicking a find item dispatches `sui:navigate` with `detail.to` set to
  the item's `data-path`.
- Pressing `Enter` on a find result dispatches the same event with the
  selected item's path.

Run: `cd packages/ui-components && pnpm vitest run test/sui-command-bar.test.ts` →
41/41 pass (39 prior + 2 new). Launcher tests unchanged (17/17 pass) —
its existing `navigate()` already dispatched `sui:navigate` with `detail.to`,
so the new shell listener picks it up without launcher changes.

## Acceptance trace

1. `⌘K` → shell2 keyboard handler at main.ts:467 calls
   `getCommandBar()?.toggle('find')` → bar opens in find mode.
2. User types "dashboard" → palette's `onInput` updates `bar.value`;
   `findItems()` filters NAV_DESTINATIONS to the Dashboard row.
3. `Enter` → `onPaletteKeydown` calls `confirmSelection()` →
   `navigateTo('/dashboard')` → `window.ship.navigate('/dashboard')`.
4. `window.ship.navigate` pushes state and dispatches `sui:navigate`.
5. The new shell listener catches it, computes `next = { view: 'dashboard' }`,
   sees it differs from `currentRoute`, calls `mountRoute(next)`.
6. `mountRoute` fetches `/_sui/views/dashboard`, mounts; the views picker
   path is the same shape (launcher's `navigate(name)` builds the path and
   dispatches the same event).
