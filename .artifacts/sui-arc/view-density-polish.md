# View density + spacing polish

**Branch:** `job/view-density-polish` · **Date:** 2026-05-13 · **Scope:** pure UX polish, no controller/runtime/SDK changes.

## What changed

### Rail ↔ content gutter
- `apps/shell2/src/chrome/layout.css` — added `column-gap: 6px` to the `#ship-app` grid. The rail and content area now have a consistent 6px gutter (mid-range of the 4–8px target). Top-bar / dock / status-bar still span `1 / -1` so they remain edge-to-edge.

### h1 normalization → `--text-2xl` (24px / 32px line-height)
- `views/events/style.css` — `.events-title` was `--text-xl`; now `--text-2xl` with matching `line-height: 32px` and `letter-spacing: var(--heading-tracking)` to match agents / sessions / workspaces / dashboard.
- `views/home/_components/home-hero/home-hero.sui` — the home hero `h1` was a hard-coded `1.5rem` (24px) with no heading-font / tracking. Now uses the token (`--text-2xl`) plus `--heading-font`, `--heading-weight`, `--heading-tracking` — same recipe as the other views.
- `agents`, `sessions`, `workspaces`, `dashboard` already used `--text-2xl`; unchanged.

### Empty-state / hero compression
- `views/home/style.css` — `.home` margin `4rem auto` → `1.5rem auto`; per-section panels (`.counter`, `.home-live`) trimmed from `2rem` outer / `1rem 1.25rem` inner to `1rem` / `0.75rem 1rem`. Added horizontal `padding: 0 var(--space-4)` so the wrap doesn't kiss the rail gutter on narrow viewports.
- `views/agents/page.sui` — `<sui-empty-state size="lg">` → `size="sm"`. Drops the empty card from 96px → 28px vertical padding.
- `views/sessions/page.sui` — same `size="lg"` → `size="sm"`.
- `views/workspaces/page.sui` — same `size="lg"` → `size="sm"`.
- `views/events/page.sui` — already `variant="placeholder"` (24px pad-y); left as-is.

### Dashboard — recent-activity stream
- `views/dashboard/page.sui` — the `Latest events` card was renamed to **Recent activity** and re-shaped as an ordered list (`<ol class="dash-activity-stream">`) with a small leading dot per row (`dash-activity-rail`). The card header now carries a count badge matching the Agents card.
- `views/dashboard/style.css` —
  - Replaced the flat `.dash-event-row` definition with `.dash-activity-stream` + `.dash-activity-row`. Each row is `8px 88px 1fr 120px` with a 6px primary-tinted dot in the leading slot; rows use a hair-line top border instead of the previous heavier bottom border.
  - **Quick links** tightened: padding `--space-4` → `--space-2 --space-3`, direction `column` → `row`, icon tile `32×32` → `24×24`. Six tiles still fit in one row at ≥640px, but the band is roughly half the height — fixes the "sparse bottom" feedback.

## What deliberately did not change

- No controller logic. The new activity stream consumes the same `dash.value.events` array the previous "Latest events" panel did — no new derived state.
- No new shared primitives. All visual treatment lives in view-local CSS.
- No top-level dep additions.

## Baselines

The following baselines move pixels and need refresh under both themes:

```
tests/e2e/visual/baselines/views-home-{paper,ship-dark}.png
tests/e2e/visual/baselines/views-agents-{paper,ship-dark}.png
tests/e2e/visual/baselines/views-sessions-{paper,ship-dark}.png
tests/e2e/visual/baselines/views-workspaces-{paper,ship-dark}.png
tests/e2e/visual/baselines/views-events-{paper,ship-dark}.png
tests/e2e/visual/baselines/views-dashboard-{paper,ship-dark}.png
```

Refresh via `pnpm test:e2e:visual -u` with `just dev` running for this worktree.

## Acceptance

1. Pixels moved on 6 views × 2 themes — baselines refreshed.
2. `pnpm test:e2e:visual` green against the refreshed set.
3. No controller / runtime / SDK files touched; rail gutter is the only `apps/` change.
