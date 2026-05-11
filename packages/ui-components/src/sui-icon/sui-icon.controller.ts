import { controller, effect, signal } from '@ship/sui-runtime'
import {
  getIconRegistry,
  getIconSvg,
  loadIconRegistry,
  resolveIconName,
} from './registry'
import type { Scope } from './sui-icon.sui'

const SVG_NS = 'http://www.w3.org/2000/svg'

// Bumped once the registry first lands so all live `<sui-icon>` effects
// re-resolve their `name`. Tests prime the registry synchronously
// (see test/setup.ts) and never observe the bump.
const registryRev = signal(getIconRegistry() ? 1 : 0)
let kicked = false
function kickFetch(): void {
  if (kicked || getIconRegistry()) return
  kicked = true
  loadIconRegistry()
    .then(() => registryRev.set(registryRev() + 1))
    .catch(() => {
      kicked = false
    })
}

function makeSvg(doc: Document): SVGSVGElement {
  const svg = doc.createElementNS(SVG_NS, 'svg') as SVGSVGElement
  svg.setAttribute('part', 'svg')
  svg.setAttribute('class', 'sui-icon-svg')
  svg.setAttribute('viewBox', '0 0 24 24')
  svg.setAttribute('fill', 'none')
  svg.setAttribute('stroke', 'currentColor')
  svg.setAttribute('stroke-width', '2')
  svg.setAttribute('stroke-linecap', 'round')
  svg.setAttribute('stroke-linejoin', 'round')
  return svg
}

function populatePaths(svg: SVGSVGElement, fragment: string): void {
  while (svg.firstChild) svg.removeChild(svg.firstChild)
  const parsed = new DOMParser().parseFromString(
    `<svg xmlns="${SVG_NS}">${fragment}</svg>`,
    'image/svg+xml',
  ).documentElement
  while (parsed.firstChild) {
    svg.appendChild(svg.ownerDocument.adoptNode(parsed.firstChild))
  }
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const root = host.shadowRoot!
    let svgEl: SVGSVGElement | null = null
    let lastResolved: string | null = null

    const stopAria = effect(() => {
      const label = scope.label()
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
      // Subscribe to the registry-load signal so the effect re-runs
      // once the lazily-fetched registry lands.
      registryRev()
      const requested = scope.name()
      const resolved = resolveIconName(requested) ?? null
      const label = scope.label()
      if (!resolved) {
        if (svgEl) {
          svgEl.remove()
          svgEl = null
        }
        lastResolved = null
        // If a name was requested but the registry isn't loaded yet,
        // kick the fetch — the effect re-runs once it lands.
        if (requested && !getIconRegistry()) kickFetch()
        return
      }
      if (!svgEl) {
        svgEl = makeSvg(host.ownerDocument)
        root.insertBefore(svgEl, root.firstChild)
      }
      if (resolved !== lastResolved) {
        populatePaths(svgEl, getIconSvg(resolved)!)
        lastResolved = resolved
      }
      if (label) svgEl.removeAttribute('aria-hidden')
      else svgEl.setAttribute('aria-hidden', 'true')
    })

    return () => {
      stopAria()
      stopRender()
    }
  },
})
