// mount() — F1 unified entry point.
//
// Replaces attach() (legacy, in @ship/ui-dom) with a payload-driven
// flow. Hosts (shell, view-host static, embed surfaces, tests) pass
// either a fetched ViewPayload or a name string; mount injects the
// SSR HTML, builds signals from payload.scope, imports the compiled
// view module, runs setup() + hydrate() + lifecycle.mount, opens any
// live SSE channel, and returns a typed MountHandle.
//
// Spec: .artifacts/design-f1-mount-unification.md
//
// IMPORTANT: this module owns the runtime-side half of the SUI mount
// contract. Anything that knows about HTML structure (importmap, boot
// scripts, doctype) lives OUTSIDE this file — in the host page.

import { signal, type Signal } from './signal'
import type { LiveSignal, LiveState } from './live'
import { route as makeRoute, type RouteSignals } from './route'
import { createScope, withScope } from './effect-scope'

/** View payload as returned by `GET /_sui/views/<name>[/<sub>]`. Keep
 *  in sync with `apps/shipd/src/views/sui_view_payload.rs::ViewPayload`. */
export interface ViewPayload<Scope = Record<string, unknown>> {
  name: string
  sub?: string
  /** SSR-rendered template body. No doctype, no script island. */
  html: string
  /** Concatenated stylesheet (parent + sub when applicable). May be empty. */
  css: string
  /** Initial scope values — same shape attach() reads from the JSON island. */
  scope: Scope
  /** URL of the compiled JS module. */
  moduleUrl: string
  /** `<sui-*>` island tags used in the template. */
  islands?: string[]
}

export interface MountOptions {
  ship?: unknown
  route?: RouteSignals
  navigate?: (path: string) => void
  /** HMR — overlay these values onto fresh signals after re-mount. */
  restoreScope?: Record<string, unknown>
  /** Where to inject the payload's css. Default: a `<style>` adjacent
   *  to `target`. `'none'` skips injection (host owns css). */
  cssScope?: 'adjacent' | 'document-head' | 'none'
  /** Pre-resolved compiled module — skips `import(payload.moduleUrl)`.
   *  Useful for tests, prefetching, and code-splitting decisions where
   *  the host already has the module in hand. */
  module?: { default: { setup: (ctx: unknown) => Record<string, unknown> } }
}

export interface MountHandle<Scope = Record<string, unknown>> {
  /** Tear down: invoke unmount, run cleanups, close SSE, remove style el. */
  dispose(): void
  /** Live signal map (writable per declared `<state>` keys). F2 types this. */
  scope: Scope
  /** The element passed to mount(). DOM stays in place after dispose. */
  element: HTMLElement
  /** HMR: dispose, reuse target, mount the new payload. Preserves
   *  signal values for keys present in both payloads' scope. */
  reload(next: ViewPayload): Promise<void>
}

interface SetupContext {
  ship?: unknown
  route?: RouteSignals
  navigate?: (path: string) => void
  scope?: Record<string, unknown>
  stateKeys?: ReadonlySet<string>
}

interface CompiledModule {
  default: { setup: (ctx: SetupContext) => Record<string, unknown> }
}

interface Lifecycle {
  state: Record<string, unknown>
  mount?: (state: Record<string, unknown>, el: HTMLElement) => void
  unmount?: (state: Record<string, unknown>) => void
}
const LIFECYCLE_KEY = '__shipSetLifecycle'

/**
 * Mount a view into a DOM target.
 *
 * @param target Element to receive the rendered view. Existing
 *               innerHTML is replaced with payload.html.
 * @param payloadOrName  A fetched ViewPayload, or a `name[/sub]`
 *               string the runtime fetches from `/_sui/views/...`.
 * @param opts   ship / route / navigate / restoreScope / cssScope.
 * @returns      MountHandle — call `dispose()` when done.
 */
export async function mount<Scope = Record<string, unknown>>(
  target: HTMLElement,
  payloadOrName: ViewPayload<Scope> | string,
  opts: MountOptions = {},
): Promise<MountHandle<Scope>> {
  if (!target) throw new Error('[sui/mount] target is required')

  const payload =
    typeof payloadOrName === 'string'
      ? await fetchPayload(payloadOrName)
      : payloadOrName

  // 1. Inject SSR HTML.
  target.innerHTML = payload.html

  // 2. Inject CSS.
  const styleEl = injectCss(target, payload.css, opts.cssScope ?? 'adjacent')

  // 3. Build signals from payload.scope.
  const built = buildScopeFromPayload(payload.scope as Record<string, unknown>, opts.restoreScope)

  // 4. Import the compiled module (or use pre-resolved one).
  const mod = (opts.module ?? (await import(/* @vite-ignore */ payload.moduleUrl))) as CompiledModule
  if (!mod?.default || typeof mod.default.setup !== 'function') {
    throw new Error(
      `[sui/mount] module ${payload.moduleUrl} missing default.setup()`,
    )
  }

  // 5. Build setup context + run setup(ctx).
  const route = opts.route ?? makeRoute()
  const navigate = opts.navigate ?? noop
  const baseScope: Record<string, unknown> = {
    ship: opts.ship,
    route,
    navigate,
    ...built.scope,
  }
  const setupCtx: SetupContext = {
    ship: opts.ship,
    route,
    navigate,
    scope: { ...baseScope },
    stateKeys: built.stateKeys,
  }
  // Run setup() inside an effect scope so any `live()` / `data()` /
  // hook subscriptions allocated during setup register their
  // cleanup with this mount. The scope is disposed by `dispose()`
  // below — without it, EventSource subscriptions opened by
  // `<data live>` views leaked across every navigation and tabs
  // eventually OOM'd.
  const scope = createScope()
  const setupRet = withScope(scope, () =>
    mod.default.setup(setupCtx) as Record<string, unknown> | undefined,
  )

  // 6. setup()'s return shadows scope keys with duplicate signals; the
  //    SSR-built signals are the source of truth (writable proxy targets
  //    init.scope, SSE patches them). Restore them.
  const finalScope: Record<string, unknown> = { ...baseScope }
  if (setupRet) Object.assign(finalScope, setupRet)
  for (const k of built.stateKeys) {
    if (built.scope[k]) finalScope[k] = built.scope[k]
  }
  for (const k of built.liveKeys) {
    if (built.scope[k]) finalScope[k] = built.scope[k]
  }

  // 7. Run hydrate(root, ctx) if the compiled module exposed it.
  const cleanups: Array<() => void> = []
  // Hoisted ahead of handle declaration so the async-mount branch
  // can guard against pushing a cleanup into a mount that already
  // disposed before its async mount() resolved.
  let disposed = false
  const hydrate = setupRet?.['hydrate']
  if (typeof hydrate === 'function') {
    try {
      const dispose = (hydrate as (r: HTMLElement, c: SetupContext) => unknown)(
        target,
        setupCtx,
      )
      if (typeof dispose === 'function') cleanups.push(dispose as () => void)
    } catch (err) {
      console.error('[sui/mount] hydrate threw:', err)
    }
  }

  // 8. Lifecycle mount.
  const lifecycle = extractLifecycle(setupRet)
  if (lifecycle?.mount) {
    try {
      const result = lifecycle.mount(lifecycle.state, target) as unknown
      // Controller mount may return any of:
      //   - void                — nothing to clean up
      //   - () => void          — disposer; idiomatic Solid/Svelte shape
      //   - Promise<void | () => void> — async mount with optional disposer
      //
      // The pre-fix branch only checked for Promise and surfaced
      // rejections, silently DROPPING the disposer return. 61 of 69
      // production controllers rely on `return () => …` for cleanup
      // (every MutationObserver, ResizeObserver, setInterval,
      // addEventListener tied to a host), so every navigation away
      // from a mounted view leaked listeners + observers + timers.
      // Honor the disposer in both sync and async cases.
      if (typeof result === 'function') {
        cleanups.push(result as () => void)
      } else if (
        result &&
        typeof result === 'object' &&
        typeof (result as Promise<unknown>).then === 'function'
      ) {
        ;(result as Promise<unknown>).then(
          (asyncResult) => {
            if (typeof asyncResult !== 'function') return
            const disposer = asyncResult as () => void
            if (disposed) {
              // The view was already disposed before the async mount
              // resolved — run the disposer immediately so its
              // resources are released, rather than letting it leak.
              try { disposer() } catch (err) {
                console.error('[sui/mount] async cleanup threw:', err)
              }
            } else {
              cleanups.push(disposer)
            }
          },
          (err) => {
            console.error('[sui/mount] controller mount() promise rejected:', err)
          },
        )
      }
    } catch (err) {
      console.error('[sui/mount] controller mount() threw:', err)
    }
  }

  // 9. Open live SSE channel if the payload declared one.
  const subscribeUrl = pluckSubscribe(payload.scope as Record<string, unknown>)
  let liveSource: EventSource | null = null
  if (subscribeUrl && built.liveKeys.size > 0) {
    liveSource = openLiveChannel(
      subscribeUrl,
      built.scope as Record<string, Signal<unknown>>,
      built.liveKeys,
      target,
    )
    cleanups.push(() => liveSource?.close())
  }

  // 10. Build the handle.
  const handle: MountHandle<Scope> = {
    element: target,
    scope: built.scope as unknown as Scope,
    dispose() {
      if (disposed) return
      disposed = true
      if (lifecycle?.unmount) {
        try {
          lifecycle.unmount(lifecycle.state)
        } catch (err) {
          console.error('[sui/mount] controller unmount() threw:', err)
        }
      }
      for (const fn of cleanups) {
        try {
          fn()
        } catch (err) {
          console.error('[sui/mount] cleanup threw:', err)
        }
      }
      // Dispose the scope: closes any EventSource subscription opened
      // by `<data live>` / `live()` during setup, plus any other
      // resource registered via `onCleanup`.
      scope.dispose()
      styleEl?.remove()
    },
    async reload(next: ViewPayload) {
      // Snapshot current signal values for the keys present in `next.scope`.
      const restored: Record<string, unknown> = {}
      const nextScope = (next.scope ?? {}) as Record<string, unknown>
      for (const k of Object.keys(nextScope)) {
        const sig = (built.scope as Record<string, Signal<unknown>>)[k]
        if (sig) restored[k] = sig()
      }
      handle.dispose()
      const remounted = (await mount(target, next as ViewPayload, {
        ...opts,
        restoreScope: restored,
      })) as MountHandle<Scope>
      // Adopt the new mount's internals into this handle so the caller's
      // reference stays stable across reloads.
      handle.scope = remounted.scope
      handle.dispose = remounted.dispose
      handle.reload = remounted.reload
    },
  }
  return handle
}

// ── Internals ─────────────────────────────────────────────────────────

async function fetchPayload(nameOrPath: string): Promise<ViewPayload> {
  const path = nameOrPath.startsWith('/_sui/views/')
    ? nameOrPath
    : `/_sui/views/${nameOrPath}`
  const res = await fetch(path, { credentials: 'same-origin' })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(`[sui/mount] fetch ${path} ${res.status}: ${body.slice(0, 200)}`)
  }
  return (await res.json()) as ViewPayload
}

interface BuiltScope {
  scope: Record<string, Signal<unknown>>
  stateKeys: Set<string>
  liveKeys: Set<string>
}

function buildScopeFromPayload(
  payloadScope: Record<string, unknown>,
  restore?: Record<string, unknown>,
): BuiltScope {
  const scope: Record<string, Signal<unknown>> = {}
  const stateKeys = new Set<string>()
  const liveKeys = new Set<string>()
  for (const [k, v] of Object.entries(payloadScope)) {
    if (k === '__subscribe__') continue
    if (isLiveStateEnvelope(v)) {
      const init = restore && k in restore ? restore[k] : v
      scope[k] = signal<LiveState<unknown>>(init as LiveState<unknown>) as unknown as LiveSignal<unknown> as unknown as Signal<unknown>
      liveKeys.add(k)
    } else {
      const init = restore && k in restore ? restore[k] : v
      scope[k] = signal<unknown>(init)
      stateKeys.add(k)
    }
  }
  return { scope, stateKeys, liveKeys }
}

function pluckSubscribe(payloadScope: Record<string, unknown>): string | undefined {
  const v = payloadScope['__subscribe__']
  return typeof v === 'string' ? v : undefined
}

function isLiveStateEnvelope(v: unknown): v is LiveState<unknown> {
  if (!v || typeof v !== 'object') return false
  const k = (v as { kind?: unknown }).kind
  return k === 'loading' || k === 'ok' || k === 'error'
}

function injectCss(
  target: HTMLElement,
  css: string,
  scope: Exclude<MountOptions['cssScope'], undefined>,
): HTMLStyleElement | null {
  if (!css || scope === 'none') return null
  const doc = target.ownerDocument ?? document
  const style = doc.createElement('style')
  style.setAttribute('data-sui-css', '1')
  style.textContent = css
  if (scope === 'document-head') {
    doc.head.appendChild(style)
  } else {
    // 'adjacent' — sibling before target so cascade order matches the
    // standalone (`<link>` in <head>) flow.
    target.parentNode?.insertBefore(style, target)
  }
  return style
}

function extractLifecycle(ret: unknown): Lifecycle | undefined {
  if (!ret || typeof ret !== 'object') return undefined
  const lc = (ret as Record<string, unknown>)[LIFECYCLE_KEY]
  if (lc && typeof lc === 'object') return lc as Lifecycle
  return undefined
}

function noop(_p: string): void {
  /* default navigate */
}

// ── Live SSE channel (parallel to ui-dom's attach/live-channel) ────────
//
// Mount owns its own copy because @ship/ui-runtime cannot import
// @ship/ui-dom (would invert the layering). Once F1.6 deletes attach(),
// the ui-dom copy goes too.

interface PatchSignalsPayload {
  name: string
  value: LiveState<unknown>
}

function openLiveChannel(
  url: string,
  scope: Record<string, Signal<unknown>>,
  liveKeys: Set<string>,
  _root: HTMLElement,
): EventSource {
  const es = new EventSource(url)
  es.addEventListener('patch-signals', (event: MessageEvent) => {
    let payload: PatchSignalsPayload
    try {
      payload = JSON.parse(event.data as string) as PatchSignalsPayload
    } catch {
      console.warn('[sui/mount] patch-signals: bad payload', event.data)
      return
    }
    const { name, value } = payload
    if (!liveKeys.has(name)) return
    const sig = scope[name]
    if (!sig) return
    ;(sig as Signal<LiveState<unknown>>).set(value)
  })
  // patch-elements deferred until cleanup commit — agent-push integration
  // isn't on F1's critical path, mount keeps backward compat by accepting
  // the event but ignoring the body until applyPatchElements is moved
  // into ui-runtime.
  es.addEventListener('error', () => {
    // Built-in EventSource reconnect handles transient drops.
  })
  return es
}
