import { controller, effect } from '@ship/ui-runtime'
import { LUCIDE_ICONS } from '../../../ship-set/src/web-components/core/lucide-icons.generated'
import type { Scope } from './sui-icon.sui'

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

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

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

    const slotEl = scope.slotEl() as HTMLSlotElement | null
    let svgEl: SVGSVGElement | null = null

    const stopSvg = effect(() => {
      const name = scope.name()
      const label = scope.label()
      const resolved = resolveName(name)
      const paths = resolved ? LUCIDE_ICONS[resolved] : undefined

      if (paths && slotEl) {
        if (!svgEl) {
          svgEl = document.createElementNS('http://www.w3.org/2000/svg', 'svg') as SVGSVGElement
          svgEl.setAttribute('viewBox', '0 0 24 24')
          svgEl.setAttribute('fill', 'none')
          svgEl.setAttribute('stroke', 'currentColor')
          svgEl.setAttribute('stroke-width', '2')
          svgEl.setAttribute('stroke-linecap', 'round')
          svgEl.setAttribute('stroke-linejoin', 'round')
          svgEl.setAttribute('part', 'svg')
          svgEl.classList.add('sui-icon-svg')
          slotEl.appendChild(svgEl)
        }
        svgEl.innerHTML = paths
        if (label) {
          svgEl.removeAttribute('aria-hidden')
        } else {
          svgEl.setAttribute('aria-hidden', 'true')
        }
      } else {
        if (svgEl) {
          svgEl.remove()
          svgEl = null
        }
      }
    })

    return () => {
      stopAria()
      stopSvg()
      if (svgEl) {
        svgEl.remove()
        svgEl = null
      }
    }
  },
})
