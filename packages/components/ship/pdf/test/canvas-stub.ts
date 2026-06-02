// A minimal, non-rendering 2d-context stub for the test env.
//
// jsdom ships no canvas backend, so `canvas.getContext('2d')` returns null and
// the pdf.js canvas renderer can't draw. These specs do NOT assert pixel output
// (that's an integration concern); they assert the engine's load/render API +
// the canvas DIMENSIONS the engine sets. The stub is a recording no-op context
// that answers every method/property pdf.js touches so `render().promise`
// resolves without a real raster backend.

interface ImageDataLike {
  data: Uint8ClampedArray
  width: number
  height: number
}

/** Build the no-op recording 2d context. */
function make2dContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const makeImageData = (w: number, h: number): ImageDataLike => ({
    data: new Uint8ClampedArray(Math.max(1, w) * Math.max(1, h) * 4),
    width: w,
    height: h,
  })

  const base: Record<string, unknown> = {
    canvas,
    // Drawing-state + path methods pdf.js calls — all no-ops.
    save() {},
    restore() {},
    scale() {},
    rotate() {},
    translate() {},
    transform() {},
    setTransform() {},
    resetTransform() {},
    getTransform() {
      return { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 }
    },
    clearRect() {},
    fillRect() {},
    strokeRect() {},
    beginPath() {},
    closePath() {},
    moveTo() {},
    lineTo() {},
    bezierCurveTo() {},
    quadraticCurveTo() {},
    arc() {},
    rect() {},
    fill() {},
    stroke() {},
    clip() {},
    fillText() {},
    strokeText() {},
    measureText() {
      return { width: 0 }
    },
    drawImage() {},
    setLineDash() {},
    getLineDash() {
      return []
    },
    createLinearGradient() {
      return { addColorStop() {} }
    },
    createRadialGradient() {
      return { addColorStop() {} }
    },
    createPattern() {
      return {}
    },
    // Pixel readback — pdf.js composes layers via these; hand back zeroed data.
    getImageData(_x: number, _y: number, w: number, h: number) {
      return makeImageData(w, h)
    },
    putImageData() {},
    createImageData(w: number, h: number) {
      return makeImageData(w, h)
    },
  }

  const ctx = new Proxy(base, {
    get(target, prop: string) {
      if (prop in target) return (target as Record<string, unknown>)[prop]
      // Unknown property reads return a no-op function (covers any method we
      // didn't enumerate) so pdf.js never trips over `undefined is not a fn`.
      return () => undefined
    },
    set(target, prop: string, value) {
      ;(target as Record<string, unknown>)[prop] = value
      return true
    },
  })

  return ctx as unknown as CanvasRenderingContext2D
}

/** Override `getContext('2d')` on a single `canvas` to return a no-op recording
 *  2d context. Returns the same canvas for chaining. */
export function stubCanvas2d(canvas: HTMLCanvasElement): HTMLCanvasElement {
  const ctx = make2dContext(canvas)
  Object.defineProperty(canvas, 'getContext', {
    configurable: true,
    value: (kind: string) => (kind === '2d' ? ctx : null),
  })
  return canvas
}

/** Patch `HTMLCanvasElement.prototype.getContext` so EVERY canvas (including
 *  ones the runtime creates inside a shadow root, which a test can't reach to
 *  stub individually) answers `getContext('2d')` with a no-op context. Call
 *  once from the suite setup. Idempotent. */
let prototypePatched = false
export function installCanvas2dStub(): void {
  if (prototypePatched) return
  prototypePatched = true
  const proto = HTMLCanvasElement.prototype as unknown as {
    getContext: (kind: string) => CanvasRenderingContext2D | null
  }
  proto.getContext = function (this: HTMLCanvasElement, kind: string) {
    return kind === '2d' ? make2dContext(this) : null
  }
}
