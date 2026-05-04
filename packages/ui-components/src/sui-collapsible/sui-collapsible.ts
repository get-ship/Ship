import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-collapsible.sui'

let _seq = 0

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const contentId = `sui-collapsible-${++_seq}`

    const stopContentId = effect(() => {
      const wrap = scope.contentWrap() as HTMLElement | null
      if (wrap) wrap.id = contentId
    })

    const syncAriaExpanded = () => {
      const slot = scope.triggerSlot() as HTMLSlotElement | null
      if (!slot) return
      const isOpen = scope.open()
      for (const el of slot.assignedElements()) {
        const h = el as HTMLElement
        h.setAttribute('aria-expanded', String(isOpen))
        h.setAttribute('aria-controls', contentId)
        if (!h.hasAttribute('tabindex') && !h.matches('button, a, input, select, textarea')) {
          h.setAttribute('tabindex', '0')
        }
      }
    }

    const stopSlot = effect(() => {
      const slot = scope.triggerSlot() as HTMLSlotElement | null
      if (!slot) return
      slot.addEventListener('slotchange', syncAriaExpanded)
      syncAriaExpanded()
      return () => slot.removeEventListener('slotchange', syncAriaExpanded)
    })

    const stopOpenSync = effect(() => {
      scope.open() // track
      scope.triggerSlot() // track
      syncAriaExpanded()
    })

    const toggle = () => {
      if (scope.disabled()) return
      const next = !scope.open()
      scope.open.set(next)
      host.dispatchEvent(
        new CustomEvent('sui-toggle', {
          detail: { open: next },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        toggle()
      }
    }

    const stopWrap = effect(() => {
      const wrap = scope.triggerWrap() as HTMLElement | null
      if (!wrap) return
      wrap.addEventListener('click', toggle)
      wrap.addEventListener('keydown', onKeydown)
      return () => {
        wrap.removeEventListener('click', toggle)
        wrap.removeEventListener('keydown', onKeydown)
      }
    })

    return () => {
      stopContentId()
      stopSlot()
      stopOpenSync()
      stopWrap()
    }
  },
})
