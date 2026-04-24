//! Shell-side defaults for the dock.
//!
//! `DEFAULT_PINS` is the shell's first-run opinion — the starter dock
//! users see before they've pinned/unpinned anything. Baked into the
//! shell package (tier-0 compiled code, changes require a rebuild)
//! because it's an app-level UX decision, not a user preference.
//!
//! Per-view display metadata (label, icon) now comes from each view's
//! `manifest.rail` block, served via `/api/views`. See
//! `packages/ship-shell/src/ship-rail.ts` for the consumer.

/** First-run dock. Order is the initial display order; users immediately
 *  own their pin array thereafter (drag-to-reorder lands in a later phase). */
export const DEFAULT_PINS: string[] = [
  "dashboard",
  "pipelines",
  "agents",
  "skills",
  "git",
  "collections",
  "settings",
];

/** Fallback label used when a view manifest has no `rail.label`. */
export function humanize(id: string): string {
  return id
    .replace(/-/g, " ")
    .split(" ")
    .map((w) => (w ? w[0].toUpperCase() + w.slice(1) : w))
    .join(" ");
}
