// <ship-pdf> controller — drives the pdf.js (pdfjs-dist) load/render engine and
// wires it to the component's props + lifecycle events. The engine owns the
// pdf.js surface and the canvas draw (`engine/pdf-engine.ts`); this controller
// is glue: read `src`/`page`/`zoom` → load/render → emit `ship-pdf-load` /
// `ship-pdf-page`, and surface a load/render failure as the template's visible
// error state (never a blank canvas).
//
// The prop signals (`src`/`page`/`zoom`) are read in the `scope` factory — where
// they're the reactive prop accessors — into deriveds (`pdfSource`/`pageNum`/
// `zoomLevel`). `mount` reads those deriveds through the writable state proxy's
// value side, which subscribes the surrounding effect, so an attribute/property
// change re-runs the load/render.
//
// Two reactive effects in `mount`:
//   - LOAD: re-runs when `pdfSource` changes — opens the document, emits
//     `ship-pdf-load { pages }`, clears any prior error, then triggers a render.
//   - RENDER: re-runs when the loaded doc / `pageNum` / `zoomLevel` / the
//     bind:this canvas change — paints the page and emits `ship-pdf-page` when
//     the displayed page changes.
// Any rejection flips `errorMessage`; the template swaps the canvas for
// <sui-error-state role=alert>.

import { controller, derived, effect, emit, ref, signal } from '@ship/sui-runtime'
import { createPdfEngine, sourceFromString, type PdfSource } from './engine/pdf-engine'
import type { Scope } from './ship-pdf.sui'

export default controller<Scope>({
  scope: ({ src, page, zoom, label }: Scope) => {
    // bind:this target — the engine-owned canvas pdf.js paints into.
    const canvasEl = ref<HTMLCanvasElement>(null)
    // Reactive views of the props, read where they're the live prop signals.
    const pdfSource = derived<PdfSource | null>(() => sourceFromString(src()))
    const pageNum = derived<number>(() => page())
    const zoomLevel = derived<number>(() => zoom())
    // Page count of the loaded document (0 before load).
    const pages = signal(0)
    const errorMessage = signal('')
    const errored = derived<boolean>(() => errorMessage() !== '')
    return { canvasEl, pdfSource, pageNum, zoomLevel, pages, errorMessage, errored, label }
  },

  mount(state, host) {
    const engine = createPdfEngine()

    // Loaded-document generation: bumped on every successful load so the render
    // effect re-runs after a fresh document opens (the engine identity is
    // stable, so we need an explicit reactive trigger).
    const docGen = signal(0)
    let lastEmittedPage = 0

    // --- LOAD: re-resolve whenever the source changes ----------------------
    const stopLoad = effect(() => {
      const source = state.pdfSource as PdfSource | null // value-side read subscribes
      // No source yet (or cleared) — clear error + page count, nothing to draw.
      if (!source) {
        state.errorMessage = ''
        state.pages = 0
        return
      }
      void engine
        .load(source)
        .then((count) => {
          state.errorMessage = ''
          state.pages = count
          lastEmittedPage = 0
          emit(host, 'ship-pdf-load', { pages: count })
          docGen.set(docGen() + 1)
        })
        .catch((err: unknown) => {
          state.pages = 0
          state.errorMessage =
            err instanceof Error ? err.message : 'The document could not be opened.'
        })
    })

    // --- RENDER: re-resolve on doc / page / zoom / canvas change -----------
    const stopRender = effect(() => {
      docGen() // subscribe — re-render after each successful load
      const page = state.pageNum as number
      const zoom = state.zoomLevel as number
      const canvas = state.canvasEl as HTMLCanvasElement | null
      // Canvas absent (error branch is showing) or no document loaded → skip.
      if (!canvas || engine.pageCount() === 0) return
      void engine
        .renderPage(canvas, page, zoom)
        .then((result) => {
          if (result.page !== lastEmittedPage) {
            lastEmittedPage = result.page
            emit(host, 'ship-pdf-page', { page: result.page })
          }
        })
        .catch((err: unknown) => {
          state.errorMessage =
            err instanceof Error ? err.message : 'The page could not be rendered.'
        })
    })

    return () => {
      stopLoad()
      stopRender()
      engine.destroy()
    }
  },
})
