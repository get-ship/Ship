import { controller, effect, bindHtml } from '@ship/ui-runtime'
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

    const stopVisibility = effect(() => {
      const svg = scope.svgEl()
      if (!svg) return
      const resolved = resolveName(scope.name())
      svg.style.display = resolved ? '' : 'none'
    })

    const svg = scope.svgEl()
    const stopPaths = svg
      ? bindHtml(svg, () => {
          const resolved = resolveName(scope.name())
          return resolved ? LUCIDE_ICONS[resolved]! : ''
        })
      : () => {}

    return () => {
      stopAria()
      stopVisibility()
      stopPaths()
    }
  },
})
