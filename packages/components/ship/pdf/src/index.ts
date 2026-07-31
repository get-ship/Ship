// @ship/pdf — Ship PDF pack (pdf.js engine via the `pdfjs-dist` npm dep).
//
// Wraps `pdfjs-dist` (declared as a runtime dependency); the Rolldown bundler
// inlines the engine at build, so the published pack carries no committed
// engine source. SLICE 0 landed the engine + its offline-worker wiring;
// SLICE 1 lands <ship-pdf>, the page renderer. Each tag's
// `<tag>.compiled.js` calls customElements.define() on import, so a single
// `import '@ship/pdf'` registers every ship-pdf-* tag globally.
import './ship-pdf/ship-pdf.compiled.js'

// Offline worker, same-origin, no CDN — bundled, not fetched.
//
// pdf.js needs its worker MessageHandler available. The dedicated-Worker path
// (`new Worker(url, { type: 'module' })`) needs the worker as a URL, which the
// deck Rolldown bundler can't emit as a served asset (no `?url`). So we bundle
// the worker module straight INTO the pack instead: importing
// `pdfjs-dist/build/pdf.worker.mjs` side-effect-registers
// `globalThis.pdfjsWorker.WorkerMessageHandler`, and pdf.js detects that handler
// and runs it on the main thread (its "fake worker" path) — no URL, no separate
// fetch, no CDN, works offline / in-product. Tradeoff: rendering runs on the
// main thread rather than a dedicated Worker thread, so a very large / heavy PDF
// can jank the UI. A host that wants a real Worker thread can serve the worker
// itself and pass an explicit `workerSrc`/`workerPort` (see the engine options).
import 'pdfjs-dist/build/pdf.worker.mjs'

// Re-export the engine's public API so consumers import from the pack, not the
// `pdfjs-dist` specifier directly.
export * from 'pdfjs-dist'

import { GlobalWorkerOptions } from 'pdfjs-dist'

/**
 * Same-origin specifier of the pdf.js worker module the pack bundles. The worker
 * runs from the bundled module (never a CDN), so this is informational/explicit:
 * a host that wants a dedicated Worker thread (rather than the bundled
 * main-thread handler) can hand this to `new Worker(pdfWorkerUrl, { type: 'module' })`
 * or assign `GlobalWorkerOptions.workerSrc`. Safe offline / in-product.
 */
export const pdfWorkerUrl = 'pdfjs-dist/build/pdf.worker.mjs'

/**
 * Opt into a dedicated Worker thread by pointing pdf.js at the bundled worker
 * specifier. Call once at startup, before any `getDocument()`. Idempotent —
 * re-applying the same value is a no-op.
 *
 * By default the pack needs NO call here: the bundled worker module registers a
 * main-thread handler (`globalThis.pdfjsWorker`) that pdf.js uses automatically.
 * Use this only when a host wants pdf.js to spawn a real Worker (and can resolve
 * `pdfWorkerUrl` to a served, same-origin URL). For explicit worker lifecycle
 * control, construct a `Worker(pdfWorkerUrl, { type: 'module' })` and assign
 * `GlobalWorkerOptions.workerPort` instead.
 */
export function useVendoredWorker(): void {
  GlobalWorkerOptions.workerSrc = pdfWorkerUrl
}
