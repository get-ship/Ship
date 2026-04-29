// view() — declares a view setup function.
//
// The Phase-3 shipd integration mounts a view by calling the setup
// function with a `ViewContext` (route params, ship SDK handle,
// navigate). The setup returns a bindings object the template runtime
// exposes to the page. In Phase 1 this is a thin wrapper around the
// shape — types + a runner — so authors can hand-write a setup file
// and have it compose with the rest of the runtime.

/**
 * The Ship SDK handle exposed to view setups.
 *
 * TODO(phase-5): tighten this to the generated SDK manifest type so
 * `ctx.ship.<domain>.<op>` autocompletes against the project's actual
 * source schemas. Phase 1 leaves the type as `unknown` so the runtime
 * doesn't depend on the SDK package shape.
 */
export type ShipSDK = unknown

/**
 * The context passed into every view setup function. Route params come
 * from the matched URL, `ship` is the live SDK handle, and `navigate`
 * triggers a client-side navigation.
 */
export interface ViewContext<RouteParams = Record<string, string>> {
  /** Route parameters captured from the URL pattern. */
  route: RouteParams
  /** Live SDK handle for issuing queries and subscriptions. */
  ship: ShipSDK
  /** Navigate to another path; replaces the current view. */
  navigate: (path: string) => void
}

/**
 * The shape of a view setup function. Receives a `ViewContext` and
 * returns whatever bindings the template needs (signals, callbacks,
 * derived values).
 */
export type ViewSetup<
  RouteParams = Record<string, string>,
  Bindings = unknown,
> = (ctx: ViewContext<RouteParams>) => Bindings

/**
 * The compiled form of a `view()` declaration — a thin wrapper that
 * carries the setup function. The shipd integration calls
 * `def.setup(ctx)` at view mount and exposes the returned bindings to
 * the template runtime.
 */
export interface ViewDef<Bindings = unknown> {
  setup: ViewSetup<Record<string, string>, Bindings>
}

/**
 * Declare a view's setup. Returns a `ViewDef` that the runtime
 * (Phase 3 shipd integration) invokes at mount time with a
 * `ViewContext`. The bindings the setup returns are exposed to the
 * template — signals get auto-tracked, callbacks fire on events, and
 * derived values recompute as their deps change.
 *
 * Phase 1 is intentionally thin: this lets authors hand-write a
 * `view.ts` file today and have its types line up with the eventual
 * compiled emit. No template runtime is invoked here.
 *
 * @param setup The setup function — receives the view context, returns
 *   bindings.
 * @returns A `ViewDef` carrying `setup`.
 *
 * @example
 * ```ts
 * export default view((ctx) => {
 *   const id = ctx.route.id
 *   const agent = live(() => ctx.ship.agents.get(id))
 *   return { agent, back: () => ctx.navigate('/') }
 * })
 * ```
 *
 * @see live for live SDK signals; state for ephemeral local state.
 */
export function view<Bindings>(
  setup: ViewSetup<Record<string, string>, Bindings>,
): ViewDef<Bindings> {
  return { setup }
}
