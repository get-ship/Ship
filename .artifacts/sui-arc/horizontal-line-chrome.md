# Horizontal line in shell2 chrome — audit

## Symptom

Screenshots of `/home`, `/git`, `/skills`, `/dashboard` show a faint horizontal
line spanning the content area, roughly where the bottom panel dock would
appear if open. The dock is *closed* in those screenshots.

## Source

`<sui-panel-dock>` host CSS, `packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.sui:46`:

```css
:host {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  position: relative;
  background: var(--card);
  box-shadow: var(--shadow-edge-top);   /* ← the culprit */
}
```

`--shadow-edge-top` is defined in tokens / themes as a two-stop shadow whose
*second* stop is `0 -1px 0 <color>` — a 1px hard line above the box. Examples:

- `packages/ship-set/src/tokens.css:157` → `..., 0 -1px 0 oklch(0 0 0 / 4%)`
- `packages/ship-set/src/themes.css:103` → `..., 0 -1px 0 oklch(0.30 0.04 60 / 6%)`
- `packages/ship-set/src/themes.css:300` → `0 -1px 0 oklch(0.78 0.20 145 / 30%)`
- `packages/ship-set/src/themes.css:560` → `0 -1px 0 oklch(1 0 0 / 36%) inset`

In `apps/shell2/src/chrome/layout.css`, the collapsed dock is sized via
`height: 0; overflow: hidden; opacity: 0`. That hides the dock's *contents*,
but `box-shadow` paints outside the box and is unaffected by `overflow:
hidden`. `opacity: 0` does fade the shadow with the box — except themes whose
shadow alpha is high (e.g. terminal-green `30%`, retro-print `36%`) leave a
visible residue, and the line is even visible on default light/dark themes
under close inspection.

Net: a 1px hard line sits at the top of the (zero-height) dock host, which is
~`top-bar + content-row` down the viewport — the spot users see in the
screenshots.

## Why not the obvious suspects

- `border-top: 1px solid var(--border)` in `body.dock-open #ship-panel-dock`
  (`layout.css:82`) only applies when the dock is open. Not active in the
  screenshots.
- `sui-nav-top` has `border-bottom: 1px solid var(--border)` but that draws
  *under the top bar*, not mid-page.
- No `<hr>` element in `apps/shell2/index.html`.

## Fix

Suppress the dock's top-edge shadow while collapsed by overriding the custom
property on the host. CSS custom properties inherit through the shadow DOM
boundary, so this reaches the component's `:host` rule without touching its
internal CSS.

`apps/shell2/src/chrome/layout.css`:

```css
body:not(.dock-open) #ship-panel-dock { --shadow-edge-top: none; }
```

The open state (`body.dock-open`) already draws its own explicit
`border-top: 1px solid var(--border)`; the shadow returns automatically when
`dock-open` is present (the `body:not(.dock-open)` override no longer
matches), so the elevation cue is preserved while the dock is visible.

## Why not edit the component

`sui-panel-dock.sui` is the right home for the shadow when the dock is
visible — it's an elevation cue native to the surface. The bug is purely a
collapsed-state visibility issue scoped to shell2's grid layout. Fixing it in
`layout.css` keeps the component reusable for callers that don't apply the
`height: 0` trick (e.g. a future shell that uses `display: none` on the
collapsed dock would not need the override).

## Verification

After the fix:

- Collapsed dock: `body` lacks `dock-open` → `--shadow-edge-top: none` →
  host's `box-shadow: var(--shadow-edge-top)` resolves to `none` → no line.
- Open dock: `body.dock-open` → override stops matching → shadow restored
  from the theme; `border-top` also drawn by layout.css.
- Reduced-motion / `dock-popout`: unaffected — these key off the same
  `dock-open` class.

Acceptance: spot-check `/home`, `/git`, `/skills`, `/dashboard` with the dock
closed; no horizontal line should appear in the content area. Open the dock
(⌘J) and confirm the elevation shadow + border-top still read.
