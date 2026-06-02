// JSDOM polyfills for the @ship/pdf pack.
//
// The vendored pdf.js build touches DOM/graphics globals (DOMMatrix, Path2D,
// OffscreenCanvas, ResizeObserver) that jsdom does not ship. These stubs let
// the engine import and the components mount under the test environment.

import { installCanvas2dStub } from './canvas-stub'

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}

if (typeof globalThis.DOMMatrix === 'undefined') {
  // @ts-expect-error — minimal JSDOM stub; pdf.js only constructs an identity matrix at import
  globalThis.DOMMatrix = class DOMMatrix {
    a = 1
    b = 0
    c = 0
    d = 1
    e = 0
    f = 0
    constructor(_init?: unknown) {}
  }
}

if (typeof globalThis.Path2D === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  globalThis.Path2D = class Path2D {}
}

// Uint8Array.prototype.toHex/.fromHex — the Stage-3 hex codec pdf.js uses to
// compute document fingerprints. Present in browsers + bun, absent in the Node
// runtime vitest transforms under here. Test-env shim only; production runs
// where the methods exist natively.
{
  const proto = Uint8Array.prototype as unknown as {
    toHex?: () => string
    fromHex?: (hex: string) => Uint8Array
  }
  if (typeof proto.toHex !== 'function') {
    proto.toHex = function (this: Uint8Array): string {
      let s = ''
      for (const b of this) s += b.toString(16).padStart(2, '0')
      return s
    }
  }
  const ctor = Uint8Array as unknown as { fromHex?: (hex: string) => Uint8Array }
  if (typeof ctor.fromHex !== 'function') {
    ctor.fromHex = (hex: string): Uint8Array => {
      const out = new Uint8Array(hex.length / 2)
      for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16)
      return out
    }
  }
}

// Map.prototype.getOrInsertComputed — the Stage-3 upsert helper pdf.js uses on
// the render path's method-promise cache. Present in bun, absent in this Node.
{
  const proto = Map.prototype as unknown as {
    getOrInsertComputed?<K, V>(key: K, compute: (key: K) => V): V
  }
  if (typeof proto.getOrInsertComputed !== 'function') {
    proto.getOrInsertComputed = function <K, V>(this: Map<K, V>, key: K, compute: (key: K) => V): V {
      if (this.has(key)) return this.get(key) as V
      const value = compute(key)
      this.set(key, value)
      return value
    }
  }
}

if (typeof globalThis.OffscreenCanvas === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  globalThis.OffscreenCanvas = class OffscreenCanvas {
    width = 0
    height = 0
    constructor(w?: number, h?: number) {
      this.width = w ?? 0
      this.height = h ?? 0
    }
  }
}

// jsdom has no canvas backend (`getContext('2d')` returns null), so patch every
// canvas — including ones the runtime mounts inside a shadow root — to hand
// pdf.js a no-op recording context. Specs assert the engine API + canvas
// dimensions, never pixel output.
installCanvas2dStub()
