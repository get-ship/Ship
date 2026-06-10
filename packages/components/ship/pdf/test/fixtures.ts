// Shared test fixtures for the @ship/pdf suite.
//
// A minimal, hand-built one-page PDF (no images/fonts — just a catalog, a pages
// node, and a single MediaBox page) as a base64 `data:` URI, plus its decoded
// bytes. pdf.js opens it and reports `numPages === 1`.
//
// `WORKER_FILE_URL` is the `pdfjs-dist` worker as a `file://` URL — the Node ESM
// loader vitest runs under rejects an `http:` worker import, so tests hand the
// engine this path-based URL. Production uses the same-origin `pdfWorkerUrl`
// (the bundler-emitted asset).

import { createRequire } from 'node:module'
import { pathToFileURL } from 'node:url'

/** Base64 body of a valid 1-page PDF (MediaBox 100×100). */
export const TINY_PDF_BASE64 =
  'JVBERi0xLjEKMSAwIG9iago8PCAvVHlwZSAvQ2F0YWxvZyAvUGFnZXMgMiAwIFIgPj4KZW5kb2JqCjIgMCBvYmoKPDwgL1R5cGUgL1BhZ2VzIC9LaWRzIFsgMyAwIFIgXSAvQ291bnQgMSA+PgplbmRvYmoKMyAwIG9iago8PCAvVHlwZSAvUGFnZSAvUGFyZW50IDIgMCBSIC9NZWRpYUJveCBbIDAgMCAxMDAgMTAwIF0gPj4KZW5kb2JqCnRyYWlsZXIKPDwgL1Jvb3QgMSAwIFIgPj4KJSVFT0YK'

/** The same 1-page PDF as a `data:` URI (what a `prop:src` string would carry). */
export const TINY_PDF_DATA_URI = `data:application/pdf;base64,${TINY_PDF_BASE64}`

/** Decoded bytes of the 1-page PDF. */
export function tinyPdfBytes(): Uint8Array {
  const bin = atob(TINY_PDF_BASE64)
  const out = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i)
  return out
}

/** Bytes that are NOT a PDF — getDocument must reject on these. */
export function invalidPdfBytes(): Uint8Array {
  return new TextEncoder().encode('this is definitely not a pdf')
}

/** A `data:` URI carrying the invalid bytes (an invalid `prop:src`). */
export const INVALID_DATA_URI = `data:application/pdf;base64,${btoa('not a pdf at all')}`

/** The `pdfjs-dist` worker as a `file://` URL for the Node test ESM loader,
 *  resolved through the installed package so it survives the package moving. */
export const WORKER_FILE_URL: string = pathToFileURL(
  createRequire(import.meta.url).resolve('pdfjs-dist/build/pdf.worker.mjs'),
).href
