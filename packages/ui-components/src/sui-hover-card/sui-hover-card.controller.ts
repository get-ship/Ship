import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-hover-card.sui'

let _seq = 0

type PopoverEl = HTMLElement & {
  showPopover?: () => void
  hidePopover?: () => void
}

type Placement = 'top' | 'bottom' | 'left' | 'right'

const GAP = 8

function positionPanel(trigger: HTMLElement, panel: HTMLElement, placement: Placement) {
  const r = trigger.getBoundingClientRect()
  panel.style.top = ''
  panel.style.left = ''
  panel.style.right = ''
  panel.style.bottom = ''
  switch (placement) {
    case 'top':
      panel.style.bottom = `${window.innerHeight - r.top + GAP}px`
      panel.style.left = `${r.left + r.width / 2}px`
      panel.style.transform = 'translateX(-50%)'
      break
    case 'left':
      panel.style.right = `${window.innerWidth - r.left + GAP}px`
      panel.style.top = `${r.top}px`
      break
    case 'right':
      panel.style.left = `${r.right + GAP}px`
      panel.style.top = `${r.top}px`
      break
    case 'bottom':
    default:
      panel.style.top = `${r.bottom + GAP}px`
      panel.style.left = `${r.left + r.width / 2}px`
      panel.style.transform = 'translateX(-50%)'
  }
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const cardId = `sui-hover-card-${++_seq}`

    const triggerSlot = scope.triggerSlot() as HTMLSlotElement | null
    const panelEl = scope.panelEl() as PopoverEl | null

    if (!panelEl) return
    panelEl.id = cardId

    let showTimer: ReturnType<typeof setTimeout> | null = null
    let hideTimer: ReturnType<typeof setTimeout> | null = null

    function clearTimers() {
      if (showTimer) { clearTimeout(showTimer); showTimer = null }
      if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
    }

    function wireTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[] ?? []
      for (const el of els) {
        el.setAttribute('aria-describedby', cardId)
      }
    }

    function doShow() {
      clearTimers()
      const trigger = (triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[])?.[0]
      if (trigger && panelEl) {
        positionPanel(trigger, panelEl as HTMLElement, (scope.placement() || 'bottom') as Placement)
      }
      panelEl?.showPopover?.()
    }

    function doHide() {
      clearTimers()
      panelEl?.hidePopover?.()
    }

    ;(host as HTMLElement & { show(): void; hide(): void }).show = doShow
    ;(host as HTMLElement & { hide(): void }).hide = doHide

    const onToggle = (ev: Event) => {
      const te = ev as ToggleEvent
      const nowOpen = te.newState === 'open'
      if (nowOpen !== scope.open()) scope.open.set(nowOpen)
      host.setAttribute('data-state', nowOpen ? 'open' : 'closed')
      host.dispatchEvent(new CustomEvent(nowOpen ? 'sui-open' : 'sui-close', {
        bubbles: true, composed: true,
      }))
    }

    const onPointerEnter = () => {
      clearTimers()
      showTimer = setTimeout(() => doShow(), scope.showDelay() ?? 300)
    }

    const onPointerLeave = () => {
      clearTimers()
      hideTimer = setTimeout(() => doHide(), scope.hideDelay() ?? 100)
    }

    const onFocusIn = () => { clearTimers(); doShow() }
    const onFocusOut = () => {
      clearTimers()
      hideTimer = setTimeout(() => doHide(), scope.hideDelay() ?? 100)
    }

    panelEl.addEventListener('toggle', onToggle)
    triggerSlot?.addEventListener('slotchange', wireTrigger)
    wireTrigger()

    host.addEventListener('pointerenter', onPointerEnter)
    host.addEventListener('pointerleave', onPointerLeave)
    host.addEventListener('focusin', onFocusIn)
    host.addEventListener('focusout', onFocusOut)

    const stopSync = effect(() => {
      host.setAttribute('data-state', scope.open() ? 'open' : 'closed')
    })

    return () => {
      clearTimers()
      panelEl.removeEventListener('toggle', onToggle)
      triggerSlot?.removeEventListener('slotchange', wireTrigger)
      host.removeEventListener('pointerenter', onPointerEnter)
      host.removeEventListener('pointerleave', onPointerLeave)
      host.removeEventListener('focusin', onFocusIn)
      host.removeEventListener('focusout', onFocusOut)
      stopSync()
    }
  },
})
