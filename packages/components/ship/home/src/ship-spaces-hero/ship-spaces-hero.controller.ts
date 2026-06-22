// <ship-spaces-hero> — @ship/home · the spaces grid on the hub `/`.
//
// The home dashboard's spaces surface. Pure-derived presenter — the
// source of truth is the `spaces` attribute (JSON-encoded list). The
// hub controller passes `[]` today (the list-all-spaces SDK method is
// a known BACKEND-GAP; see .artifacts/hub-buildout-flags.md), and the
// empty-state takes over. When the SDK lands, the controller switches
// to `live(() => ship.spaces.list())` and the host pipes the result
// into the attribute; this controller stays stable.

import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-spaces-hero.sui'

interface SpaceRow {
  id: string
  name: string
  path: string
}

function parseSpaces(raw: string): SpaceRow[] {
  if (!raw) return []
  try {
    const v = JSON.parse(raw) as unknown
    if (!Array.isArray(v)) return []
    const out: SpaceRow[] = []
    for (const it of v) {
      if (it && typeof it === 'object') {
        const row = it as Partial<SpaceRow>
        if (
          typeof row.id === 'string' &&
          typeof row.name === 'string' &&
          typeof row.path === 'string'
        ) {
          out.push({ id: row.id, name: row.name, path: row.path })
        }
      }
    }
    return out
  } catch {
    return []
  }
}

export default controller({
  scope: ({ spaces, limit }: ViewScope<Scope>, host?: Element) => {
    const rows = derived<SpaceRow[]>(() => {
      const all = parseSpaces(spaces())
      const n = Number.isFinite(limit()) ? Math.max(0, Math.floor(limit())) : 8
      return all.slice(0, n)
    })
    const countLabel = derived<string>(() => {
      const n = rows().length
      return n === 0
        ? 'no spaces registered'
        : n === 1
          ? '1 space'
          : `${n} spaces`
    })

    const onOpen = (e: MouseEvent, id: string): void => {
      e.preventDefault()
      // ship-space-open — the host routes to the space surface. We
      // preventDefault so the hub's router (rather than a full-page
      // navigation to /spaces/:id) handles the click.
      host?.dispatchEvent(
        new CustomEvent('ship-space-open', {
          detail: { id },
          bubbles: true,
          composed: true,
        }),
      )
    }
    const onAdd = (): void => {
      host?.dispatchEvent(
        new CustomEvent('ship-space-add', {
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { rows, countLabel, onOpen, onAdd }
  },
})
