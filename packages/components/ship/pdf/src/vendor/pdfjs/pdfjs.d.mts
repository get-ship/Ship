// Type declarations for the vendored pdf.js ESM build (pdfjs-dist@6.0.227).
//
// Faithful to the upstream public API surface (package/types/src/pdf.d.ts).
// The load-bearing rendering path — getDocument, the document/page proxies,
// RenderTask, PDFWorker, GlobalWorkerOptions, version — is typed accurately.
// The remaining named exports (editor/layer/util helpers) are re-declared with
// their real names so `import { X } from './vendor/pdfjs/pdfjs.mjs'` type-checks;
// callers that need their internals should narrow at the use site.
//
// This declaration is hand-maintained alongside the vendored build. It is NOT
// generated from an npm dependency — @ship/pdf vendors the engine.

/** Source for loading a PDF: URL, typed array, or full init parameters. */
export type DocumentInitSource =
  | string
  | URL
  | TypedArray
  | ArrayBuffer
  | DocumentInitParameters

export type TypedArray =
  | Int8Array
  | Uint8Array
  | Uint8ClampedArray
  | Int16Array
  | Uint16Array
  | Int32Array
  | Uint32Array
  | Float32Array
  | Float64Array

export interface DocumentInitParameters {
  url?: string | URL
  data?: TypedArray | ArrayBuffer | number[]
  httpHeaders?: Record<string, string>
  withCredentials?: boolean
  password?: string
  /** Worker handle; set instead of GlobalWorkerOptions.workerSrc per-document. */
  worker?: PDFWorker
  verbosity?: number
  cMapUrl?: string
  cMapPacked?: boolean
  standardFontDataUrl?: string
  disableAutoFetch?: boolean
  disableStream?: boolean
  disableRange?: boolean
  rangeChunkSize?: number
  [key: string]: unknown
}

export interface OnProgressParameters {
  loaded: number
  total: number
}

export interface GetViewportParameters {
  scale: number
  rotation?: number
  offsetX?: number
  offsetY?: number
  dontFlip?: boolean
}

export interface PageViewport {
  readonly viewBox: number[]
  readonly scale: number
  readonly rotation: number
  readonly offsetX: number
  readonly offsetY: number
  readonly transform: number[]
  readonly width: number
  readonly height: number
  clone(options?: Partial<GetViewportParameters>): PageViewport
  convertToViewportPoint(x: number, y: number): number[]
  convertToViewportRectangle(rect: number[]): number[]
  convertToPdfPoint(x: number, y: number): number[]
}

export interface RenderParameters {
  canvasContext: CanvasRenderingContext2D
  viewport: PageViewport
  canvas?: HTMLCanvasElement
  intent?: string
  annotationMode?: number
  transform?: number[] | null
  background?: string | CanvasGradient | CanvasPattern
  optionalContentConfigPromise?: Promise<unknown>
  annotationCanvasMap?: Map<string, HTMLCanvasElement>
  pageColors?: { background?: string; foreground?: string } | null
  printAnnotationStorage?: unknown
  isEditing?: boolean
}

export interface RenderTask {
  readonly promise: Promise<void>
  cancel(extraDelay?: number): void
}

export interface RefProxy {
  num: number
  gen: number
}

export interface PDFPageProxy {
  readonly pageNumber: number
  readonly rotate: number
  readonly ref: RefProxy | null
  readonly userUnit: number
  readonly view: number[]
  getViewport(params: GetViewportParameters): PageViewport
  render(params: RenderParameters): RenderTask
  getTextContent(params?: Record<string, unknown>): Promise<TextContent>
  getAnnotations(params?: { intent?: string }): Promise<unknown[]>
  getOperatorList(params?: Record<string, unknown>): Promise<unknown>
  streamTextContent(params?: Record<string, unknown>): ReadableStream
  cleanup(resetStats?: boolean): boolean
}

export interface TextItem {
  str: string
  dir: string
  transform: number[]
  width: number
  height: number
  fontName: string
  hasEOL: boolean
}

export interface TextContent {
  items: Array<TextItem | { type: string }>
  styles: Record<string, unknown>
}

export interface PDFDocumentProxy {
  readonly numPages: number
  readonly fingerprints: [string, string | null]
  readonly loadingTask: PDFDocumentLoadingTask
  getPage(pageNumber: number): Promise<PDFPageProxy>
  getPageIndex(ref: RefProxy): Promise<number>
  getPageLabels(): Promise<string[] | null>
  getMetadata(): Promise<{ info: Record<string, unknown>; metadata: unknown }>
  getOutline(): Promise<unknown[] | null>
  getDestination(id: string): Promise<unknown[] | null>
  getDestinations(): Promise<Record<string, unknown[]>>
  getData(): Promise<Uint8Array>
  saveDocument(): Promise<Uint8Array>
  cleanup(keepLoadedFonts?: boolean): Promise<void>
  destroy(): Promise<void>
}

export interface PDFDocumentLoadingTask {
  readonly docId: string
  readonly promise: Promise<PDFDocumentProxy>
  destroyed: boolean
  onProgress: ((p: OnProgressParameters) => void) | null
  onPassword: ((update: (password: string) => void, reason: number) => void) | null
  destroy(): Promise<void>
}

export class PDFWorker {
  constructor(params?: { name?: string; port?: Worker | null; verbosity?: number })
  readonly promise: Promise<void>
  readonly port: Worker | null
  readonly messageHandler: unknown
  destroy(): void
  static fromPort(params: { port: Worker; name?: string; verbosity?: number }): PDFWorker
  static get workerSrc(): string
}

export interface PDFDataRangeTransport {
  length: number
  initialData: Uint8Array
  requestDataRange(begin: number, end: number): void
  abort(): void
}

/**
 * Worker configuration. Point `workerSrc` at the vendored worker URL, or set
 * `workerPort` to a constructed `Worker`, so pdf.js never fetches from a CDN.
 */
export const GlobalWorkerOptions: {
  workerSrc: string
  workerPort: Worker | null
}

/** Load a PDF document. */
export function getDocument(src?: DocumentInitSource): PDFDocumentLoadingTask

/** Bundled pdf.js version string ("6.0.227"). */
export const version: string

/** Bundled pdf.js build hash. */
export const build: string

export const PixelsPerInch: { CSS: number; PDF: number; PDF_TO_CSS_UNITS: number }
export const AnnotationMode: Readonly<Record<string, number>>
export const AnnotationType: Readonly<Record<string, number>>
export const AnnotationEditorType: Readonly<Record<string, number>>
export const AnnotationEditorParamsType: Readonly<Record<string, number>>
export const OPS: Readonly<Record<string, number>>
export const ImageKind: Readonly<Record<string, number>>
export const PasswordResponses: Readonly<Record<string, number>>
export const PermissionFlag: Readonly<Record<string, number>>
export const VerbosityLevel: Readonly<Record<string, number>>

export class AbortException extends Error {}
export class InvalidPDFException extends Error {}
export class ResponseException extends Error {}
export class RenderingCancelledException extends Error {}

// Remaining public exports preserved by name. They are layer/editor/util
// helpers used by full-fidelity viewers; typed permissively but truthfully named.
export const AnnotationEditorLayer: unknown
export const AnnotationEditorUIManager: unknown
export const AnnotationLayer: unknown
export const ColorPicker: unknown
export const CSSConstants: unknown
export const DOMSVGFactory: unknown
export const DrawLayer: unknown
export const FeatureTest: unknown
export const MathClamp: unknown
export const OutputScale: unknown
export const PDFDateString: unknown
export const SignatureExtractor: unknown
export const SupportedImageMimeTypes: unknown
export const TextLayer: unknown
export const TextLayerImages: unknown
export const TouchManager: unknown
export const Util: unknown
export const XfaLayer: unknown

export function applyOpacity(...args: unknown[]): unknown
export function createValidAbsoluteUrl(...args: unknown[]): URL | null
export function fetchData(...args: unknown[]): Promise<unknown>
export function findContrastColor(...args: unknown[]): unknown
export function getFilenameFromUrl(...args: unknown[]): string
export function getPdfFilenameFromUrl(...args: unknown[]): string
export function getRGB(...args: unknown[]): unknown
export function getRGBA(...args: unknown[]): unknown
export function getUuid(...args: unknown[]): string
export function isDataScheme(url: string): boolean
export function isPdfFile(filename: string): boolean
export function isValidExplicitDest(...args: unknown[]): boolean
export function makeArr(...args: unknown[]): unknown
export function makeMap(...args: unknown[]): unknown
export function makeObj(...args: unknown[]): unknown
export function noContextMenu(event: Event): void
export function normalizeUnicode(text: string): string
export function renderRichText(...args: unknown[]): unknown
export function setLayerDimensions(...args: unknown[]): void
export function shadow<T>(obj: object, prop: string, value: T): T
export function stopEvent(event: Event): void
export function updateUrlHash(...args: unknown[]): string
