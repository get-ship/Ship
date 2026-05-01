// Render `<while-loading bind="x">` and `<while-error bind="x" let:e>`
// elements. Each one defers to the matching runtime helper
// (`bindWhileLoading` / `bindWhileError`) and mounts the branch's own
// bindings inline — leaf bindings work on detached nodes, and
// branch-internal anchors are wired up after the runtime inserts
// them into the parent.

import {
  bindWhileLoading,
  bindWhileError,
  type Disposer,
  type LiveSignal,
} from '@ship/ui-runtime'

import type { Instr } from './compile'
import { extendScope, render, type Scope } from './render'

interface OneRender {
  nodes: Node[]
  mount: () => Disposer
}

export function renderWhileLoading(
  bindName: string,
  body: Instr[],
  scope: Scope,
  doc: Document,
): OneRender {
  const anchor = doc.createComment('while-loading')
  return {
    nodes: [anchor],
    mount: () => {
      const sig = scope[bindName] as LiveSignal<unknown> | undefined
      if (typeof sig !== 'function') {
        throw new Error(
          `[ship-set-dom] <while-loading bind="${bindName}">: scope value is not a signal`,
        )
      }
      return bindWhileLoading({
        anchor,
        signal: sig,
        render: () => {
          const r = render(body, scope, doc)
          const childCleanup = r.mount()
          return { nodes: r.nodes, cleanup: childCleanup }
        },
      })
    },
  }
}

export function renderWhileError(
  decl: { bind: string; errorVar: string },
  body: Instr[],
  scope: Scope,
  doc: Document,
): OneRender {
  const anchor = doc.createComment('while-error')
  return {
    nodes: [anchor],
    mount: () => {
      const sig = scope[decl.bind] as LiveSignal<unknown> | undefined
      if (typeof sig !== 'function') {
        throw new Error(
          `[ship-set-dom] <while-error bind="${decl.bind}">: scope value is not a signal`,
        )
      }
      return bindWhileError<Error>({
        anchor,
        signal: sig,
        render: (error) => {
          const child = extendScope(scope, { [decl.errorVar]: error })
          const r = render(body, child, doc)
          const childCleanup = r.mount()
          return { nodes: r.nodes, cleanup: childCleanup }
        },
      })
    },
  }
}
