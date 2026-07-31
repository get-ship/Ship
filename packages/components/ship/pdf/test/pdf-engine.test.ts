// SLICE 1 — the load/render engine. These specs drive the `getDocument`
// wrapper directly (no DOM/custom-element): a tiny valid PDF resolves with
// numPages===1, a page renders into a canvas (asserted via the engine's
// numPages/page + canvas dimensions + a probed 2d context, NOT pixel output —
// jsdom's canvas is non-rendering), and an invalid source REJECTS (no throw).

import { describe, it, expect, afterEach } from 'vitest'
import { createPdfEngine, sourceFromString, type PdfEngine } from '../src/ship-pdf/engine/pdf-engine'
import { tinyPdfBytes, invalidPdfBytes, TINY_PDF_DATA_URI, WORKER_FILE_URL } from './fixtures'
import { stubCanvas2d } from './canvas-stub'

const engines: PdfEngine[] = []
function engine(): PdfEngine {
  const e = createPdfEngine({ workerSrc: WORKER_FILE_URL })
  engines.push(e)
  return e
}
afterEach(() => {
  for (const e of engines.splice(0)) {
    try {
      e.destroy()
    } catch {
      /* best-effort */
    }
  }
})

describe('sourceFromString', () => {
  it('decodes a base64 data: URI to bytes', () => {
    const s = sourceFromString(TINY_PDF_DATA_URI)
    expect(s).not.toBeNull()
    expect('data' in s!).toBe(true)
  })

  it('treats http/blob strings as a url source', () => {
    expect(sourceFromString('https://x/y.pdf')).toEqual({ url: 'https://x/y.pdf' })
    expect(sourceFromString('blob:abc')).toEqual({ url: 'blob:abc' })
  })

  it('returns null for an empty string', () => {
    expect(sourceFromString('')).toBeNull()
  })
})

describe('createPdfEngine — load', () => {
  it('getDocument resolves numPages === 1 for a tiny one-page PDF', async () => {
    const e = engine()
    const pages = await e.load({ data: tinyPdfBytes() })
    expect(pages).toBe(1)
    expect(e.pageCount()).toBe(1)
  })

  it('loads the same PDF from a data: URI source string', async () => {
    const e = engine()
    const src = sourceFromString(TINY_PDF_DATA_URI)!
    const pages = await e.load(src)
    expect(pages).toBe(1)
  })

  it('REJECTS (no throw) on an invalid source — pageCount stays 0', async () => {
    const e = engine()
    await expect(e.load({ data: invalidPdfBytes() })).rejects.toBeTruthy()
    expect(e.pageCount()).toBe(0)
  })
})

describe('createPdfEngine — renderPage', () => {
  it('renders page 1 into a canvas and reports positive dimensions', async () => {
    const e = engine()
    await e.load({ data: tinyPdfBytes() })
    const canvas = stubCanvas2d(document.createElement('canvas'))
    const result = await e.renderPage(canvas, 1, 1)
    expect(result.page).toBe(1)
    expect(result.width).toBeGreaterThan(0)
    expect(result.height).toBeGreaterThan(0)
    // canvas was sized by the engine to the rendered viewport
    expect(canvas.width).toBe(result.width)
    expect(canvas.height).toBe(result.height)
  })

  it('scales the canvas with zoom', async () => {
    const e = engine()
    await e.load({ data: tinyPdfBytes() })
    const c1 = stubCanvas2d(document.createElement('canvas'))
    const c2 = stubCanvas2d(document.createElement('canvas'))
    const a = await e.renderPage(c1, 1, 1)
    const b = await e.renderPage(c2, 1, 2)
    expect(b.width).toBeGreaterThan(a.width)
    expect(b.height).toBeGreaterThan(a.height)
  })

  it('rejects rendering a page out of range', async () => {
    const e = engine()
    await e.load({ data: tinyPdfBytes() })
    const canvas = document.createElement('canvas')
    await expect(e.renderPage(canvas, 2, 1)).rejects.toThrow(/out of range/)
  })

  it('rejects rendering before any document is loaded', async () => {
    const e = engine()
    const canvas = document.createElement('canvas')
    await expect(e.renderPage(canvas, 1, 1)).rejects.toThrow(/no document/)
  })
})
