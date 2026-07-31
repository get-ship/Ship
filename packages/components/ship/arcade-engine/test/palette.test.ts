import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

// palette.ts reads CSS custom properties off a host's computed style, falls
// back to a 1x1 canvas to normalize non-hex colors (rgb/hsl/oklch), and
// watches documentElement for theme changes via MutationObserver. None of
// those are present under environment:'node', so we install focused fakes:
//   - getComputedStyle(host) → returns a token map the test controls
//   - document.createElement('canvas') → a probe with a scriptable 2D context
//   - MutationObserver → records the callback so the test can trip it by hand

interface FakeCtx {
  fillStyle: string
  fillRect(): void
  clearRect(): void
  getImageData(): { data: [number, number, number, number] }
}

let lastObserverCb: (() => void) | null = null
let tokenMap: Record<string, string> = {}
// Maps a CSS color string → the RGBA the fake canvas "paints" for it.
let colorPaint: Record<string, [number, number, number, number]> = {}

function makeFakeCtx(): FakeCtx {
  let current = '#000000'
  return {
    get fillStyle() { return current },
    set fillStyle(v: string) { current = v },
    fillRect() {},
    clearRect() {},
    getImageData() {
      const rgba = colorPaint[current] ?? [0, 0, 0, 0]
      return { data: rgba }
    },
  }
}

function installDom() {
  lastObserverCb = null
  tokenMap = {}
  colorPaint = {}
  const docEl = {} as unknown as Element
  vi.stubGlobal('document', {
    documentElement: docEl,
    createElement: (tag: string) => {
      if (tag !== 'canvas') return {}
      let _w = 0
      let _h = 0
      const ctx = makeFakeCtx()
      return {
        get width() { return _w },
        set width(v: number) { _w = v },
        get height() { return _h },
        set height(v: number) { _h = v },
        getContext: () => ctx,
      }
    },
  })
  vi.stubGlobal('getComputedStyle', () => ({
    getPropertyValue: (name: string) => tokenMap[name] ?? '',
  }))
  class FakeMutationObserver {
    constructor(cb: () => void) { lastObserverCb = cb }
    observe() {}
    disconnect() { lastObserverCb = null }
  }
  vi.stubGlobal('MutationObserver', FakeMutationObserver)
}

beforeEach(() => { installDom() })
afterEach(() => { vi.unstubAllGlobals() })

// Imported AFTER installDom so module-level `typeof document` checks see the fake.
async function loadPalette() {
  return (await import('../src/palette.ts')).createPalette
}

describe('palette: direct hex parsing', () => {
  it('parses a 6-digit hex without touching the host style', async () => {
    const createPalette = await loadPalette()
    const host = {} as unknown as Element
    const p = createPalette(host)
    expect(p.resolve('#ff8800')).toBe(0xff8800)
    expect(p.resolve('ff8800')).toBe(0xff8800)
  })

  it('expands a 3-digit shorthand hex', async () => {
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('#f80')).toBe(0xff8800)
  })
})

describe('palette: token resolution', () => {
  it('resolves a token from the --token custom property (hex value)', async () => {
    tokenMap['--accent'] = '#112233'
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('accent')).toBe(0x112233)
  })

  it('falls back through --tone-<t>-bg then --color-<t> candidates', async () => {
    tokenMap['--color-danger'] = '#abcdef'
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('danger')).toBe(0xabcdef)
  })

  it('round-trips a non-hex (oklch/rgb) token through the canvas probe', async () => {
    tokenMap['--brand'] = 'oklch(0.7 0.1 200)'
    colorPaint['oklch(0.7 0.1 200)'] = [10, 20, 30, 255]
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('brand')).toBe((10 << 16) | (20 << 8) | 30)
  })

  it('caches a resolved token (second resolve does not re-read styles)', async () => {
    tokenMap['--accent'] = '#010203'
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('accent')).toBe(0x010203)
    // Mutate the underlying token; cache should keep returning the old value.
    tokenMap['--accent'] = '#ffffff'
    expect(p.resolve('accent')).toBe(0x010203)
  })

  it('falls back to white for an unresolved token', async () => {
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('does-not-exist')).toBe(0xffffff)
  })

  it('treats a fully-transparent computed color as unresolved (alpha 0)', async () => {
    tokenMap['--ghost'] = 'transparent'
    colorPaint['transparent'] = [50, 60, 70, 0] // alpha 0 → null → white fallback
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    expect(p.resolve('ghost')).toBe(0xffffff)
  })
})

describe('palette: theme-change invalidation', () => {
  it('clears the cache and notifies subscribers on a mutation', async () => {
    tokenMap['--accent'] = '#010203'
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    let notified = 0
    p.onChange(() => notified++)
    expect(p.resolve('accent')).toBe(0x010203)
    // Theme switch: change the token and trip the observer.
    tokenMap['--accent'] = '#0a0b0c'
    lastObserverCb?.()
    expect(notified).toBe(1)
    expect(p.resolve('accent')).toBe(0x0a0b0c) // re-read after cache clear
  })

  it('onChange returns an unsubscribe', async () => {
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    let count = 0
    const off = p.onChange(() => count++)
    lastObserverCb?.()
    expect(count).toBe(1)
    off()
    lastObserverCb?.()
    expect(count).toBe(1)
  })

  it('destroy disconnects the observer and stops notifications', async () => {
    const createPalette = await loadPalette()
    const p = createPalette({} as unknown as Element)
    let count = 0
    p.onChange(() => count++)
    p.destroy()
    expect(lastObserverCb).toBeNull() // disconnect cleared it
    lastObserverCb?.()
    expect(count).toBe(0)
  })
})
