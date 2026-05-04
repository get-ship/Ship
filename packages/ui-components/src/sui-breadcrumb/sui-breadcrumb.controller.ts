import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-breadcrumb.sui'

const SEP_SVG = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6"/></svg>'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const navEl = scope.navEl() as HTMLElement | null
    if (!navEl) return

    const rebuild = () => {
      const items = Array.from(
        host.querySelectorAll<Element>(':scope > sui-breadcrumb-item'),
      )
      items.forEach((el, i) => el.setAttribute('slot', `i${i}`))

      while (navEl.firstChild) navEl.removeChild(navEl.firstChild)

      const sep = scope.separator()
      for (let i = 0; i < items.length; i++) {
        const slotEl = document.createElement('slot')
        slotEl.setAttribute('name', `i${i}`)
        navEl.appendChild(slotEl)
        if (i < items.length - 1) {
          const sepEl = document.createElement('span')
          sepEl.className = 'sep'
          sepEl.setAttribute('aria-hidden', 'true')
          if (sep === '/') {
            sepEl.innerHTML = SEP_SVG
          } else {
            sepEl.textContent = sep
          }
          navEl.appendChild(sepEl)
        }
      }
    }

    const observer = new MutationObserver(rebuild)
    observer.observe(host, { childList: true })

    const stopEffect = effect(() => {
      scope.separator()
      rebuild()
    })

    return () => {
      observer.disconnect()
      stopEffect()
    }
  },
})
