import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-error-state.sui'

// Default back/retry handler. The hub routes 404 + boundary recovery
// through this surface; clicks on the primary action either navigate
// to `href` (rendered as an anchor) or call history.back() and emit
// `ship-error-state:retry` for hosts that want a custom recovery path
// in addition to (or instead of) the history navigation.
export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>, host) => {
    const goBack = (_e: Event): void => {
      host?.dispatchEvent(
        new CustomEvent('ship-error-state:retry', {
          bubbles: true,
          composed: true,
        }),
      )
      try {
        history.back()
      } catch {
        // history.back throws when there's no prior entry (deep-link).
        // Fall back to the app root — the chrome's home link still
        // works because the hub's router will SPA-navigate to "/".
        location.assign('/')
      }
    }

    // When the action slot is populated, the slot content renders
    // alongside the default back button. The default back button can
    // be suppressed by the host (set `action-label=""` and the host
    // is in charge). Here we only set the `data-action-from-slot`
    // marker when the slot has *any* assigned content AND the host
    // has explicitly opted in (action-label is empty).
    const onActionSlot = (): void => {
      if (!host) return
      const slot = host.shadowRoot?.querySelector(
        'slot[name="action"]',
      ) as HTMLSlotElement | null
      if (!slot) return
      const has = slot.assignedNodes({ flatten: true }).some(n =>
        n.nodeType === Node.ELEMENT_NODE ||
        (n.nodeType === Node.TEXT_NODE && (n.textContent ?? '').trim().length > 0),
      )
      if (has) host.setAttribute('data-action-from-slot', '')
      else host.removeAttribute('data-action-from-slot')
    }
    queueMicrotask(onActionSlot)

    return { goBack, onActionSlot }
  },

  mount(_state, host) {
    // Wire the slotchange listener so `data-action-from-slot` tracks
    // slot updates (the queueMicrotask call in scope() only handles
    // the initial render). Hosts that swap the action slot after
    // mount would otherwise leave the marker stale.
    const slot = host.shadowRoot?.querySelector(
      'slot[name="action"]',
    ) as HTMLSlotElement | null
    if (slot) {
      const update = (): void => {
        const has = slot.assignedNodes({ flatten: true }).some(n =>
          n.nodeType === Node.ELEMENT_NODE ||
          (n.nodeType === Node.TEXT_NODE && (n.textContent ?? '').trim().length > 0),
        )
        if (has) host.setAttribute('data-action-from-slot', '')
        else host.removeAttribute('data-action-from-slot')
      }
      slot.addEventListener('slotchange', update)
      update()
    }
  },
})
