import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-tooltip.sui'

let _seq = 0

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const bubbleId = `sui-tooltip-${++_seq}`

    const stopBubbleId = effect(() => {
      const bubble = scope.bubbleEl() as HTMLElement | null
      if (bubble) bubble.id = bubbleId
    })

    const wireTrigger = () => {
      const slot = scope.defaultSlot() as HTMLSlotElement | null
      if (!slot) return
      const first = slot.assignedElements({ flatten: true })[0] as HTMLElement | undefined
      if (!first) return
      const existing = first.getAttribute('aria-describedby') ?? ''
      if (!existing.split(/\s+/).includes(bubbleId)) {
        first.setAttribute(
          'aria-describedby',
          existing ? `${existing} ${bubbleId}` : bubbleId,
        )
      }
    }

    const stopSlot = effect(() => {
      const slot = scope.defaultSlot() as HTMLSlotElement | null
      if (!slot) return
      slot.addEventListener('slotchange', wireTrigger)
      wireTrigger()
      return () => slot.removeEventListener('slotchange', wireTrigger)
    })

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') (host as HTMLElement).blur()
    }
    host.addEventListener('keydown', onKeydown)

    return () => {
      stopBubbleId()
      stopSlot()
      host.removeEventListener('keydown', onKeydown)
    }
  },
})
