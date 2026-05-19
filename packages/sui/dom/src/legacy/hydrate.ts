// Public hydrate entry point. Parses a `.sui` source, runs the
// optional paired setup, builds the scope from <data> / <state> /
// setup return, compiles the template body, and inserts the rendered
// DOM into `opts.container`. Returns a disposer that tears the whole
// tree down: every signal subscriber, every event listener, every
// DOM node added by the call.

import { parseSetSource, type UseDecl } from './parse'
import { compileChildren } from '../template/compile'
import { extendScope, render, type Scope } from '../template/render'
import { mountData } from './directives/data'
import { mountState } from './directives/state'
import { registerComponent } from './directives/component'
import { getTemplateHook } from '@ship/sui-runtime'
// R2: route lives at the `/route` subpath now; the barrel doesn't
// re-export it. Legacy hydrate imports it directly so this file
// keeps working without forcing the barrel back to its old shape.
import { route as makeRoute, type RouteSignals } from '@ship/sui-runtime/route'
// Built-in `<use-*>` hooks register on barrel import. The legacy
// hydrate flow is the only path that resolves `<use-*>` against the
// registry, so the legacy bundle owns triggering registration — the
// production runtime entry no longer ships hook code.
import '@ship/sui-runtime/template-hooks'

/** Options accepted by `hydrate`. */
export interface HydrateOptions {
  /** Where to mount the rendered nodes. */
  container: HTMLElement
  /** Reactive URL proxy. Defaults to a fresh `route()` from `window.location`. */
  route?: RouteSignals
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
 * @param template The `.sui` file contents.
 * @param setup Optional default-export from the paired `.ts` file.
 * @param opts Mount target and runtime context.
 * @returns A handle with a `dispose` callback.
 *
 * @example
 * ```ts
 * import { hydrate } from '@ship/sui-dom/legacy'
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
  const route = opts.route ?? makeRoute()
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
  //
  // Order: state mounts FIRST so that reactive `<data>` source thunks
  // (which receive `$scope` as their third argument) can read state
  // signals at evaluation time. `live()` runs the thunk in a tracked
  // reactive scope; reading a state signal subscribes the channel,
  // and a state change triggers the prior SSE subscription to dispose
  // + a new one to open with the updated args.
  let scope: Scope = { ship, route, navigate }
  const stateKeys = new Set<string>()
  for (const decl of parsed.state) {
    scope[decl.name] = mountState(decl, scope)
    stateKeys.add(decl.name)
  }
  for (const decl of parsed.data) {
    scope[decl.name] = mountData(decl, ship, route, scope)
  }
  // Lifecycle pulled from a controller-shaped setup return; populated
  // below if `setup` is provided and it returned the LIFECYCLE_KEY
  // brand. `mount` runs after the render completes; `unmount` runs on
  // dispose before the rendered tree is torn down.
  const LIFECYCLE_KEY = '__shipSetLifecycle'
  type ControllerLifecycle = {
    state: Record<string, unknown>
    mount?: (state: Record<string, unknown>, el: HTMLElement) => void
    unmount?: (state: Record<string, unknown>) => void
  }
  let lifecycle: ControllerLifecycle | undefined

  if (setup) {
    // Pass the current scope (data + state signals) so a controller
    // resolver can read template-declared signals from its `derived`
    // and `use:` factories. Existing `view()` setups that only read
    // ship/route/navigate are unaffected by the extra field.
    interface SetupCtx {
      ship: unknown
      route: RouteSignals
      navigate: (path: string) => void
      scope: Record<string, unknown>
      stateKeys: ReadonlySet<string>
    }
    const ctx: SetupCtx = {
      ship,
      route,
      navigate,
      scope: { ...scope },
      stateKeys,
    }
    const ret = (setup as (ctx: SetupCtx) => unknown)(ctx)
    if (ret && typeof ret === 'object') {
      scope = extendScope(scope, ret as Scope)
      const lc = (ret as Record<string, unknown>)[LIFECYCLE_KEY]
      if (lc && typeof lc === 'object') {
        lifecycle = lc as ControllerLifecycle
      }
    }
  }

  // Invoke `<use-*>` template hooks AFTER setup (controller bindings
  // are now in scope, so action="X" lookups work) and BEFORE render
  // (so any scope contributions via `as=X` are visible to the
  // template). Each hook may register a cleanup that runs alongside
  // the framework's binding teardown on dispose.
  const hookCleanups: Array<() => void> = []
  for (const u of parsed.uses) {
    invokeUseHook(u, scope, opts.container, (cleanup) => {
      hookCleanups.push(cleanup)
    })
  }

  const instrs = compileTopLevel(parsed.templateNodes)
  const r = render(instrs, scope, opts.container.ownerDocument)
  for (const n of r.nodes) opts.container.appendChild(n)
  const cleanup = r.mount()

  // Run user mount() once the rendered nodes are in the container.
  // The user receives the writable state proxy + the container itself;
  // anything they push to a closure-captured cleanup list is the
  // unmount-time tear-down (separate from the framework's own bindings
  // cleanup). Errors here are caught so a buggy mount can't strand
  // the rendered tree without a path to dispose.
  if (lifecycle?.mount) {
    try {
      lifecycle.mount(lifecycle.state, opts.container)
    } catch (err) {
      console.error('[ship-set-dom] controller mount() threw:', err)
    }
  }

  return {
    dispose: () => {
      // User unmount runs first so it sees the rendered DOM and any
      // signals it wants to read still backing data.
      if (lifecycle?.unmount) {
        try {
          lifecycle.unmount(lifecycle.state)
        } catch (err) {
          console.error('[ship-set-dom] controller unmount() threw:', err)
        }
      }
      // Tear down template hooks after user unmount but before
      // removing the rendered nodes — hook cleanups (event listeners
      // on document, observers, etc.) outlive their owning element.
      for (const fn of hookCleanups) {
        try {
          fn()
        } catch (err) {
          console.error('[ship-set-dom] template hook cleanup threw:', err)
        }
      }
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
 * Invoke a single `<use-*>` template hook factory. Resolves attribute
 * values (literal / identifier / binding) against the provided scope,
 * forwards parsed children, and pushes any returned scope contribution
 * into the scope. Errors are caught and logged.
 */
function invokeUseHook(
  decl: UseDecl,
  scope: Scope,
  container: HTMLElement,
  registerCleanup: (fn: () => void) => void,
): void {
  const hook = getTemplateHook(decl.name)
  if (!hook) {
    console.warn(`[ship-set-dom] no template hook registered for <use-${decl.name}>`)
    return
  }

  // Resolve each attribute. `literal` → raw string. `identifier` /
  // `binding` → look up the named identifier in scope; pass through
  // the resolved value. Hooks that bind a controller `on.*` action by
  // name (e.g. `<use-kbd> <on action="search" />`) get a resolver
  // closure for that lookup.
  const resolvedAttrs: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(decl.attrs)) {
    if (v.kind === 'literal') {
      resolvedAttrs[k] = v.raw
    } else {
      // identifier or binding — resolve from scope; missing → undefined.
      resolvedAttrs[k] = scope[v.raw]
    }
  }

  // Hook-specific child handling: <use-kbd> consumes `<on key=
  // action= />` records as `KbdShortcut[]`. For other hooks, raw
  // children are also exposed (only some hooks read them).
  if (decl.name === 'kbd') {
    const shortcuts = decl.children
      .filter((c) => c.tag === 'on')
      .map((c) => ({ key: c.attrs.key ?? '', action: c.attrs.action ?? '' }))
      .filter((s) => s.key && s.action)
    resolvedAttrs.shortcuts = shortcuts
    // Bind a scope-resolver the kbd hook can use to look up actions.
    resolvedAttrs.resolve = (action: string): ((e: KeyboardEvent) => void) | undefined => {
      const v = scope[action]
      return typeof v === 'function' ? (v as (e: KeyboardEvent) => void) : undefined
    }
  }

  try {
    const cleanups: Array<() => void> = []
    const ctx = {
      attrs: resolvedAttrs,
      el: container,
      cleanup: (fn: () => void) => cleanups.push(fn),
    }
    const contributed = hook.factory(ctx)
    if (contributed !== undefined) {
      // The hook contributed a value; expose it under its `as=`
      // identifier (only `<use-shell as="X">` does this in v1).
      const asAttr = decl.attrs.as
      if (asAttr && asAttr.kind === 'literal') {
        scope[asAttr.raw] = contributed
      }
    }
    for (const c of cleanups) registerCleanup(c)
  } catch (err) {
    console.error(`[ship-set-dom] <use-${decl.name}> factory threw:`, err)
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
