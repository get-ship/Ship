// controller.ts — typed contract for the five-section controller shape.
//
// Per the "state is declarative-only" ADR, controllers no longer have
// a `state:` section — state is declared exclusively in the template
// via `<state name="X" value="..." />`. A controller has five
// sections: `derived`, `on`, `use`, `mount`, `unmount`. The state-
// shape generic is gone; mutation/read views are derived from the
// `Scope` generic, which is what the template declares.
//
// Hooks (`ControllerHook`) keep an internal state-bearing shape — they
// declare their own writable signals as part of their composition
// surface. Hook state is namespaced under the `use:` key and never
// flat-merges into the host scope.

import type { Signal } from './signal'
import type { ShipSDK } from './ship'
import type { RouteSignals } from './route'

// ---------------------------------------------------------------------------
// State-shape extraction over Scope
// ---------------------------------------------------------------------------

/**
 * Extract the state-key subset of `Scope`. A state key is a scope
 * identifier whose value is a writable `Signal<T>` — that's how the
 * template's `<state>` directive contributes to scope. Read-only
 * derived/data identifiers and non-signal values fall through.
 */
export type StateKeys<Scope> = {
  [K in keyof Scope]: Scope[K] extends Signal<unknown> ? K : never
}[keyof Scope]

/** The signal-typed value at scope key K, or never. */
export type SignalValueAt<Scope, K extends keyof Scope> = Scope[K] extends Signal<infer V>
  ? V
  : never

// ---------------------------------------------------------------------------
// Scope shapes
// ---------------------------------------------------------------------------

/**
 * Read-only access to a state key from declarative contexts (templates,
 * `derived` sections, controller hooks). Callable to read; no `.set`.
 */
export interface ReadonlyView<T> {
  /** Read the current value (and track in any active reactive scope). */
  (): T
}

/**
 * The "this view's state, as seen from declarative contexts."
 *
 * Each state key (i.e. each scope key whose value is a `Signal`) is
 * exposed as a callable getter, never as a writable proxy. Used in
 * `derived` section signatures and as a constraint on what templates
 * can read.
 */
export type ReadonlyState<Scope> = {
  readonly [K in StateKeys<Scope>]: ReadonlyView<SignalValueAt<Scope, K>>
}

/**
 * The "this view's state, as seen from imperative contexts" (handlers,
 * `mount`, `unmount`).
 *
 * Each state key is a plain assignable property: read returns the
 * current value, write replaces it. The runtime translates the
 * assignment into a `signal.set()` call.
 */
export type MutableState<Scope> = {
  [K in StateKeys<Scope>]: SignalValueAt<Scope, K>
}

/**
 * The shared view scope, as destructured by each controller section.
 *
 * Includes `ship`, the reactive `route` proxy, the `<data>`-declared
 * signals from the template, the `<state>`-declared writable signals
 * (read-only here), and the controller-hook contributions.
 *
 * `route` is a `RouteSignals` whose `params.<name>` and
 * `query.<name>` are themselves signals — reading them inside a
 * `derived` body or a `<data live>` source thunk subscribes the
 * surrounding scope, so URL changes re-fire the dependent
 * computation.
 */
export type ViewScope<Scope = object> = Scope & {
  ship: ShipSDK
  route: RouteSignals
}

// ---------------------------------------------------------------------------
// Section types
// ---------------------------------------------------------------------------

/** A section-init can be a static value or a factory that receives scope. */
export type SectionInit<V, Scope> = V | ((scope: Scope) => V)

/**
 * `derived` section signature. Each key returns a value; the runtime
 * memoizes the result and tracks the function's reactive reads.
 * State here is read-only.
 */
export type DerivedSection<Scope> = Record<
  string,
  (scope: ViewScope<Scope>) => unknown
>

/**
 * `on` section signature. Each handler receives a writable state
 * mutator as its first argument and any handler-specific args after.
 *
 * Handlers may also accept the broader scope (with state attached) as
 * their first argument when they need to read other signals or call
 * `ship.*` directly. The runtime resolver discriminates by the first
 * parameter's shape.
 */
export type OnSection<Scope> = Record<
  string,
  // `any[]` (not `unknown[]`) is deliberate. Handlers declare typed
  // args; under contravariant param checks they aren't assignable to
  // a `(...args: unknown[])` constraint, so we widen to `any` so
  // authors can name argument types without an intermediate cast.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  | ((state: MutableState<Scope>, ...args: any[]) => void | Promise<void>)
  | ((
      scope: ViewScope<Scope> & { state: MutableState<Scope> },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      ...args: any[]
    ) => void | Promise<void>)
>

/**
 * `use` section signature. Each entry is a controller hook (see
 * `ControllerHook`). Results are namespaced by key in the resolved
 * view scope.
 */
export type UseSection<Scope> = Record<string, ControllerHook>

/**
 * `mount` lifecycle. Runs once after the rendered DOM is attached and
 * all signals/handlers/hooks are wired. Receives a writable state
 * mutator and the owning element. May return a cleanup function that
 * is called on disconnect (equivalent to pushing to cleanups).
 */
export type MountSection<Scope> = (state: MutableState<Scope>, el: HTMLElement) => (() => void) | void

/**
 * `unmount` lifecycle. Runs once on disconnect; tear down whatever
 * `mount` set up. Receives a writable state mutator.
 */
export type UnmountSection<Scope> = (state: MutableState<Scope>) => void

// ---------------------------------------------------------------------------
// The controller surface
// ---------------------------------------------------------------------------

/**
 * The full shape of a controller: five optional sections, each with a
 * static or factory form where applicable.
 *
 * State is **not** a controller section — declare it in the template
 * via `<state name="X" value="..." />`. Computed-initial cases move
 * to `mount` (`state.X = crypto.randomUUID()`).
 *
 * @typeParam Scope Template-declared scope (`<data>`, `<state>`,
 *   `<use-* as=>`) plus `use:` hook contributions.
 */
export interface ControllerShape<Scope = object> {
  /**
   * Factory that produces typed scope contributions: signals,
   * derived, hooks, shared-provider/consumer wrappers. The returned
   * object is merged into the template's scope. Runs before any
   * other section so derived/on/use can reference its entries.
   *
   * Replaces the historical `state:` + `derived:` + `use:` split for
   * tier-2+ controllers. Author writes idiomatic TypeScript inside
   * the factory; types flow through inference.
   */
  scope?: (init: ViewScope<Scope>, host?: Element) => Record<string, unknown>

  /** Pure memos. Each key is a function `(scope) => T`. */
  derived?: DerivedSection<Scope>

  /**
   * Event handlers. The **only** section in a controller that can
   * mutate state.
   */
  on?: OnSection<Scope>

  /**
   * Composition of controller hooks. Static or factory form; factory
   * form receives the merged scope.
   */
  use?: SectionInit<UseSection<Scope>, ViewScope<Scope>>

  /** Runs once after mount; receives writable state. */
  mount?: MountSection<Scope>

  /** Runs once on disconnect; receives writable state. */
  unmount?: UnmountSection<Scope>
}

/**
 * The two valid forms of a controller export — an object (tier 2) or
 * a function (tier 3). The function form's argument is `{ ship }`
 * only — template scope is not available at controller-instantiation
 * time, only inside section factories.
 */
export type ControllerDef<Scope = object> =
  | ControllerShape<Scope>
  | ((ctx: { ship: ShipSDK }) => ControllerShape<Scope>)

// `controller(def)` runtime helper lives in `controller-resolver.ts`.

// ---------------------------------------------------------------------------
// Controller hooks
// ---------------------------------------------------------------------------

/**
 * Authoring contract for a controller hook.
 *
 * Hooks are composable bundles; unlike controllers, they **do** declare
 * their own `state:` section because hook-local state is what makes
 * them coherent slices (`useFilteredList` owns its `query`,
 * `useSelection` owns its `selectedId`). Hook state is namespaced
 * under the `use:` key in the consuming controller — there is no
 * automatic flat-merge — so two hooks both declaring `state.query`
 * coexist as `<keyA>.query` and `<keyB>.query`.
 *
 * @typeParam S Hook-internal state shape.
 * @typeParam Scope Extra scope (usually `object` for hooks).
 */
export interface ControllerHook<
  S extends object = object,
  Scope = object,
> {
  /**
   * Hook-internal state. Static initial values; the runtime constructs
   * a `Signal<T>` per key.
   */
  state?: S | ((scope: { ship: ShipSDK } & Scope) => S)
  /** Pure memos over the merged hook scope (state + outer scope). */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  derived?: Record<string, (scope: any) => unknown>
  /** Event handlers; only section that mutates hook state. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  on?: Record<string, (state: any, ...args: any[]) => void | Promise<void>>
  /** Hook composition (a hook may itself compose other hooks). */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  use?: Record<string, ControllerHook> | ((scope: any) => Record<string, ControllerHook>)
  /** Runs once after mount; receives writable hook state. */
  mount?: (state: S, el: HTMLElement) => void
  /** Runs once on disconnect; receives writable hook state. */
  unmount?: (state: S) => void
}

// Template-hook contracts (`TemplateHook`, `TemplateHookContext`,
// `TemplateHookAttrKind`) live in `./template-hook.ts`.
export type {
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
} from './template-hook'
