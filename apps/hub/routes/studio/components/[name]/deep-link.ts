// URL ↔ prop-panel sync for the studio detail view.
//
// readSearchParams seeds initial prop values from `?variant=ghost&tone=…`
// at mount time. writeSearchParams mirrors changes back via
// `history.replaceState` so the URL stays shareable without polluting
// history (back-button still returns to the catalog). highlightCell adds
// a `data-active-cell` attribute on the one matrix cell whose
// (propA, propB) pair matches the URL — CSS picks it up from there.

import type { PropEntry } from '../catalog'

export function readSearchParams(
  search: string,
  props: readonly PropEntry[],
): Record<string, string> {
  const known = new Set(props.map(p => p.name))
  const out: Record<string, string> = {}
  const params = new URLSearchParams(search)
  for (const [k, v] of params) {
    if (known.has(k)) out[k] = v
  }
  return out
}

export function writeSearchParams(
  values: Record<string, unknown>,
  props: readonly PropEntry[],
): void {
  const params = new URLSearchParams()
  for (const p of props) {
    const v = values[p.name]
    if (v == null || v === '') continue
    const str = String(v)
    // Booleans only land in the URL when explicitly true — skip the
    // implicit-false noise.
    if (str === 'false') continue
    if (p.default !== undefined && str === String(p.default)) continue
    params.set(p.name, str)
  }
  const q = params.toString()
  const next = q ? `${location.pathname}?${q}` : location.pathname
  if (next === location.pathname + location.search) return
  history.replaceState({}, '', next)
}

export function highlightActiveCell(
  root: ParentNode,
  values: Record<string, unknown>,
): void {
  for (const cell of root.querySelectorAll<HTMLElement>('.matrix-cell')) {
    cell.removeAttribute('data-active-cell')
  }
  for (const host of root.querySelectorAll<HTMLElement>('[data-matrix-cell]')) {
    let cellProps: Record<string, string> = {}
    try {
      cellProps = JSON.parse(host.dataset.props ?? '{}') as Record<string, string>
    } catch {
      continue
    }
    const keys = Object.keys(cellProps)
    if (keys.length === 0) continue
    const matches = keys.every(k => String(values[k] ?? '') === cellProps[k])
    if (!matches) continue
    const cell = host.closest<HTMLElement>('.matrix-cell')
    if (cell) cell.setAttribute('data-active-cell', 'true')
  }
}
