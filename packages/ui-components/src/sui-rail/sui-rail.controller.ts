import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-rail.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // Reactively update toggle button icon + label based on collapsed state.
    const stopIcon = effect(() => {
      const icon = scope.toggleIcon() as HTMLElement | null
      const btn = scope.toggleBtn() as HTMLElement | null
      const col = scope.collapsed()
      if (icon) icon.setAttribute('name', col ? 'chevrons-right' : 'chevrons-left')
      if (btn) btn.setAttribute('aria-label', col ? 'Expand rail' : 'Collapse rail')
    })

    const onToggle = () => {
      const next = !scope.collapsed()
      scope.collapsed.set(next)
      host.dispatchEvent(
        new CustomEvent('sui-rail:toggle', {
          detail: { collapsed: next },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const stopBtn = effect(() => {
      const btn = scope.toggleBtn() as HTMLButtonElement | null
      if (!btn) return
      btn.addEventListener('click', onToggle)
      return () => btn.removeEventListener('click', onToggle)
    })

    return () => {
      stopIcon()
      stopBtn()
    }
  },
})
