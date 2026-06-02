// @ship/pdf — Ship PDF pack (SHIP-tier vendored engine).
//
// Wraps a vendored pdf.js (pdfjs-dist) build that ships in src/vendor/pdfjs —
// it is NOT an npm dependency. SLICE 0 landed the engine + its offline-worker
// wiring; SLICE 1 lands <ship-pdf>, the page renderer. Each tag's
// `<tag>.compiled.js` calls customElements.define() on import, so a single
// `import '@ship/pdf'` registers every ship-pdf-* tag globally.
import './ship-pdf/ship-pdf.compiled.js'

// Re-export the vendored engine's public API so consumers import from the pack,
// not the vendor path directly.
export * from './vendor/pdfjs/pdfjs.mjs'

import { GlobalWorkerOptions } from './vendor/pdfjs/pdfjs.mjs'

/**
 * URL of the vendored pdf.js worker, resolved relative to this module so it
 * loads same-origin from the host — never a CDN. Safe offline / in-product.
 */
export const pdfWorkerUrl: string = new URL(
  './vendor/pdfjs/pdf.worker.mjs',
  import.meta.url,
).href

/**
 * Point pdf.js at the vendored worker. Call once at startup, before any
 * `getDocument()`. Idempotent — re-applying the same URL is a no-op.
 *
 * Uses `GlobalWorkerOptions.workerSrc` (the URL form). For explicit worker
 * lifecycle control, construct a `Worker(pdfWorkerUrl, { type: 'module' })`
 * and assign `GlobalWorkerOptions.workerPort` instead.
 */
export function useVendoredWorker(): void {
  GlobalWorkerOptions.workerSrc = pdfWorkerUrl
}
