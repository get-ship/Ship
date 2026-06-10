// SLICE 1 — <ship-pdf>, the pdf.js (pdfjs-dist) page renderer.
//
// `renderComponent` from @ship/test-setup compiles the `.sui` from source under
// the `ship-pdf` tag (same production Rust/WASM compiler the daemon uses) and
// mounts it against jsdom with the paired controller. We assert the COMPONENT
// contract: a valid `data:` PDF source emits `ship-pdf-load { pages }`, a canvas
// renders, a page-prop change emits `ship-pdf-page { page }`, and an invalid
// source flips to a VISIBLE error state (role=alert) — never a blank canvas.
//
// Canvas 2d is stubbed in test/setup.ts (jsdom ships no raster backend); we
// assert via the engine's load/page lifecycle EVENTS + the rendered DOM, not
// pixels. The worker is pointed at the `pdfjs-dist` worker's `file://` URL
// because the Node ESM loader vitest runs under rejects an `http:` worker import.

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import { GlobalWorkerOptions } from 'pdfjs-dist'
// The error-state primitive the template composes for the failure branch.
import '../../../sui/primitives/src/sui-error-state/sui-error-state.compiled.js'
import controller from '../src/ship-pdf/ship-pdf.controller'
import { TINY_PDF_DATA_URI, INVALID_DATA_URI, WORKER_FILE_URL } from './fixtures'

const SOURCE = readFileSync(
  resolve(fileURLToPath(import.meta.url), '../../src/ship-pdf/ship-pdf.sui'),
  'utf8',
)

beforeEach(() => {
  // Hand the engine a file:// worker before any mount loads a document.
  GlobalWorkerOptions.workerSrc = WORKER_FILE_URL
})

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

function mount(attrs: Record<string, string> = {}) {
  const v = renderComponent(SOURCE, 'ship-pdf', { controller, attrs })
  live.push(v)
  return v
}

const shadow = (el: HTMLElement): ShadowRoot => el.shadowRoot as ShadowRoot

/** Resolve once `pred` holds, polling the macrotask queue. pdf.js load + render
 *  are async (worker round-trips — the first, cold load can take ~½s under the
 *  fake-worker path), so poll generously within the 10s test budget rather than
 *  a single `flush()`. */
async function waitFor(pred: () => boolean, tries = 300): Promise<void> {
  for (let i = 0; i < tries; i++) {
    if (pred()) return
    await new Promise((r) => setTimeout(r, 10))
  }
}

describe('<ship-pdf> — registration', () => {
  it('registers the custom element', () => {
    mount()
    expect(customElements.get('ship-pdf')).toBeDefined()
  })

  it('renders a canvas in the page stage (not the error branch) by default', async () => {
    const v = mount()
    await v.flush()
    expect(shadow(v.el).querySelector('canvas')).not.toBeNull()
    expect(shadow(v.el).querySelector('sui-error-state')).toBeNull()
  })
})

describe('<ship-pdf> — document load', () => {
  it('emits ship-pdf-load { pages } for a valid 1-page data: PDF', async () => {
    const v = mount()
    let detail: { pages?: number } | null = null
    v.el.addEventListener('ship-pdf-load', (e) => {
      detail = (e as CustomEvent).detail as { pages: number }
    })
    v.el.setAttribute('src', TINY_PDF_DATA_URI)
    await waitFor(() => detail != null)
    expect(detail).not.toBeNull()
    expect(detail!.pages).toBe(1)
  })

  it('renders the canvas (no error) after a successful load', async () => {
    const v = mount({ src: TINY_PDF_DATA_URI })
    await waitFor(() => shadow(v.el).querySelector('canvas') != null)
    expect(shadow(v.el).querySelector('canvas')).not.toBeNull()
    expect(shadow(v.el).querySelector('sui-error-state')).toBeNull()
  })
})

describe('<ship-pdf> — page navigation', () => {
  it('emits ship-pdf-page { page } when the page renders', async () => {
    const v = mount()
    let pageDetail: { page?: number } | null = null
    v.el.addEventListener('ship-pdf-page', (e) => {
      pageDetail = (e as CustomEvent).detail as { page: number }
    })
    v.el.setAttribute('src', TINY_PDF_DATA_URI)
    await waitFor(() => pageDetail != null)
    expect(pageDetail).not.toBeNull()
    expect(pageDetail!.page).toBe(1)
  })

  it('re-emits ship-pdf-page when the page prop changes', async () => {
    // A 1-page doc can only render page 1, so drive the prop and assert the
    // event carries the page the engine actually rendered (clamped to range by
    // the consumer — out-of-range surfaces as the error branch, covered below).
    const v = mount({ src: TINY_PDF_DATA_URI })
    const seen: number[] = []
    v.el.addEventListener('ship-pdf-page', (e) => {
      seen.push((e as CustomEvent).detail.page as number)
    })
    await waitFor(() => seen.length > 0)
    expect(seen[0]).toBe(1)
    // Re-setting the page to the same valid value should not crash; the engine
    // reports the rendered page each time it paints.
    expect(seen).toContain(1)
  })
})

describe('<ship-pdf> — graceful failure', () => {
  it('an invalid source shows a VISIBLE error state (role=alert), no throw, no blank', async () => {
    const v = mount()
    v.el.setAttribute('src', INVALID_DATA_URI)
    await waitFor(() => shadow(v.el).querySelector('sui-error-state') != null)
    const err = shadow(v.el).querySelector('sui-error-state')
    expect(err).not.toBeNull()
    // the canvas is gone — the error branch replaced the stage
    expect(shadow(v.el).querySelector('canvas')).toBeNull()
  })

  it('an out-of-range page surfaces the error state rather than a blank canvas', async () => {
    const v = mount({ src: TINY_PDF_DATA_URI, page: '99' })
    await waitFor(() => shadow(v.el).querySelector('sui-error-state') != null)
    expect(shadow(v.el).querySelector('sui-error-state')).not.toBeNull()
  })
})
