// State variant matrix helper for ship-ui-studio.
//
// Derives 1- or 2-axis grids from a component's enum props so the studio
// can show every state at a glance — the kind of Storybook-style sweep
// that catches "ghost-tone-warning has wrong contrast" without clicking
// through the prop panel one combination at a time.

import type { PropEntry } from '../catalog'

export interface MatrixAxis {
  name: string
  values: string[]
}

export interface MatrixCell {
  id: string
  label: string
  props: Record<string, string>
  propsJson: string
}

// When a component has `variant` + another enum, prefer this order for
// the second axis. `tone` is the canonical pair (button, badge, alert);
// `size` is the fallback for tone-less components (input, kbd).
const SECONDARY_AXIS_CANDIDATES = ['tone', 'size'] as const

export function deriveAxes(props: readonly PropEntry[]): MatrixAxis[] {
  const enums = props.filter(
    (p): p is PropEntry & { values: string[] } =>
      Array.isArray(p.values) && p.values.length > 0,
  )
  if (enums.length === 0) return []

  const variant = enums.find(p => p.name === 'variant')
  if (variant) {
    const secondary = SECONDARY_AXIS_CANDIDATES
      .map(name => enums.find(p => p.name === name && p !== variant))
      .find((p): p is PropEntry & { values: string[] } => p !== undefined)
    if (secondary) {
      return [
        { name: variant.name, values: variant.values },
        { name: secondary.name, values: secondary.values },
      ]
    }
    return [{ name: variant.name, values: variant.values }]
  }

  // Prefer `tone` then `size` as the sole axis when variant is absent.
  const fallback =
    enums.find(p => p.name === 'tone')
    ?? enums.find(p => p.name === 'size')
    ?? enums[0]
  return [{ name: fallback.name, values: fallback.values }]
}

export function buildCells(axes: readonly MatrixAxis[]): MatrixCell[] {
  if (axes.length === 0) return []

  if (axes.length === 1) {
    const [axis] = axes
    return axis.values.map(v => {
      const props = { [axis.name]: v }
      return {
        id: `${axis.name}-${v}`,
        label: v,
        props,
        propsJson: JSON.stringify(props),
      }
    })
  }

  const [a, b] = axes
  const cells: MatrixCell[] = []
  for (const av of a.values) {
    for (const bv of b.values) {
      const props = { [a.name]: av, [b.name]: bv }
      cells.push({
        id: `${a.name}-${av}-${b.name}-${bv}`,
        label: `${av} · ${bv}`,
        props,
        propsJson: JSON.stringify(props),
      })
    }
  }
  return cells
}

// Apply matrix-cell props to a freshly created host element. Mirrors the
// per-cell attribute semantics used by the single preview (boolean attrs
// are presence-only; unset values remove the attribute) but trimmed for
// the matrix case where every cell value is a non-empty enum string.
export function applyCellProps(
  node: Element,
  cellProps: Record<string, string>,
): void {
  for (const [k, v] of Object.entries(cellProps)) {
    if (v == null || v === '') node.removeAttribute(k)
    else node.setAttribute(k, v)
  }
}

// Instantiate one live element per `[data-matrix-cell]` host the template
// emitted. Caller owns the slot-composition lookup so the matrix helper
// stays decoupled from the studio's compound-component catalog.
export function mountMatrixCells(
  root: ParentNode,
  expectedTag: string,
  composeFor: (tag: string) => string | undefined,
): void {
  const hosts = root.querySelectorAll<HTMLElement>('[data-matrix-cell]')
  for (const host of hosts) {
    const cellTag = host.dataset.matrixTag
    if (!cellTag || cellTag !== expectedTag) continue
    let cellProps: Record<string, string> = {}
    try {
      cellProps = JSON.parse(host.dataset.props ?? '{}') as Record<string, string>
    } catch {
      continue
    }
    const cellNode = document.createElement(cellTag)
    cellNode.className = 'matrix-cell-rendered'
    const compose = composeFor(cellTag)
    if (compose) {
      const range = document.createRange()
      range.selectNodeContents(cellNode)
      cellNode.replaceChildren(range.createContextualFragment(compose))
    } else {
      cellNode.replaceChildren(document.createTextNode('example'))
    }
    applyCellProps(cellNode, cellProps)
    host.replaceChildren(cellNode)
  }
}
