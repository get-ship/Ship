/*
 * window-lazy.ts — deferred window bodies.
 *
 * The desktop renders one window's body live (the open/default one) and wraps
 * every *closed* window's body in an inert `<template data-window-body>`. That
 * keeps ~5k DOM nodes and all their CE hydration + per-window scripts off
 * the homepage's critical path until a window is actually opened.
 *
 * Each window/game script wraps its setup in `whenWindowBody(app, init)`:
 *   • if the body is already live (the open window) → run init immediately,
 *     identical to the old eager behaviour;
 *   • if it's still in a template → register init; desktop.ts clones the
 *     template on first open and calls `hydrateWindowBody(app)`, which runs the
 *     registered inits against the now-live content.
 *
 * Backward-compatible: for a non-deferred window the wrap is a no-op, so a
 * script can adopt it without behaviour change.
 */

type Init = () => void;

const pending = new Map<string, Init[]>();
const hydrated = new Set<string>();

function bodyLive(app: string): boolean {
  const w = document.querySelector(`[data-app-window="${app}"]`);
  if (!w) return false;
  // Deferred while an un-expanded body template is still present.
  if (w.querySelector('template[data-window-body]')) return false;
  return (w.querySelector('.content')?.childElementCount ?? 0) > 0;
}

/** Run `init` once the named window's body is in the live DOM. */
export function whenWindowBody(app: string, init: Init): void {
  if (hydrated.has(app) || bodyLive(app)) { init(); return; }
  const list = pending.get(app);
  if (list) list.push(init);
  else pending.set(app, [init]);
}

/** desktop.ts calls this right after cloning a window's body template in. */
export function hydrateWindowBody(app: string): void {
  if (hydrated.has(app)) return;
  hydrated.add(app);
  const list = pending.get(app);
  pending.delete(app);
  for (const fn of list ?? []) fn();
}
