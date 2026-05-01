// Paired controller for `<set-button>`.
//
// The template (`set-button.sui`) declares props, scoped styles, and
// the `<button>` content with slots. This controller adds the one piece
// that can't live in declarative template-only — auto-detecting an empty
// default slot so the host element collapses to a square icon-only
// button (the `:host([data-icon-only])` CSS rule fires off this attr).
//
// Mirrors the slotchange logic from the Lit `<ship-button>` original
// (packages/ship-set/src/web-components/core/ship-button.ts).

import type { ComponentSetupContext } from '@ship/ui-runtime'

export default function setup({ host, cleanups }: ComponentSetupContext): void {
  // The compiler calls paired setup BEFORE rendering the template, so the
  // shadow root is empty here. Defer to the next microtask so the
  // <button> + <slot> are in place before we query.
  let slot: HTMLSlotElement | null = null

  const update = (): void => {
    if (!slot) return
    const nodes = slot.assignedNodes({ flatten: true })
    const hasText = nodes.some((n) => {
      if (n.nodeType === Node.TEXT_NODE) {
        return (n.textContent || '').trim().length > 0
      }
      if (n.nodeType === Node.ELEMENT_NODE) {
        const el = n as Element
        return (
          (el.textContent || '').trim().length > 0 ||
          el.childElementCount > 0
        )
      }
      return false
    })
    if (hasText) host.removeAttribute('data-icon-only')
    else host.setAttribute('data-icon-only', '')
  }

  queueMicrotask(() => {
    const root = host.shadowRoot ?? host
    slot = root.querySelector('slot:not([name])') as HTMLSlotElement | null
    if (!slot) return
    slot.addEventListener('slotchange', update)
    // Initial check — assigned nodes may already be in place at connect
    // time without firing a slotchange event.
    update()
  })

  cleanups.push(() => {
    if (slot) slot.removeEventListener('slotchange', update)
  })
}
