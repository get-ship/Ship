// Render a `let` Instr — extends `scope` with a named *getter* over a
// compiled expression, then renders the body in the extended scope.
//
// The getter form (not a static snapshot) keeps the binding reactive:
// every read of `scope[name]` re-evaluates the underlying expression,
// so `{#async overview as v}` can wire `v` to `overview().value` and
// every `v.field` inside the body reads through the live signal.

import type { Disposer } from '@ship/sui-runtime'

import type { CompiledExpr } from './expr'
import type { Instr } from './compile'
import { extendScope, render, type Scope } from './render'

interface OneRender {
  nodes: Node[]
  mount: () => Disposer
}

export function renderLet(
  name: string,
  value: CompiledExpr,
  body: Instr[],
  scope: Scope,
  doc: Document,
): OneRender {
  const child = extendScope(scope, {})
  Object.defineProperty(child, name, {
    get: () => value(scope),
    enumerable: true,
    configurable: false,
  })
  const r = render(body, child, doc)
  return { nodes: r.nodes, mount: r.mount }
}
