import { controller, effect, signal } from '@ship/sui-runtime'
import { fetchIcon, getCachedIcon } from './registry'
import type { Scope } from './sui-icon.sui'

const SVG_NS = 'http://www.w3.org/2000/svg'

// Bumped after each successful fetch so the render effect re-runs when an
// icon lands. Tests prime the cache synchronously and never observe a bump.
const fetchTick = signal(0)

/** Cache-or-fetch one URL. Used by the `src=` path; the `name=` path goes
 *  through `fetchIcon` which encapsulates the daemon's per-icon endpoint. */
const srcCache = new Map<string, string | null>()
const srcInflight = new Map<string, Promise<string | null>>()
function fetchSrc(url: string): Promise<string | null> {
  if (srcCache.has(url)) return Promise.resolve(srcCache.get(url) ?? null)
  const pending = srcInflight.get(url)
  if (pending) return pending
  const p = fetch(url)
    .then((r) => (r.ok ? r.text() : null))
    .catch(() => null)
    .then((svg) => {
      srcCache.set(url, svg)
      srcInflight.delete(url)
      return svg
    })
  srcInflight.set(url, p)
  return p
}

/** Parse one `<svg>` element from a standalone SVG document string. Returns
 *  the root or null when the input is malformed. */
function parseSvg(doc: Document, markup: string): SVGSVGElement | null {
  const parsed = new DOMParser().parseFromString(markup, 'image/svg+xml')
  const root = parsed.documentElement
  if (!(root instanceof SVGSVGElement)) return null
  return doc.adoptNode(root) as SVGSVGElement
}

export default controller<Scope>({
  mount(state, host) {
    const root = host.shadowRoot!
    let svgEl: SVGSVGElement | null = null
    let lastKey: string | null = null

    const stopAria = effect(() => {
      const label = state.$signals.label()
      if (label) {
        host.setAttribute('role', 'img')
        host.setAttribute('aria-label', label)
        host.removeAttribute('aria-hidden')
      } else {
        host.setAttribute('aria-hidden', 'true')
        host.removeAttribute('role')
        host.removeAttribute('aria-label')
      }
    })

    const stopRender = effect(() => {
      // Subscribe to the fetch tick so this effect re-runs when an icon lands.
      fetchTick()
      const src = state.$signals.src()
      const name = state.$signals.name()
      const label = state.$signals.label()
      const key = src ? `src:${src}` : name ? `name:${name}` : ''
      if (!key) {
        if (svgEl) {
          svgEl.remove()
          svgEl = null
        }
        lastKey = null
        return
      }
      // Resolve to fully-fetched SVG markup synchronously when cached, else
      // kick a fetch and bail (the effect will re-run on tick).
      let markup: string | null | undefined
      if (src) {
        markup = srcCache.get(src)
        if (markup === undefined) {
          fetchSrc(src).then(() => fetchTick.set(fetchTick() + 1))
          return
        }
      } else {
        markup = getCachedIcon(name)
        if (markup === undefined) {
          fetchIcon(name).then(() => fetchTick.set(fetchTick() + 1))
          return
        }
      }
      if (!markup) {
        // 404 or malformed — drop any prior render so the slot fallback shows.
        if (svgEl) {
          svgEl.remove()
          svgEl = null
        }
        lastKey = null
        return
      }
      if (key !== lastKey) {
        const parsed = parseSvg(host.ownerDocument, markup)
        if (!parsed) return
        // The daemon emits `stroke="currentColor"` so host CSS `color` flows
        // through; preserve the parsed attributes verbatim.
        parsed.setAttribute('part', 'svg')
        parsed.setAttribute('class', 'sui-icon-svg')
        if (svgEl) svgEl.replaceWith(parsed)
        else root.insertBefore(parsed, root.firstChild)
        svgEl = parsed
        lastKey = key
      }
      if (svgEl) {
        if (label) svgEl.removeAttribute('aria-hidden')
        else svgEl.setAttribute('aria-hidden', 'true')
      }
    })

    return () => {
      stopAria()
      stopRender()
    }
  },
})

/** Test-only: bump the tick so primed-cache changes flow through the effect. */
export function _bumpForTests(): void {
  fetchTick.set(fetchTick() + 1)
}

// Reserved for SVG namespace usage in future programmatic builders.
void SVG_NS
