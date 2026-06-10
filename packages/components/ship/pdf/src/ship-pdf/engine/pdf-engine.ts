// pdf-engine — the small load/render engine <ship-pdf> wraps around the
// pdf.js (pdfjs-dist) build. It owns the loading task + the document/page proxies and the
// canvas draw; the controller is glue (read props → drive the engine → emit
// events). Keeping the pdf.js surface here means the component never touches
// `getDocument`/`RenderTask` directly and the engine is unit-testable on its
// own (load a tiny PDF, assert `numPages`, probe the canvas context).
//
// Worker: the pack bundles the pdf.js worker module (`@ship/pdf`'s `index.ts`
// side-effect-imports it), which registers a main-thread `WorkerMessageHandler`
// (`globalThis.pdfjsWorker`). pdf.js detects that handler and renders on the main
// thread automatically — no URL, no separate fetch, no CDN, works offline. So no
// `workerSrc` is required. The `workerSrc` seam below stays for explicit
// overrides: a Node test env (whose ESM loader rejects `http:` worker imports)
// hands a `file://` URL, and a host that wants a dedicated Worker thread can pass
// the served worker URL. Importing `pdfWorkerUrl` from the pack barrel also pulls
// the bundled worker module into THIS module's graph, so the engine never relies
// on a separate side-effect import landing first.

import { getDocument, GlobalWorkerOptions } from 'pdfjs-dist'
import type {
  DocumentInitParameters,
  PDFDocumentProxy,
  PDFPageProxy,
  RenderTask,
} from 'pdfjs-dist'
import { pdfWorkerUrl } from '../../index'

/** What the engine loads from: a URL (http/https/data/blob) or raw bytes. */
export type PdfSource =
  | { url: string }
  | { data: ArrayBuffer | Uint8Array }

export interface PdfEngineOptions {
  /** Override the worker URL to spawn a dedicated Worker. Omit to use the pack's
   *  bundled main-thread worker handler (the default — offline, no URL). */
  workerSrc?: string
}

export interface RenderResult {
  /** The 1-based page number that was rendered. */
  page: number
  /** Rendered viewport pixel width (canvas width). */
  width: number
  /** Rendered viewport pixel height (canvas height). */
  height: number
}

/** Coerce a `prop:src`-style string into a PdfSource. A `data:` URI is decoded
 *  to bytes (pdf.js loads `data:` faster from bytes than via its URL stream);
 *  everything else (http/https/blob) loads as a URL. */
export function sourceFromString(src: string): PdfSource | null {
  if (!src) return null
  if (src.startsWith('data:')) {
    const comma = src.indexOf(',')
    if (comma === -1) return null
    const meta = src.slice(5, comma)
    const body = src.slice(comma + 1)
    if (meta.includes(';base64')) {
      const bin = atob(body)
      const bytes = new Uint8Array(bin.length)
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
      return { data: bytes }
    }
    return { data: new TextEncoder().encode(decodeURIComponent(body)) }
  }
  return { url: src }
}

/** A load/render engine bound to one mounted `<ship-pdf>`. Single source at a
 *  time: `load()` destroys the previous document before opening a new one. */
export interface PdfEngine {
  /** Open a document from a source. Resolves with the page count. Rejects
   *  (never throws synchronously) on an invalid/corrupt source. */
  load(source: PdfSource): Promise<number>
  /** Page count of the loaded document, or 0 before a successful load. */
  pageCount(): number
  /** Render a 1-based page into `canvas` at `zoom` scale. Resolves with the
   *  rendered dimensions. Rejects if no document is loaded or the page is out
   *  of range. Cancels any in-flight render first. */
  renderPage(canvas: HTMLCanvasElement, page: number, zoom: number): Promise<RenderResult>
  /** Destroy the document + worker transport and cancel any in-flight render. */
  destroy(): void
}

export function createPdfEngine(opts: PdfEngineOptions = {}): PdfEngine {
  // An explicit `opts.workerSrc` makes pdf.js spawn a dedicated Worker from that
  // URL (tests pass a `file://`; a host can pass a served URL). Otherwise we fall
  // back to `pdfWorkerUrl` only when nothing is configured — but pdf.js prefers
  // the bundled main-thread handler (registered by the pack's worker-module
  // import) over any `workerSrc`, so this fallback just keeps the bundled worker
  // referenced. A `workerSrc` a host already set (e.g. `useVendoredWorker()`) is
  // respected, never clobbered.
  if (opts.workerSrc) {
    GlobalWorkerOptions.workerSrc = opts.workerSrc
  } else if (!GlobalWorkerOptions.workerSrc) {
    GlobalWorkerOptions.workerSrc = pdfWorkerUrl
  }

  let doc: PDFDocumentProxy | null = null
  let task: RenderTask | null = null
  let pageProxy: PDFPageProxy | null = null
  let pageProxyNum = 0

  const cancelRender = (): void => {
    if (task) {
      try {
        task.cancel()
      } catch {
        /* a cancelled task may reject; the caller's promise handles it */
      }
      task = null
    }
  }

  const closeDoc = (): void => {
    cancelRender()
    pageProxy = null
    pageProxyNum = 0
    if (doc) {
      void doc.destroy().catch(() => {})
      doc = null
    }
  }

  return {
    async load(source: PdfSource): Promise<number> {
      closeDoc()
      const params: DocumentInitParameters =
        'url' in source ? { url: source.url } : { data: source.data }
      doc = await getDocument(params).promise
      return doc.numPages
    },

    pageCount(): number {
      return doc ? doc.numPages : 0
    },

    async renderPage(
      canvas: HTMLCanvasElement,
      page: number,
      zoom: number,
    ): Promise<RenderResult> {
      if (!doc) throw new Error('ship-pdf: no document loaded')
      if (page < 1 || page > doc.numPages) {
        throw new Error(`ship-pdf: page ${page} out of range (1..${doc.numPages})`)
      }
      cancelRender()
      // Cache the page proxy across re-renders of the same page (zoom changes).
      if (pageProxyNum !== page || !pageProxy) {
        pageProxy = await doc.getPage(page)
        pageProxyNum = page
      }
      const scale = zoom > 0 ? zoom : 1
      const viewport = pageProxy.getViewport({ scale })
      canvas.width = Math.max(1, Math.floor(viewport.width))
      canvas.height = Math.max(1, Math.floor(viewport.height))
      const ctx = canvas.getContext('2d')
      if (!ctx) throw new Error('ship-pdf: 2d canvas context unavailable')
      task = pageProxy.render({ canvasContext: ctx, viewport, canvas })
      await task.promise
      task = null
      return { page, width: canvas.width, height: canvas.height }
    },

    destroy(): void {
      closeDoc()
    },
  }
}
