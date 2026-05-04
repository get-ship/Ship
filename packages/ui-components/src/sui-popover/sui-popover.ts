import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-popover.sui'

// Per-instance anchor-name so multiple popovers on a page each address
// their own trigger via CSS Anchor Positioning.
let anchorSeq = 0

const anchorSupported =
  typeof CSS !== 'undefined' &&
  typeof CSS.supports === 'function' &&
  CSS.supports('anchor-name', '--x')

type Placement =
  | 'top' | 'top-end'
  | 'bottom' | 'bottom-end'
  | 'left' | 'left-end'
  | 'right' | 'right-end'

interface PopoverEl extends HTMLElement {
  showPopover?: () => void
  hidePopover?: () => void
  togglePopover?: () => boolean
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const id = ++anchorSeq
    const anchorName = `--sui-popover-${id}`
    const panelId = `sui-popover-panel-${id}`

    host.style.setProperty('--sui-popover-anchor', anchorName)

    const triggerSlot = scope.triggerSlot()
    const panelEl = scope.panelEl() as PopoverEl | null
    if (!panelEl) return

    panelEl.id = panelId

    // ── Trigger wiring ──────────────────────────────────────────────────
    // The platform handles open/close: setting popovertarget on the
    // slotted button wires its click + keyboard activation directly to
    // the popover. We do NOT add a manual click handler — that would
    // double-toggle and steal focus management from the browser.
    // We DO set the ARIA attributes the popover attribute doesn't infer
    // (haspopup) and reflect aria-expanded from the toggle event.
    function wireTrigger() {
      const slot = triggerSlot as HTMLSlotElement | null
      if (!slot) return
      const els = slot.assignedElements({ flatten: true }) as HTMLElement[]
      for (const el of els) {
        // popovertarget is the native wiring. Setting it is enough; the
        // browser will toggle the panel on click/Enter/Space.
        el.setAttribute('popovertarget', panelId)
        el.setAttribute('aria-haspopup', 'dialog')
        el.setAttribute('aria-controls', panelId)
        el.setAttribute('aria-expanded', scope.open() ? 'true' : 'false')
      }
    }
    wireTrigger()
    triggerSlot?.addEventListener('slotchange', wireTrigger)

    // ── Native toggle event → reflect into scope.open + emit sui-toggle
    function onToggle(ev: Event) {
      const newState = (ev as ToggleEvent).newState
      const nowOpen = newState === 'open'
      // Reflect attribute so consumer-observable scope.open() flips.
      // Guard against feedback: we only act if the attribute disagrees.
      const hasAttr = host.hasAttribute('open')
      if (nowOpen && !hasAttr) host.setAttribute('open', '')
      else if (!nowOpen && hasAttr) host.removeAttribute('open')

      // Update aria-expanded on each trigger.
      const slot = triggerSlot as HTMLSlotElement | null
      const els = (slot?.assignedElements({ flatten: true }) ?? []) as HTMLElement[]
      for (const el of els) {
        el.setAttribute('aria-expanded', nowOpen ? 'true' : 'false')
      }

      host.dispatchEvent(new CustomEvent('sui-toggle', {
        detail: { open: nowOpen },
        bubbles: true,
        composed: true,
      }))

      if (nowOpen && !anchorSupported) positionFallback()
    }
    panelEl.addEventListener('toggle', onToggle)

    // ── Public methods (delegate to native) ────────────────────────────
    type PopoverHost = HTMLElement & {
      show: () => void
      hide: () => void
      toggle: () => void
    }
    ;(host as PopoverHost).show = () => panelEl.showPopover?.()
    ;(host as PopoverHost).hide = () => panelEl.hidePopover?.()
    ;(host as PopoverHost).toggle = () =>
      typeof panelEl.togglePopover === 'function'
        ? void panelEl.togglePopover()
        : (panelEl.matches?.(':popover-open')
            ? panelEl.hidePopover?.()
            : panelEl.showPopover?.())

    // ── Drive native popover from `open` prop ──────────────────────────
    // Consumer flips the attribute → effect calls show/hide. Native
    // light-dismiss flips the attribute via onToggle → effect sees the
    // values agree and no-ops. No feedback loop.
    const stopOpenSync = effect(() => {
      const wantOpen = scope.open()
      const isOpen = panelEl.matches?.(':popover-open') ?? false
      if (wantOpen && !isOpen) panelEl.showPopover?.()
      else if (!wantOpen && isOpen) panelEl.hidePopover?.()
    })

    // ── Position fallback for browsers without anchor-positioning ──────
    function positionFallback() {
      const slot = triggerSlot as HTMLSlotElement | null
      const els = (slot?.assignedElements({ flatten: true }) ?? []) as HTMLElement[]
      const trigger = els[0]
      if (!trigger || !panelEl) return
      const r = trigger.getBoundingClientRect()
      const placement = scope.placement() as Placement
      panelEl.style.top = ''
      panelEl.style.left = ''
      panelEl.style.right = ''
      panelEl.style.bottom = ''
      const gap = 6
      switch (placement) {
        case 'top':
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`
          panelEl.style.left = `${r.left}px`
          break
        case 'top-end':
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`
          panelEl.style.right = `${window.innerWidth - r.right}px`
          break
        case 'bottom-end':
          panelEl.style.top = `${r.bottom + gap}px`
          panelEl.style.right = `${window.innerWidth - r.right}px`
          break
        case 'left':
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`
          panelEl.style.top = `${r.top}px`
          break
        case 'left-end':
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`
          break
        case 'right':
          panelEl.style.left = `${r.right + gap}px`
          panelEl.style.top = `${r.top}px`
          break
        case 'right-end':
          panelEl.style.left = `${r.right + gap}px`
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`
          break
        case 'bottom':
        default:
          panelEl.style.top = `${r.bottom + gap}px`
          panelEl.style.left = `${r.left}px`
      }
    }

    function onWindowChange() {
      if (!scope.open()) return
      positionFallback()
    }
    if (!anchorSupported) {
      window.addEventListener('resize', onWindowChange)
      window.addEventListener('scroll', onWindowChange, { capture: true, passive: true })
    }

    return () => {
      triggerSlot?.removeEventListener('slotchange', wireTrigger)
      panelEl?.removeEventListener('toggle', onToggle)
      if (!anchorSupported) {
        window.removeEventListener('resize', onWindowChange)
        window.removeEventListener('scroll', onWindowChange, { capture: true } as EventListenerOptions)
      }
      stopOpenSync()
    }
  },
})
