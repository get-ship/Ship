// Execute a compiled instruction tree against a scope and produce DOM
// nodes. Bindings (bindText, bindAttr, bindIf, …) are deferred to a
// post-mount phase so that anchors and elements have valid parents
// when the runtime's bind* helpers do their initial inserts. The
// caller appends the returned nodes to a parent first, then calls
// `mount()` to wire up reactivity.

import { bindText, effect, type Disposer } from '@ship/sui-runtime'

import type { Instr } from './compile'
import { applyAttr } from './render-attrs'
import { renderFor } from './render-for'
import { renderWhileLoading, renderWhileError } from './render-while'

/** Scope is a plain object; child scopes inherit via prototype chain. */
export type Scope = Record<string, unknown>

/** Output of `render()` — DOM nodes plus a deferred mount callback. */
export interface Rendered {
  nodes: Node[]
  /** Run after `nodes` are inserted into a parent. Returns a disposer. */
  mount: () => Disposer
}

/** Make a child scope that prototypically inherits from `parent`. */
export const extendScope = (parent: Scope, additions: Scope): Scope =>
  Object.assign(Object.create(parent) as Scope, additions)

const composeDisposers = (ds: Disposer[]): Disposer => () => {
  for (const d of ds) {
    try {
      d()
    } catch (e) {
      console.error('[ship-set-dom] cleanup threw:', e)
    }
  }
}

/** Render instructions; returns nodes and a mount callback. */
export function render(
  instrs: Instr[],
  scope: Scope,
  doc: Document,
): Rendered {
  const nodes: Node[] = []
  const mounts: Array<() => Disposer> = []
  for (const instr of instrs) {
    const r = renderOne(instr, scope, doc)
    for (const n of r.nodes) nodes.push(n)
    mounts.push(r.mount)
  }
  return {
    nodes,
    mount: () => composeDisposers(mounts.map((m) => m())),
  }
}

interface OneRender {
  nodes: Node[]
  mount: () => Disposer
}

const NO_MOUNT: () => Disposer = () => () => {}

function renderOne(instr: Instr, scope: Scope, doc: Document): OneRender {
  switch (instr.kind) {
    case 'static-text':
      return { nodes: [doc.createTextNode(instr.text)], mount: NO_MOUNT }
    case 'comment':
      return { nodes: [doc.createComment(instr.data)], mount: NO_MOUNT }
    case 'dynamic-text': {
      const t = doc.createTextNode('')
      return {
        nodes: [t],
        mount: () => bindText(t, () => instr.expr(scope)),
      }
    }
    case 'element':
      return renderElement(instr, scope, doc)
    case 'if':
      return renderIf(instr, scope, doc)
    case 'for':
      return renderFor(instr, scope, doc)
    case 'match':
      return renderMatch(instr, scope, doc)
    case 'while-loading':
      return renderWhileLoading(instr.bind, instr.body, scope, doc)
    case 'while-error':
      return renderWhileError(
        { bind: instr.bind, errorVar: instr.errorVar },
        instr.body,
        scope,
        doc,
      )
  }
}

function renderElement(
  instr: Extract<Instr, { kind: 'element' }>,
  scope: Scope,
  doc: Document,
): OneRender {
  const el = doc.createElement(instr.tag)
  const childRender = render(instr.children, scope, doc)
  for (const c of childRender.nodes) el.appendChild(c)
  return {
    nodes: [el],
    mount: () => {
      const disposers: Disposer[] = []
      for (const a of instr.attrs) applyAttr(el, a, scope, disposers)
      disposers.push(childRender.mount())
      return composeDisposers(disposers)
    },
  }
}

function renderIf(
  instr: Extract<Instr, { kind: 'if' }>,
  scope: Scope,
  doc: Document,
): OneRender {
  const anchor = doc.createComment('if')
  // A branch that throws while computing its cond is treated as
  // inactive. This lets nested ifs co-exist without re-render-order
  // races (e.g. `{#if x.kind === 'ok'}{#if x.value.length === 0}…`
  // where the inner cond reads `x.value` that's undefined while the
  // outer effect hasn't fired yet).
  const all: { cond: () => boolean; body: Instr[] }[] = instr.branches.map(
    (b) => ({
      cond: () => {
        try {
          return Boolean(b.cond(scope))
        } catch {
          return false
        }
      },
      body: b.body,
    }),
  )
  return {
    nodes: [anchor],
    mount: () => mountChainedIf(anchor, all, instr.else_, scope, doc),
  }
}

export function mountChainedIf(
  anchor: Comment,
  branches: { cond: () => boolean; body: Instr[] }[],
  else_: Instr[] | undefined,
  scope: Scope,
  doc: Document,
): Disposer {
  let active: { nodes: Node[]; cleanup: () => void } | null = null
  let lastIdx = -2
  const tearDown = () => {
    if (!active) return
    for (const n of active.nodes) {
      try {
        n.parentNode?.removeChild(n)
      } catch {
        /* already detached */
      }
    }
    active.cleanup()
    active = null
  }
  const dispose = effect(() => {
    let pickIdx = -1
    for (let i = 0; i < branches.length; i++) {
      if (branches[i]!.cond()) {
        pickIdx = i
        break
      }
    }
    if (pickIdx === lastIdx) return
    tearDown()
    lastIdx = pickIdx
    const body = pickIdx === -1 ? (else_ ?? []) : branches[pickIdx]!.body
    if (body.length === 0) {
      active = { nodes: [], cleanup: () => {} }
      return
    }
    const r = render(body, scope, doc)
    const parent = anchor.parentNode
    if (parent) {
      for (const n of r.nodes) parent.insertBefore(n, anchor)
    }
    const cleanup = r.mount()
    active = { nodes: r.nodes, cleanup }
  })
  return () => {
    dispose()
    tearDown()
  }
}

function renderMatch(
  instr: Extract<Instr, { kind: 'match' }>,
  scope: Scope,
  doc: Document,
): OneRender {
  const branches = instr.cases.map((c) => ({
    cond: () => {
      try {
        return instr.discriminant(scope) === c.test(scope)
      } catch {
        return false
      }
    },
    body: c.body,
  }))
  const anchor = doc.createComment('match')
  return {
    nodes: [anchor],
    mount: () => mountChainedIf(anchor, branches, instr.default_, scope, doc),
  }
}
