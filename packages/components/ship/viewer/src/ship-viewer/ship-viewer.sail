<controller>
// ship-viewer — the content-renderer dispatcher controller (tier-2, glue-only).
// Builds the pure `createMimeRegistry()` engine from @ship/sail-media, SEEDS it
// with this router's own default kind→TAG table (`defaultEntries()` — the
// dispatch decision lives here, not in the engine), reads the `prop:src`
// identity binding AND the `content-type` / `filename` / `text` SSR fallback
// props, and exposes the deriveds the template `{#match kind()}` branches on.
// State (the resolved descriptor + which renderer wins) lives in the engine;
// this controller is wiring.
//
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. The active child's
// bubbling+composed `sui-viewer-load`/`-error` (the ViewerRenderer event
// contract) are re-emitted from the host (deduped) so ONE listener on
// `<ship-viewer>` catches any renderer.
//
// Authority: build-spec §"The <ship-viewer> dispatcher view", §"Controller
// shape", behavioral specs #1, #8, #9.

import { controller, derived, emit, signal } from '@ship/sail-runtime'
import { createMimeRegistry, extensionOf, type ViewerSource } from '@ship/sail-media/engine'
import { defaultEntries } from '../engine/registry'
import type { Scope } from './ship-viewer.sail'

/** A lang hint for the code arm, derived from the filename extension. */
function langOf(filename: string): string {
  return extensionOf(filename)
}

export default controller({
  scope: ({ contentType, filename, text }: Scope) => {
    // Seed the engine with this router's default kind→TAG table. The engine
    // resolves against whatever entries it is handed — the dispatch decision
    // (which renderer a kind routes to) is owned here, by the router.
    const registry = createMimeRegistry({ entries: defaultEntries() })
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
    // The pdf arm passes a STRING `src` to <ship-pdf> (its declared prop), not
    // the ViewerSource identity: a resolved `url` is used verbatim, else a
    // `blob:` URL is minted from the source's resolved `bytes`.
    const pdfSrc = derived<string>(() => {
      const s = resolvedSource()
      if (!s) return ''
      if (s.url) return s.url
      if (s.bytes) {
        const part = (s.bytes instanceof Uint8Array ? s.bytes : new Uint8Array(s.bytes)) as BlobPart
        return URL.createObjectURL(new Blob([part], { type: 'application/pdf' }))
      }
      return ''
    })
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
      pdfSrc,
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
    // single `on:sui-viewer-*` on <ship-viewer> catches any renderer. The guard
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
    sr.addEventListener('sail-viewer-load', onLoad)
    sr.addEventListener('sail-viewer-error', onError)

    return () => {
      sr.removeEventListener('sail-viewer-load', onLoad)
      sr.removeEventListener('sail-viewer-error', onError)
    }
  },
})
</controller>

<!--
  <ship-viewer> — the content-renderer DISPATCHER / ROUTER (Backbone #1).

  Reads a resolved source (via `src` identity OR the `content-type` /
  `filename` / `text` SSR fallback props), resolves the renderer through the
  pure `createMimeRegistry()` engine (from @ship/sail-media, seeded with this
  router's default kind→TAG table), and renders the matching child via
  `<match on={kind()}>`. Adding a content type is ONE registry entry — never an
  edit here. The dispatcher is thin: state (the resolved descriptor + which
  renderer wins) lives in the engine; the controller is glue.

  Leaf renderers are the @ship/sail-media tags (sui-media-image / -av / -csv);
  the markdown / code / json / diff arms compose @ship/sail-primitives targets.

  Read-only, source-in-never-fetch: no `fetch`/`EventSource`/`ship.*`. The host
  resolves the bytes/text; the dispatcher only inspects + routes.

  Props (build-spec §Props):
    content-type  string  reflect  — SSR/declarative type hint (src.contentType supersedes)
    filename      string  reflect  — extension fallback for detection
    text          string           — text-content fallback (src.text supersedes)
    src           src identity only — NOT a <!-- <prop> --> (the live ViewerSource/Blob/url)

  Events (re-emitted from the active child, ViewerRenderer contract):
    sui-viewer-load            — first successful render
    sui-viewer-error {message} — parse/decode failure (one host listener on <ship-viewer> catches any renderer)

  Slots: toolbar (above the renderer), download (forwarded into the unsupported
  empty-state's action slot), empty (override the default empty-state).
  A11y: host carries role=region; [data-viewer-loading] reflects active load.
-->

<!-- <prop name="content-type" type="string" default="" reflect /> -->
<!-- <prop name="filename" type="string" default="" reflect /> -->
<!-- <prop name="text" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<div><slot name="toolbar"></slot></div>

<match on={kind()}>
  <case of={'pdf'}>
    <!-- The pdf renderer is `<ship-pdf>` — the @ship/pdf SHIP-tier component
         wrapping a vendored pdf.js engine. The dispatcher resolves a real,
         defined element here (no fallback/blank): `pdfSrc()` is the source's
         resolved URL (or a blob: URL minted from its bytes). -->
    <ship-pdf src={pdfSrc()}></ship-pdf>
  </case><case of={'image'}>    <sail-media-image src={source()}></sail-media-image>
  </case><case of={'media'}>    <sail-media-av src={source()}></sail-media-av>
  </case><case of={'tabular'}>  <sail-media-csv src={source()}></sail-media-csv>
  </case><case of={'markdown'}> <sail-markdown value={textValue()}></sail-markdown>
  </case><case of={'code'}>     <code-block block lang={lang()}>{textValue()}</code-block>
  </case><case of={'json'}>     <sail-json-view data={textValue()}></sail-json-view>
  </case><case of={'diff'}>     <sail-diff value={textValue()} from={from()} to={to()}></sail-diff>
  </case><default>
    <empty-state variant="placeholder">
      <span slot="title">No preview</span>
      {unsupportedMsg()}
      <slot name="download" slot="action"></slot>
    </empty-state>
</default></match>
