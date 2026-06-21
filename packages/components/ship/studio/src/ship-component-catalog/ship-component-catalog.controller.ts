// <ship-component-catalog> — @ship/studio · the hub `/studio/components` index.
//
// BACKEND-GAP controller (see the .sui for the data story): the catalog
// needs a `ship.ui.components.list()` Connect op that does not exist. The
// shell's `/_deck/catalog/workspace` endpoint was purged; the framework
// no longer writes `.deck/catalogs/workspace.json`; the file is a stale
// snapshot from a previous build, not a live index.
//
// Per the charter §Connect-only, the controller is wired with NO data
// source. It exposes the same shape the live-op wiring would expose
// (entries, groups, state, sub) — when the backend ships the op, the
// only change is to add `live(() => ship.ui.components.list())` and
// route the result through the existing `entries` derived. The
// template's `{#if state() === 'ready' && entries().length > 0}`
// branch flips on automatically — no template change required.
//
// The exports below are the contract the page composes; they are pure
// functions of the prop scope and the (currently empty) catalog
// signal. No I/O, no fetch, no Connect call — the controller is a
// pure-derived presenter over a (today empty) data signal.

import { controller, derived, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-component-catalog.sui'

/** One catalog row, in the shape a future `ship.ui.components.list()` op
 *  would return. Mirrors the shell's `CatalogEntry` minus the `kind`
 *  field — the hub doesn't host Lit / view components, only sui / ship. */
export interface CatalogEntry {
  /** Custom-element tag, e.g. `sui-card`, `ship-nav-top`. */
  tag: string
  /** Originating workspace package (drives the per-package pill). */
  pkg?: string
  /** Human-readable description; falls back to a prop-count string when
   *  the CEM doesn't carry one (matches the shell's pattern). */
  description?: string
  /** Number of declared props; drives the "N props" fallback caption. */
  propCount?: number
  /** Number of declared subcomponents; drives the "N parts" fallback. */
  partCount?: number
  /** Semantic group: Actions / Inputs / Layout / Feedback / Indicators /
   *  Overlays / Data / Navigation / Charts / Pipelines / Other. */
  category: string
}

export interface CatalogGroup {
  id: string
  label: string
  count: number
  cells: CatalogEntry[]
}

/** State machine the template reads.
 *  - `'ready'`    — entries present, the live branch renders.
 *  - `'gap'`      — no data; render the empty-state. */
export type CatalogState = 'gap' | 'ready'

/** Sub-text under the H1. Mirrors the shell's "N primitives · M charts
 *  · K pipelines" pattern, but the hub's BACKEND-GAP surface is a single
 *  "no catalog available" line — kept stable so the template doesn't
 *  fork on the gap. */
function subFor(state: CatalogState, count: number): string {
  if (state === 'gap') return 'BACKEND-GAP · no components op'
  if (count === 0) return 'no components'
  if (count === 1) return '1 component'
  return `${count} components`
}

export default controller<ViewScope<Scope>>({
  scope: ({ filter, pkg }: ViewScope<Scope>, host?: Element) => {
    // `entries` is the single source of truth — wired to a static
    // empty array today, ready to switch to a LiveSource the moment
    // the backend ships the op. The empty array makes the BACKEND-GAP
    // explicit (no fabricated data) and the shape stable (the
    // template's `entries().length > 0` test is the gate).
    const entries = signal<CatalogEntry[]>([])

    const state = derived<CatalogState>(() =>
      entries().length > 0 ? 'ready' : 'gap',
    )

    const sub = derived<string>(() => subFor(state(), entries().length))

    /** Grouped + filtered view; mirrors the shell's `groups` derived
     *  but for the ship pack's pure-presenter model (no async mount).
     *  The free-text filter is applied against tag + category + pkg; the
     *  pkg pill is applied against the `pkg` prefix. */
    const groups = derived<CatalogGroup[]>(() => {
      const text = (filter() ?? '').toLowerCase().trim()
      const wantPkg = (pkg() ?? 'all').trim() || 'all'
      const cells = entries().filter((e) => {
        if (wantPkg !== 'all' && (e.pkg ?? '') !== wantPkg) return false
        if (!text) return true
        return (
          e.tag.toLowerCase().includes(text) ||
          e.category.toLowerCase().includes(text) ||
          (e.pkg ?? '').toLowerCase().includes(text) ||
          (e.description ?? '').toLowerCase().includes(text)
        )
      })
      const byCat = new Map<string, CatalogEntry[]>()
      for (const c of cells) {
        const arr = byCat.get(c.category) ?? []
        arr.push(c)
        byCat.set(c.category, arr)
      }
      return [...byCat.entries()]
        .map(([label, list]): CatalogGroup => ({
          id: label.toLowerCase(),
          label,
          count: list.length,
          cells: list.sort((a, b) => a.tag.localeCompare(b.tag)),
        }))
        .sort((a, b) => a.label.localeCompare(b.label))
    })

    /** Click on a catalog cell. `preventDefault` keeps the host's
     *  router (not a full page nav) responsible for the navigation —
     *  same pattern as the home pack's `ship-space-open`. */
    const onOpen = (e: MouseEvent, tag: string): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-component-open', {
          detail: { tag },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { entries, state, sub, groups, onOpen }
  },
})
