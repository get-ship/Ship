// Theme-aware palette: resolves token names like "accent" or "surface-fg" to
// 0xRRGGBB ints for Pixi tints. Reads CSS custom properties from the host
// element's computed style. Watches `documentElement.dataset.theme` so a
// theme switch invalidates the cache and notifies subscribers.

export interface Palette {
  resolve(tokenOrHex: string): number
  onChange(fn: () => void): () => void
  destroy(): void
}

const HEX_RE = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i

function parseHex(input: string): number | null {
  const m = HEX_RE.exec(input.trim())
  if (!m) return null
  let hex = m[1] as string
  if (hex.length === 3) hex = hex[0]! + hex[0]! + hex[1]! + hex[1]! + hex[2]! + hex[2]!
  return Number.parseInt(hex, 16)
}

// Round-trip any CSS-color (hex, rgb(), hsl(), oklch(), color()) through a
// 1×1 canvas so we always end up with RGB bytes. getComputedStyle may return
// oklch() verbatim on modern browsers, so a regex-only path silently drops
// theme tokens defined in oklch.
let probeCtx: CanvasRenderingContext2D | null = null
function getProbeCtx(): CanvasRenderingContext2D | null {
  if (probeCtx) return probeCtx
  if (typeof document === 'undefined') return null
  const c = document.createElement('canvas')
  c.width = c.height = 1
  probeCtx = c.getContext('2d', { willReadFrequently: true })
  return probeCtx
}

function parseCssColor(input: string): number | null {
  const v = input.trim()
  if (!v) return null
  const hex = parseHex(v)
  if (hex !== null) return hex
  const ctx = getProbeCtx()
  if (!ctx) return null
  try {
    ctx.fillStyle = '#000000'
    ctx.clearRect(0, 0, 1, 1)
    ctx.fillStyle = v
    ctx.fillRect(0, 0, 1, 1)
    const [r, g, b, a] = ctx.getImageData(0, 0, 1, 1).data
    if (a === 0) return null
    return ((r ?? 0) << 16) | ((g ?? 0) << 8) | (b ?? 0)
  } catch {
    return null
  }
}

export function createPalette(host: Element): Palette {
  const cache = new Map<string, number>()
  const subs = new Set<() => void>()

  function read(token: string): number {
    const direct = parseHex(token)
    if (direct !== null) return direct
    const cached = cache.get(token)
    if (cached !== undefined) return cached
    const cs = getComputedStyle(host)
    const candidates = [`--${token}`, `--tone-${token}-bg`, `--color-${token}`]
    for (const v of candidates) {
      const raw = cs.getPropertyValue(v)
      if (raw) {
        const c = parseCssColor(raw)
        if (c !== null) {
          cache.set(token, c)
          return c
        }
      }
    }
    cache.set(token, 0xffffff)
    return 0xffffff
  }

  const obs = new MutationObserver(() => {
    cache.clear()
    for (const fn of subs) fn()
  })
  obs.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme', 'class'],
  })

  return {
    resolve(tokenOrHex: string) { return read(tokenOrHex) },
    onChange(fn: () => void) {
      subs.add(fn)
      return () => subs.delete(fn)
    },
    destroy() {
      obs.disconnect()
      subs.clear()
      cache.clear()
    },
  }
}
