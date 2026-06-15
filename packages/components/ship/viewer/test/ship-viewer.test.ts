// The <ship-viewer> dispatcher (the content-renderer ROUTER):
//   - `../src/ship-viewer/ship-viewer.sui`            (template — {#match kind()})
//   - `../src/ship-viewer/ship-viewer.controller.ts`  (tier-2 controller)
//
// Harness: `renderComponent` from `@ship/test-setup` compiles the dispatcher
// `.sui` from source under the `ship-viewer` tag and registers it as a custom
// element (the new-tag-via-harness path — no WASM rebuild). The composed
// committed renderer arms (`sui-json-view`, `sui-markdown`, `sui-empty-state`)
// are registered by importing their primitives `.compiled.js`, exactly as the
// dispatcher composes them at runtime.
//
// Authority: build-spec §"The <ship-viewer> dispatcher view", §"Controller
// shape", behavioral specs #1 (dispatch by content-type, swap without remount),
// #8 (unsupported → VISIBLE empty state via slots), #9 (lifecycle events bubble
// to the dispatcher).

import { readFileSync } from 'node:fs'
import { afterEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'
// Composed committed renderer arms — register before the dispatcher mounts.
// Cross-pack relative imports into @ship/sui-primitives's src tree: the
// compiled-js shim resolves the sibling `.sui` and compiles it on demand
// (primitives' package `exports` only maps `.`, so subpath imports resolve by
// path, not by package export).
import '../../../sui/primitives/src/sui-json-view/sui-json-view.compiled.js'
import '../../../sui/primitives/src/sui-markdown/sui-markdown.compiled.js'
import '../../../sui/primitives/src/sui-empty-state/sui-empty-state.compiled.js'
// The pdf arm dispatches to `<ship-pdf>` (the @ship/pdf SHIP-tier renderer).
// Register it the same way the composed primitives are registered — by importing
// its compiled.js — so the dispatcher resolves a real, defined element.
import '@ship/pdf'
import dispatcher from '../src/ship-viewer/ship-viewer.controller'

const SOURCE = readFileSync(
  new URL('../src/ship-viewer/ship-viewer.sui', import.meta.url),
  'utf8',
)

interface ViewerSource {
  contentType?: string
  filename?: string
  text?: string
  bytes?: ArrayBuffer | Uint8Array
  from?: string
  to?: string
}

/** Mount the dispatcher and assign a live ViewerSource via the `prop:src`
 *  identity property (never a stringified attribute). Returns the harness
 *  handle; query `el.shadowRoot` for the resolved child renderer. */
function mountViewer(src: ViewerSource) {
  const v = renderComponent(SOURCE, 'ship-viewer', { controller: dispatcher })
  ;(v.el as unknown as { src: ViewerSource }).src = src
  return v
}

const shadow = (el: HTMLElement): ShadowRoot => el.shadowRoot as ShadowRoot

// Track every mounted instance for teardown (renderComponent returns an
// `unmount`, but a thrown spec must not leak a host into the next test).
const live: Array<{ unmount(): void }> = []
afterEach(() => {
  for (const v of live.splice(0)) {
    try {
      v.unmount()
    } catch {
      /* best-effort */
    }
  }
})

describe('<ship-viewer> dispatcher — registration', () => {

  it('host carries role=region (a11y landmark)', async () => {
    const v = mountViewer({ contentType: 'application/json', text: '{"a":1}' })
    live.push(v)
    await v.flush()
    expect(v.el.getAttribute('role')).toBe('region')
  })
})

describe('<ship-viewer> dispatcher — {#match kind()} routing (#1)', () => {
  it('json content-type renders a sui-json-view child', async () => {
    const v = mountViewer({ contentType: 'application/json', text: '{"a":1}' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-json-view')).not.toBeNull()
  })

  it('markdown content-type renders a sui-markdown child', async () => {
    const v = mountViewer({ contentType: 'text/markdown', text: '# Hi' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-markdown')).not.toBeNull()
  })

  it('csv content-type renders a sui-media-csv child', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-media-csv')).not.toBeNull()
  })

  it('swaps the child renderer when the source kind changes — no host remount', async () => {
    const v = mountViewer({ contentType: 'application/json', text: '{"a":1}' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-json-view')).not.toBeNull()
    const before = v.el

    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/markdown', text: '# Hi' }
    await v.flush()
    expect(v.el).toBe(before) // same host element — only the child swapped
    expect(shadow(v.el).querySelector('sui-markdown')).not.toBeNull()
    expect(shadow(v.el).querySelector('sui-json-view')).toBeNull()
  })

  it('seeds detection from the content-type/text SSR attributes (no prop:src)', async () => {
    const v = renderComponent(SOURCE, 'ship-viewer', {
      controller: dispatcher,
      attrs: { 'content-type': 'application/json', text: '{"a":1}' },
    })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-json-view')).not.toBeNull()
  })
})

describe('<ship-viewer> dispatcher — unsupported → VISIBLE empty state (#8)', () => {
  it('an unknown type renders sui-empty-state (the {:default} arm)', async () => {
    const v = mountViewer({ contentType: 'application/x-unknown' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-empty-state')).not.toBeNull()
    expect(shadow(v.el).querySelector('sui-json-view')).toBeNull()
  })

  it('shows the "No preview" title via slot="title" (NOT an inert prop)', async () => {
    const v = mountViewer({ contentType: 'application/x-unknown' })
    live.push(v)
    await v.flush()
    const titleSlotted = shadow(v.el).querySelector('[slot="title"]')
    expect(titleSlotted).not.toBeNull()
    expect(titleSlotted!.textContent).toContain('No preview')
  })

  it('shows the normalized type in the empty-state body message', async () => {
    const v = mountViewer({ contentType: 'application/x-unknown' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).textContent).toContain('application/x-unknown')
  })

  it('forwards a consumer download affordance into the empty-state action slot', async () => {
    const v = renderComponent(SOURCE, 'ship-viewer', { controller: dispatcher })
    live.push(v)
    const dl = document.createElement('button')
    dl.setAttribute('slot', 'download')
    dl.textContent = 'Download'
    v.el.appendChild(dl)
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'application/x-unknown' }
    await v.flush()
    expect(shadow(v.el).querySelector('slot[name="download"]')).not.toBeNull()
  })

  it('does NOT reflect [data-viewer-loading] for an unsupported type', async () => {
    const v = mountViewer({ contentType: 'application/x-unknown' })
    live.push(v)
    await v.flush()
    expect(v.el.hasAttribute('data-viewer-loading')).toBe(false)
  })
})

describe('<ship-viewer> dispatcher — lifecycle events bubble to the host (#9)', () => {
  // The dispatcher's re-emit contract is tested in isolation from any specific
  // renderer's emit behavior: the active child renderer emits a bubbling+
  // composed ViewerRenderer event (sui-viewer-load / sui-viewer-error), and the
  // dispatcher re-fires it from the host so ONE listener on <ship-viewer> catches
  // any renderer. We fire a synthetic child event into the dispatcher's shadow
  // subtree to drive exactly that re-emit seam (each real renderer's own emit is
  // covered in its renderer spec, e.g. sui-media-csv).
  const childEvent = (name: string, detail?: unknown): CustomEvent =>
    new CustomEvent(name, { detail, bubbles: true, composed: true })

  it('re-emits a child sui-viewer-load from the dispatcher host', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    let loads = 0
    v.el.addEventListener('sui-viewer-load', () => loads++)
    const child = shadow(v.el).querySelector('sui-media-csv')
    expect(child).not.toBeNull()
    child!.dispatchEvent(childEvent('sui-viewer-load'))
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })

  it('a single on:sui-viewer-error on <ship-viewer> catches a child error {message}', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    v.el.addEventListener('sui-viewer-error', (e) => { caught = e as CustomEvent })
    const child = shadow(v.el).querySelector('sui-media-csv')
    child!.dispatchEvent(childEvent('sui-viewer-error', { message: 'boom' }))
    await v.flush()
    expect(caught).not.toBeNull()
    expect((caught as unknown as CustomEvent).detail).toHaveProperty('message')
  })

  it('the dispatcher re-emitted event bubbles + crosses the shadow boundary (composed)', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    // Listen on the container ABOVE the host — only a bubbling, composed event reaches it.
    v.root.addEventListener('sui-viewer-load', (e) => { caught = e as CustomEvent })
    shadow(v.el).querySelector('sui-media-csv')!.dispatchEvent(childEvent('sui-viewer-load'))
    await v.flush()
    expect(caught).not.toBeNull()
  })
})

describe('<ship-viewer> dispatcher — pdf routes to <ship-pdf> (NOT the fallback)', () => {
  it('an application/pdf source dispatches to a real <ship-pdf> element', async () => {
    const v = mountViewer({ contentType: 'application/pdf', url: 'https://x/doc.pdf' })
    live.push(v)
    await v.flush()
    const pdf = shadow(v.el).querySelector('ship-pdf')
    expect(pdf).not.toBeNull()
    // a REAL, defined element — not an inert/undefined custom element
    expect(customElements.get('ship-pdf')).toBeDefined()
    // and NOT the {:default}/empty-state fallback
    expect(shadow(v.el).querySelector('sui-empty-state')).toBeNull()
  })

  it('routes a .pdf filename (extension detection) to <ship-pdf>', async () => {
    const v = mountViewer({ filename: 'report.pdf', url: 'https://x/report.pdf' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('ship-pdf')).not.toBeNull()
    expect(shadow(v.el).querySelector('sui-empty-state')).toBeNull()
  })

  it('passes the resolved url through to <ship-pdf> as its src', async () => {
    const v = mountViewer({ contentType: 'application/pdf', url: 'https://x/doc.pdf' })
    live.push(v)
    await v.flush()
    const pdf = shadow(v.el).querySelector('ship-pdf') as (HTMLElement & { src?: string }) | null
    expect(pdf).not.toBeNull()
    expect(pdf!.src).toBe('https://x/doc.pdf')
  })
})
