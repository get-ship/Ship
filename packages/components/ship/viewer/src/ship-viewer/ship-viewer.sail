<!--
  <ship-viewer> — the content-renderer DISPATCHER / ROUTER.

  Reads a resolved source (via `src` identity OR the `content-type` /
  `filename` / `text` SSR fallback props), resolves the renderer through the
  pure `createMimeRegistry()` engine (from @ship/sail-media, seeded with this
  router's default kind→TAG table), and renders the matching child.

  Leaf renderers are the @ship/sail-media tags; markdown / code / json / diff
  compose primitives; pdf is `<ship-pdf>`.

  Props:
    content-type  string  — SSR/declarative type hint (src.contentType supersedes)
    filename      string  — extension fallback for detection
    text          string  — text-content fallback (src.text supersedes)
    src           identity only — the live ViewerSource/Blob/url (not a prop)

  Events (re-emitted from the active child):
    sail-viewer-load
    sail-viewer-error {message}
-->
<scope>
  props: {
    'content-type' = ''
    filename = ''
    text = ''
  }
</scope>

<controller>
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. The active child's
// bubbling+composed viewer-load / viewer-error events are re-emitted from the
// host (deduped) so ONE listener on <ship-viewer> catches any renderer.

import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './ship-viewer.sail'
import { createMimeRegistry, extensionOf, pdfUrlOf, type ViewerSource } from './media'
import { defaultEntries } from '../engine/registry'
import { attachViewer } from './attach'

function langOf(filename: string): string {
  return extensionOf(filename)
}

export default controller((ctx: ControllerScope<Scope>) => {
  const registry = createMimeRegistry({ entries: defaultEntries() })
  const srcSig = signal<ViewerSource | null>(null)

  const resolvedSource = derived<ViewerSource | null>(() => {
    const live = srcSig()
    if (live != null) return live
    const ct = ctx.contentType()
    const fn = ctx.filename()
    const tx = ctx.text()
    if (!ct && !fn && !tx) return null
    return { contentType: ct, filename: fn, text: tx }
  })

  const descriptor = derived(() => registry.resolve(resolvedSource() ?? {}))
  const kind = derived(() => descriptor().kind)
  const source = derived<ViewerSource | null>(() => resolvedSource())
  const pdfSrc = derived<string>(() => pdfUrlOf(resolvedSource()))
  const textValue = derived<string>(() => resolvedSource()?.text ?? '')
  const lang = derived<string>(() => langOf(ctx.filename()))
  const from = derived<string>(() => resolvedSource()?.from ?? '')
  const to = derived<string>(() => resolvedSource()?.to ?? '')
  const unsupportedMsg = derived<string>(
    () => `No preview for ${descriptor().normalizedType || 'unknown type'}`,
  )

  mount((host) => attachViewer(host, srcSig))

  return { kind, source, pdfSrc, textValue, lang, from, to, unsupportedMsg }
})
</controller>

<div><slot name="toolbar"></slot></div>
<div>
  <if {kind() === 'pdf'}>
    <ship-pdf src={pdfSrc()}></ship-pdf>
  {else-if {kind() === 'image'} =>
    <sail-media-image src={source()}></sail-media-image>
  }
  {else-if {kind() === 'media'} =>
    <sail-media-av src={source()}></sail-media-av>
  }
  {else-if {kind() === 'tabular'} =>
    <sail-media-csv src={source()}></sail-media-csv>
  }
  {else-if {kind() === 'markdown'} =>
    <sail-markdown value={textValue()}></sail-markdown>
  }
  {else-if {kind() === 'code'} =>
    <code-block block lang={lang()}>{textValue()}</code-block>
  }
  {else-if {kind() === 'json'} =>
    <sail-json-view data={textValue()}></sail-json-view>
  }
  {else-if {kind() === 'diff'} =>
    <sail-diff value={textValue()} from={from()} to={to()}></sail-diff>
  }
  {else =>
    <empty-state>
      <span slot="title">No preview</span>
      {unsupportedMsg()}
      <slot name="download" slot="action"></slot>
    </empty-state>
  }</if>
</div>
