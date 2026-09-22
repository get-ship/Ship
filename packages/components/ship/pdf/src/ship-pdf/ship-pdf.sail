<scope>
  props: {
    src = ''
    page = 1
    zoom = 1
    label = 'PDF page'
  }
</scope>
<controller>
// <ship-pdf> controller — drives the pdf.js (pdfjs-dist) load/render engine and
// wires it to the component's props + lifecycle events. The engine owns the
// pdf.js surface and the canvas draw (`engine/pdf-engine.ts`); this controller
// is glue: read `src`/`page`/`zoom` → load/render → emit `ship-pdf-load` /
// `ship-pdf-page`, and surface a load/render failure as the template's visible
// error state (never a blank canvas).
//
// The prop signals (`src`/`page`/`zoom`) are read in the setup function — where
// they're the reactive prop accessors — into deriveds (`pdfSource`/`pageNum`/
// `zoomLevel`). `mount` reads those deriveds, which subscribes the surrounding
// effect, so an attribute/property change re-runs the load/render.
//
// Two reactive effects in `mount`:
//   - LOAD: re-runs when `pdfSource` changes — opens the document, emits
//     `ship-pdf-load { pages }`, clears any prior error, then triggers a render.
//   - RENDER: re-runs when the loaded doc / `pageNum` / `zoomLevel` / the
//     captured canvas change — paints the page and emits `ship-pdf-page` when
//     the displayed page changes.
// Any rejection flips `errorMessage`; the template swaps the canvas for
// <error-state>.

import { emit } from '@ship/sail-runtime'
import type { Action } from '@ship/sail-runtime'
import { createPdfEngine, sourceFromString, type PdfSource } from './engine/pdf-engine'
import type { Scope } from './ship-pdf.sail'

export default controller((ctx: Scope, host?: Element) => {
  // use: capture target — the engine-owned canvas pdf.js paints into.
  const canvasEl = signal<Element | null>(null)
  const captureCanvas: Action<void> = (node) => {
    canvasEl.set(node)
    return { destroy: () => canvasEl.set(null) }
  }
  // Reactive views of the props, read where they're the live prop signals.
  const pdfSource = derived<PdfSource | null>(() => sourceFromString(ctx.src()))
  const pageNum = derived<number>(() => ctx.page())
  const zoomLevel = derived<number>(() => ctx.zoom())
  // Page count of the loaded document (0 before load).
  const pages = signal(0)
  const errorMessage = signal('')
  const errored = derived<boolean>(() => errorMessage() !== '')

  mount((mountedHost) => {
    const engine = createPdfEngine()
    const target = mountedHost ?? host

    // Loaded-document generation: bumped on every successful load so the render
    // effect re-runs after a fresh document opens (the engine identity is
    // stable, so we need an explicit reactive trigger).
    const docGen = signal(0)
    let lastEmittedPage = 0

    // --- LOAD: re-resolve whenever the source changes ----------------------
    const stopLoad = effect(() => {
      const source = pdfSource()
      // No source yet (or cleared) — clear error + page count, nothing to draw.
      if (!source) {
        errorMessage.set('')
        pages.set(0)
        return
      }
      void engine
        .load(source)
        .then((count) => {
          errorMessage.set('')
          pages.set(count)
          lastEmittedPage = 0
          if (target) emit(target, 'ship-pdf-load', { pages: count })
          docGen.set(docGen() + 1)
        })
        .catch((err: unknown) => {
          pages.set(0)
          errorMessage.set(
            err instanceof Error ? err.message : 'The document could not be opened.',
          )
        })
    })

    // --- RENDER: re-resolve on doc / page / zoom / canvas change -----------
    const stopRender = effect(() => {
      docGen() // subscribe — re-render after each successful load
      const page = pageNum()
      const zoom = zoomLevel()
      const canvas = canvasEl()
      // Canvas absent (error branch is showing) or no document loaded → skip.
      if (!(canvas instanceof HTMLCanvasElement) || engine.pageCount() === 0) return
      void engine
        .renderPage(canvas, page, zoom)
        .then((result) => {
          if (result.page !== lastEmittedPage) {
            lastEmittedPage = result.page
            if (target) emit(target, 'ship-pdf-page', { page: result.page })
          }
        })
        .catch((err: unknown) => {
          errorMessage.set(
            err instanceof Error ? err.message : 'The page could not be rendered.',
          )
        })
    })

    return () => {
      stopLoad()
      stopRender()
      engine.destroy()
    }
  })

  return { captureCanvas, pdfSource, pageNum, zoomLevel, pages, errorMessage, errored }
})
</controller>

<!--
  <ship-pdf> — a PDF page renderer wrapping the pdf.js (pdfjs-dist) engine.

  SHIP-tier (engine via the pdfjs-dist npm dep, not sui-media): the controller
  drives a small load/render engine (`engine/pdf-engine.ts`) that calls pdf.js
  `getDocument()` and paints the current page onto an engine-OWNED <canvas>
  (`use:` capture — pdf.js draws into the 2d context; the controller never sets
  innerHTML). `GlobalWorkerOptions` is pointed at the pack's same-origin
  bundler-emitted worker (never a CDN).

  Props
  ─────────────────────────────────────────────────────────────────────────────
  src    string  reflect  URL, `data:` URI, or blob: URL of the PDF to render.
  page   number  1  reflect  1-based page to display. Driven externally or by
                             the toolbar slot.
  zoom   number  1          render scale (1 = 100%).
  label  string            aria-label for the canvas (defaults to "PDF page").
  ─────────────────────────────────────────────────────────────────────────────
  Events
    ship-pdf-load { pages }  — emitted once a document loads, with its page count.
    ship-pdf-page { page }   — emitted when the rendered page changes.

  Slots
    toolbar — chrome above the page (prev/next/zoom). Empty by default; the
              consumer drops controls in, or drives `page`/`zoom` as props.

  Graceful: a load/render failure renders a VISIBLE error state (error-state,
  role=alert) — never a blank canvas or undefined element.
-->

<div><slot name="toolbar"></slot></div>

<if {errored()}>
  <error-state
    title="Couldn't open PDF"
    body={errorMessage()}
    variant="inline">
  </error-state>
{else =>
  <div>
    <canvas use:captureCanvas aria-label={label()} role="img"></canvas>
  </div>
}</if>
