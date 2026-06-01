// sui-viewer — the dispatcher controller (tier-2, glue-only). Builds the pure
// `createMimeRegistry()` engine, seeds it from the `prop:src` identity binding
// AND the `content-type` / `filename` / `text` SSR fallback props, and exposes
// the deriveds the template `{#match kind()}` branches on. State (the resolved
// descriptor + which renderer wins) lives in the engine; this controller is
// wiring.
//
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. The active child's
// bubbling+composed `sui-viewer-load`/`-error` are re-emitted from the host
// (deduped) so ONE listener on `<sui-viewer>` catches any renderer.
//
// Authority: build-spec §"The <sui-viewer> dispatcher view", §"Controller
// shape", behavioral specs #1, #8, #9.

import { controller, derived, emit, signal } from '@ship/sui-runtime'
import { extensionOf } from '../engine/detect'
import { createMimeRegistry } from '../engine/create-mime-registry'
import type { ViewerSource } from '../engine/types'
import type { Scope } from './sui-viewer.sui'

/** A lang hint for the code arm, derived from the filename extension. */
function langOf(filename: string): string {
  return extensionOf(filename)
}

export default controller<Scope>({
  scope: ({ contentType, filename, text }) => {
    const registry = createMimeRegistry()
    // prop:src identity — the live ViewerSource the consumer resolved. Assigned
    // to the host `src` property in mount; null until then.
    const srcSig = signal<ViewerSource | null>(null)

    // The effective source: prop:src identity supersedes the declarative SSR
    // props (content-type / filename / text). Recomputes when either changes.
    const resolvedSource = derived<ViewerSource | null>(() => {
      const live = srcSig()
      if (live != null) return live
      const ct = contentType()
      const fn = filename()
      const tx = text()
      if (!ct && !fn && !tx) return null
      return { contentType: ct, filename: fn, text: tx }
    })

    // Feed the engine. `descriptor`/`kind`/`renderer` are memoized deriveds on
    // the registry; reading them in the template drives the {#match}.
    const descriptor = derived(() => registry.resolve(resolvedSource() ?? {}))
    const kind = derived(() => descriptor().kind)

    const source = derived<ViewerSource | null>(() => resolvedSource())
    const textValue = derived<string>(() => resolvedSource()?.text ?? '')
    const lang = derived<string>(() => langOf(filename()))
    const from = derived<string>(() => resolvedSource()?.from ?? '')
    const to = derived<string>(() => resolvedSource()?.to ?? '')
    const unsupportedMsg = derived<string>(
      () => `No preview for ${descriptor().normalizedType || 'unknown type'}`,
    )

    return {
      srcSig,
      kind,
      source,
      textValue,
      lang,
      from,
      to,
      unsupportedMsg,
    }
  },

  mount(state, host) {
    // role=region a11y landmark (consumer-overridable).
    if (!host.hasAttribute('role')) host.setAttribute('role', 'region')

    // prop:src identity setter — a live JS reference, never a stringified
    // attribute. Routes into the scope signal so detection re-resolves.
    Object.defineProperty(host, 'src', {
      configurable: true,
      get: () => state.srcSig,
      set: (s: ViewerSource | null) => {
        state.srcSig = s
      },
    })

    // Re-emit the active child renderer's lifecycle events from the host so a
    // single `on:sui-viewer-*` on <sui-viewer> catches any renderer. The guard
    // `e.target === host` skips the host-originated re-emit so there is no echo
    // loop (the child event's target is the child; ours is the host).
    const reemit = (name: 'sui-viewer-load' | 'sui-viewer-error') => (e: Event) => {
      if (e.target === host) return
      emit(host, name, (e as CustomEvent).detail)
    }
    const onLoad = reemit('sui-viewer-load')
    const onError = reemit('sui-viewer-error')

    // Listen inside the shadow subtree for the bubbling child events. The child
    // emits bubbles+composed; the listener on the host's shadowRoot catches them
    // and we re-fire from the host itself.
    const sr = host.shadowRoot ?? host
    sr.addEventListener('sui-viewer-load', onLoad)
    sr.addEventListener('sui-viewer-error', onError)

    return () => {
      sr.removeEventListener('sui-viewer-load', onLoad)
      sr.removeEventListener('sui-viewer-error', onError)
    }
  },
})
