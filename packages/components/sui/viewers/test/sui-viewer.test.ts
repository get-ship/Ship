// TESTS-FIRST — FOUNDATIONAL slice: the <sui-viewer> dispatcher.
//
// Status: RED. The dispatcher source + controller are greenfield:
//   - `../src/sui-viewer/sui-viewer.sui`            (template — {#match kind()})
//   - `../src/sui-viewer/sui-viewer.controller.ts`  (tier-2 controller)
// Neither exists yet, so `readFileSync`/the controller import throws and every
// spec errors RED. DO NOT add the impl here. DO NOT weaken into passing stubs.
//
// Harness: `renderComponent` from `@ship/test-setup` compiles the dispatcher
// `.sui` from source under the `sui-viewer` tag and registers it as a custom
// element (the new-tag-via-harness path — no WASM rebuild). The composed
// committed renderer arms (`sui-json-view`, `sui-markdown`, `sui-empty-state`)
// are registered by importing their primitives `.compiled.js`, exactly as the
// dispatcher composes them at runtime.
//
// Authority: build-spec §"The <sui-viewer> dispatcher view", §"Controller
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
import '../../primitives/src/sui-json-view/sui-json-view.compiled.js'
import '../../primitives/src/sui-markdown/sui-markdown.compiled.js'
import '../../primitives/src/sui-empty-state/sui-empty-state.compiled.js'
// RED: greenfield dispatcher controller.
import dispatcher from '../src/sui-viewer/sui-viewer.controller'

const SOURCE = readFileSync(
  new URL('../src/sui-viewer/sui-viewer.sui', import.meta.url),
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
  const v = renderComponent(SOURCE, 'sui-viewer', { controller: dispatcher })
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

describe('<sui-viewer> dispatcher — registration', () => {
  it('registers the custom element', () => {
    const v = renderComponent(SOURCE, 'sui-viewer', { controller: dispatcher })
    live.push(v)
    expect(customElements.get('sui-viewer')).toBeDefined()
  })

  it('host carries role=region (a11y landmark)', async () => {
    const v = mountViewer({ contentType: 'application/json', text: '{"a":1}' })
    live.push(v)
    await v.flush()
    expect(v.el.getAttribute('role')).toBe('region')
  })
})

describe('<sui-viewer> dispatcher — {#match kind()} routing (#1)', () => {
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

  it('csv content-type renders a sui-viewer-csv child', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-viewer-csv')).not.toBeNull()
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
    const v = renderComponent(SOURCE, 'sui-viewer', {
      controller: dispatcher,
      attrs: { 'content-type': 'application/json', text: '{"a":1}' },
    })
    live.push(v)
    await v.flush()
    expect(shadow(v.el).querySelector('sui-json-view')).not.toBeNull()
  })
})

describe('<sui-viewer> dispatcher — unsupported → VISIBLE empty state (#8)', () => {
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
    const v = renderComponent(SOURCE, 'sui-viewer', { controller: dispatcher })
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

describe('<sui-viewer> dispatcher — lifecycle events bubble to the host (#9)', () => {
  // The dispatcher's re-emit contract is tested in isolation from any specific
  // renderer's emit behavior: the active child renderer emits a bubbling+
  // composed ViewerRenderer event (sui-viewer-load / sui-viewer-error), and the
  // dispatcher re-fires it from the host so ONE listener on <sui-viewer> catches
  // any renderer. We fire a synthetic child event into the dispatcher's shadow
  // subtree to drive exactly that re-emit seam (each real renderer's own emit is
  // covered in its renderer spec, e.g. sui-viewer-csv).
  const childEvent = (name: string, detail?: unknown): CustomEvent =>
    new CustomEvent(name, { detail, bubbles: true, composed: true })

  it('re-emits a child sui-viewer-load from the dispatcher host', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    let loads = 0
    v.el.addEventListener('sui-viewer-load', () => loads++)
    const child = shadow(v.el).querySelector('sui-viewer-csv')
    expect(child).not.toBeNull()
    child!.dispatchEvent(childEvent('sui-viewer-load'))
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })

  it('a single on:sui-viewer-error on <sui-viewer> catches a child error {message}', async () => {
    const v = mountViewer({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    let caught: CustomEvent | null = null
    v.el.addEventListener('sui-viewer-error', (e) => { caught = e as CustomEvent })
    const child = shadow(v.el).querySelector('sui-viewer-csv')
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
    shadow(v.el).querySelector('sui-viewer-csv')!.dispatchEvent(childEvent('sui-viewer-load'))
    await v.flush()
    expect(caught).not.toBeNull()
  })
})
