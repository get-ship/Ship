import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-stepper.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'progressbar')

    const stopEffect = effect(() => {
      const t = Math.max(1, Math.floor(scope.total() || 0))
      const c = Math.max(1, Math.min(t, Math.floor(scope.current() || 1)))
      const label = scope.label()

      host.setAttribute('aria-valuemin', '1')
      host.setAttribute('aria-valuemax', String(t))
      host.setAttribute('aria-valuenow', String(c))
      if (label) {
        host.setAttribute('aria-label', label)
      } else {
        host.setAttribute('aria-label', `Step ${c} of ${t}`)
      }

      const container = scope.dotsEl() as HTMLElement | null
      if (!container) return
      container.textContent = ''
      for (let i = 1; i <= t; i++) {
        const dot = document.createElement('span')
        dot.className = 'dot'
        dot.setAttribute('part', 'dot')
        dot.setAttribute('data-state', i === c ? 'active' : i < c ? 'done' : 'pending')
        dot.setAttribute('aria-hidden', 'true')
        container.appendChild(dot)
      }
    })

    return () => {
      stopEffect()
    }
  },
})
