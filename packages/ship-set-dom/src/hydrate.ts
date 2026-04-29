// Public hydrate entry point. Parses a `.html.set` source, runs the
// optional paired setup, builds the scope from <data> / <state> /
// setup return, compiles the template body, and inserts the rendered
// DOM into `opts.container`. Returns a disposer that tears the whole
// tree down: every signal subscriber, every event listener, every
// DOM node added by the call.

import { parseSetSource } from './parse'
import { compileChildren } from './template/compile'
import { extendScope, render, type Scope } from './template/render'
import { mountData } from './directives/data'
import { mountState } from './directives/state'
import { registerComponent } from './directives/component'

/** Options accepted by `hydrate`. */
export interface HydrateOptions {
  /** Where to mount the rendered nodes. */
  container: HTMLElement
  /** Route parameters captured by the surrounding navigator. */
  route?: Record<string, string>
  /** Live SDK handle exposed to setup and `<data>` source expressions. */
  ship?: unknown
  /** Client-side navigator passed through to setup. */
  navigate?: (path: string) => void
}

/** Handle returned by `hydrate`. */
export interface HydrateHandle {
  /** Tears down the rendered tree, every binding, and every effect. */
  dispose: () => void
}

/**
 * Parse `template`, run the optional paired `setup`, and mount the
 * resulting view (or register the resulting component) against
 * `opts.container`.
 *
 * For a non-component template:
 *  - `<data>` declarations evaluate first; live ones become
 *    `LiveSignal`s, one-shots become plain values.
 *  - `<state>` declarations seed local Signals.
 *  - `setup` (if provided) runs with `{ ship, route, navigate }` and
 *    its return value is merged into scope (so authors can expose
 *    derived signals or callbacks to the template).
 *  - The template body renders into `container`.
 *
 * For a component template (`<component>` directive present):
 *  - The body becomes the component's render function.
 *  - `setup` (if provided) runs per-instance with the prop signals.
 *  - Nothing is rendered into `container` directly — instances
 *    appear when authors place `<the-tag>` in a parent template.
 *
 * @param template The `.html.set` file contents.
 * @param setup Optional default-export from the paired `.ts` file.
 * @param opts Mount target and runtime context.
 * @returns A handle with a `dispose` callback.
 *
 * @example
 * ```ts
 * import { hydrate } from '@ship-set/dom'
 *
 * const handle = hydrate(
 *   '<state name="count" value="0" /><button on:click={() => count.set(count() + 1)}>{count()}</button>',
 *   undefined,
 *   { container: document.getElementById('app')! },
 * )
 * // later
 * handle.dispose()
 * ```
 */
export function hydrate(
  template: string,
  setup?: () => unknown,
  opts?: HydrateOptions,
): HydrateHandle {
  if (!opts || !opts.container) {
    throw new Error('[ship-set-dom] hydrate: opts.container is required')
  }
  const parsed = parseSetSource(template)
  const route = opts.route ?? {}
  const ship = opts.ship
  const navigate = opts.navigate ?? (() => {})

  // Component path — register the custom element and stop. Container
  // gets nothing; instances are placed by parent templates.
  if (parsed.component) {
    const bodyInstrs = compileTopLevel(parsed.templateNodes)
    registerComponent(
      parsed.component,
      bodyInstrs,
      opts.container.ownerDocument,
      setup,
    )
    return { dispose: () => {} }
  }

  // View path — build scope, render into container.
  let scope: Scope = { ship, route, navigate }
  for (const decl of parsed.data) {
    scope[decl.name] = mountData(decl, ship, route)
  }
  for (const decl of parsed.state) {
    scope[decl.name] = mountState(decl, scope)
  }
  if (setup) {
    const ctx = { ship, route, navigate }
    const ret = (setup as (ctx: typeof ctx) => unknown)(ctx)
    if (ret && typeof ret === 'object') {
      scope = extendScope(scope, ret as Scope)
    }
  }

  const instrs = compileTopLevel(parsed.templateNodes)
  const r = render(instrs, scope, opts.container.ownerDocument)
  for (const n of r.nodes) opts.container.appendChild(n)
  const cleanup = r.mount()
  return {
    dispose: () => {
      for (const n of r.nodes) {
        try {
          n.parentNode?.removeChild(n)
        } catch {
          /* already detached */
        }
      }
      cleanup()
    },
  }
}

/**
 * Compile a flat list of top-level template nodes by injecting them
 * into a transient parent and reusing `compileChildren`.
 */
function compileTopLevel(
  nodes: Node[],
): ReturnType<typeof compileChildren> {
  if (nodes.length === 0) return []
  const owner = nodes[0]!.ownerDocument
  if (!owner) return []
  const fragment = owner.createElement('div')
  for (const n of nodes) fragment.appendChild(n)
  return compileChildren(fragment)
}
