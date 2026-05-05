import { controller, effect } from '@ship/ui-runtime'
import { LUCIDE_ICONS } from '../../../ship-set/src/web-components/core/lucide-icons.generated'
import type { Scope } from './sui-icon.sui'

const SVG_NS = 'http://www.w3.org/2000/svg'

const ALIASES: Record<string, string> = {
  git: 'git-branch',
  refresh: 'refresh-cw',
  stop: 'stop-circle',
  warning: 'triangle-alert',
}

function resolveName(name: string): string | undefined {
  if (!name) return undefined
  if (LUCIDE_ICONS[name]) return name
  const aliased = ALIASES[name]
  if (aliased && LUCIDE_ICONS[aliased]) return aliased
  return undefined
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
      const resolved = resolveName(scope.name()) ?? null
      const label = scope.label()
      if (!resolved) {
        if (svgEl) {
          svgEl.remove()
          svgEl = null
        }
        lastResolved = null
        return
      }
      if (!svgEl) {
        svgEl = makeSvg(host.ownerDocument)
        root.insertBefore(svgEl, root.firstChild)
      }
      if (resolved !== lastResolved) {
        populatePaths(svgEl, LUCIDE_ICONS[resolved]!)
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
