import { describe, it, expect, beforeEach } from 'vitest'
import * as pdfjs from '../src/vendor/pdfjs/pdfjs.mjs'
import { pdfWorkerUrl, useVendoredWorker } from '../src/index.ts'

// SLICE 0 contract: the vendored pdf.js engine is real, self-contained, and
// wired to its vendored worker offline (no CDN). These specs guard the vendor.

describe('vendored pdf.js engine', () => {
  it('exposes the load-bearing API', () => {
    expect(typeof pdfjs.getDocument).toBe('function')
    // GlobalWorkerOptions is a class-based singleton (typeof 'function') whose
    // workerSrc/workerPort accessors are the offline-worker config surface.
    expect(pdfjs.GlobalWorkerOptions).toBeTruthy()
    expect('workerSrc' in pdfjs.GlobalWorkerOptions).toBe(true)
    expect('workerPort' in pdfjs.GlobalWorkerOptions).toBe(true)
    expect(typeof pdfjs.PDFWorker).toBe('function')
  })

  it('reports the vendored version (6.0.227)', () => {
    expect(pdfjs.version).toBe('6.0.227')
  })

  it('re-exports the engine through the pack barrel', async () => {
    const barrel = await import('../src/index.ts')
    expect(barrel.getDocument).toBe(pdfjs.getDocument)
    expect(barrel.version).toBe('6.0.227')
  })
})

describe('offline worker wiring', () => {
  beforeEach(() => {
    pdfjs.GlobalWorkerOptions.workerSrc = ''
    pdfjs.GlobalWorkerOptions.workerPort = null
  })

  it('resolves the worker URL same-origin via import.meta.url (no CDN)', () => {
    expect(pdfWorkerUrl).toContain('pdf.worker.mjs')
    expect(pdfWorkerUrl).not.toMatch(/cdn|unpkg|jsdelivr|cloudflare/i)
  })

  it('useVendoredWorker points GlobalWorkerOptions at the vendored worker', () => {
    expect(pdfjs.GlobalWorkerOptions.workerSrc).toBe('')
    useVendoredWorker()
    expect(pdfjs.GlobalWorkerOptions.workerSrc).toBe(pdfWorkerUrl)
    // idempotent
    useVendoredWorker()
    expect(pdfjs.GlobalWorkerOptions.workerSrc).toBe(pdfWorkerUrl)
  })
})
