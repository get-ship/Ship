// Render a `{#for}` block — defers to the runtime's `bindFor` for
// keyed reconciliation, with a leading effect that mounts the
// `{:else}` branch when the iterable is empty.

import {
  bindFor,
  effect,
  type Disposer,
  type RenderedItem,
} from '@ship/ui-runtime'

import type { Instr } from './compile'
import { extendScope, render, type Rendered, type Scope } from './render'

interface OneRender {
  nodes: Node[]
  mount: () => Disposer
}

export function renderFor(
  instr: Extract<Instr, { kind: 'for' }>,
  scope: Scope,
  doc: Document,
): OneRender {
  const anchor = doc.createComment('for')
  return { nodes: [anchor], mount: () => mountFor(anchor, instr, scope, doc) }
}

function mountFor(
  anchor: Comment,
  instr: Extract<Instr, { kind: 'for' }>,
  scope: Scope,
  doc: Document,
): Disposer {
  let elseActive: { nodes: Node[]; cleanup: () => void } | null = null
  const tearDownElse = () => {
    if (!elseActive) return
    for (const n of elseActive.nodes) {
      try {
        n.parentNode?.removeChild(n)
      } catch {
        /* detached */
      }
    }
    elseActive.cleanup()
    elseActive = null
  }

  const itemsThunk = (): readonly unknown[] => {
    let v: unknown
    try {
      v = instr.iterable(scope)
    } catch {
      return []
    }
    if (Array.isArray(v)) return v
    if (v == null) return []
    if (typeof (v as { length?: number }).length === 'number') {
      return Array.from(v as ArrayLike<unknown>)
    }
    return []
  }

  const wrappedRender = (item: unknown, i: number): RenderedItem => {
    const additions: Scope = { [instr.varName]: item }
    if (instr.indexName) additions[instr.indexName] = i
    const childScope = extendScope(scope, additions)
    const r: Rendered = render(instr.body, childScope, doc)
    // bindFor inserts r.nodes after producing them. We mount bindings
    // here; pure leaf bindings (bindText, bindAttr) work on detached
    // nodes, so mounting before insert is safe for the typical row.
    const cleanup = r.mount()
    return { nodes: r.nodes, cleanup }
  }

  const dispose = bindFor<unknown>({
    anchor,
    items: () => itemsThunk(),
    key: (item, i) =>
      instr.key
        ? instr.key(
            extendScope(scope, {
              [instr.varName]: item,
              ...(instr.indexName ? { [instr.indexName]: i } : {}),
            }),
          )
        : i,
    render: wrappedRender,
  })

  const elseDispose = instr.else_
    ? effect(() => {
        const empty = itemsThunk().length === 0
        if (empty && !elseActive) {
          const r = render(instr.else_!, scope, doc)
          const parent = anchor.parentNode
          if (parent) {
            for (const n of r.nodes) parent.insertBefore(n, anchor)
          }
          const cleanup = r.mount()
          elseActive = { nodes: r.nodes, cleanup }
        } else if (!empty && elseActive) {
          tearDownElse()
        }
      })
    : null

  return () => {
    dispose()
    elseDispose?.()
    tearDownElse()
  }
}
