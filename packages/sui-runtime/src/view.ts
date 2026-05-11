// view() — wraps a setup function as the default export of a compiled
// SUI view module.
//
// The sui-compiler emits every `.sui` view as:
//
//   import { view, … } from "@ship/sui-runtime"
//   export default view((ctx) => { …; return { …$scope, __mount(c) {…} } })
//
// `attach()` (in @ship/sui-dom) reads `module.default.setup(ctx)` to seed
// scope and then walks SSR DOM using the sibling `bindings` manifest.
// This factory is the contract glue between those two halves.

import type { ShipSDK } from './ship'
import type { RouteSignals } from './route'

export interface ViewContext {
  ship: ShipSDK
  route: RouteSignals
  navigate: (path: string) => void
}

export type ViewSetup<Bindings = Record<string, unknown>> = (
  ctx: ViewContext,
) => Bindings

export interface ViewDef<Bindings = Record<string, unknown>> {
  setup: ViewSetup<Bindings>
}

export function view<Bindings = Record<string, unknown>>(
  setup: ViewSetup<Bindings>,
): ViewDef<Bindings> {
  return { setup }
}
