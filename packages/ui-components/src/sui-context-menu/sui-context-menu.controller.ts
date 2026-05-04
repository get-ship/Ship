import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-context-menu.sui'

let _seq = 0

type PopoverEl = HTMLElement & {
  showPopover?: () => void
  hidePopover?: () => void
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const panelId = `sui-context-menu-panel-${++_seq}`

    const triggerSlot = scope.triggerSlot() as HTMLSlotElement | null
    const panelEl = scope.panelEl() as PopoverEl | null

    if (!panelEl) return
    panelEl.id = panelId

    // ── Typeahead state ──────────────────────────────────────────────────
    let typeaheadBuf = ''
    let typeaheadTimer: ReturnType<typeof setTimeout> | null = null

    function pushTypeahead(key: string): number {
      if (typeaheadTimer) clearTimeout(typeaheadTimer)
      typeaheadBuf += key.toLowerCase()
      typeaheadTimer = setTimeout(() => { typeaheadBuf = '' }, 350)
      const items = getItems()
      const i = items.indexOf(document.activeElement as HTMLElement)
      for (let j = 0; j < items.length; j++) {
        const idx = (i + 1 + j) % items.length
        const label = (items[idx].textContent ?? '').trim().toLowerCase()
        if (label.startsWith(typeaheadBuf)) return idx
      }
      return -1
    }

    // ── Item management ──────────────────────────────────────────────────
    function getItems(): HTMLElement[] {
      return Array.from(
        host.querySelectorAll<HTMLElement>('[role="menuitem"], button[data-value], [data-value]'),
      ).filter(el => !el.hasAttribute('disabled'))
    }

    function focusItem(index: number) {
      const items = getItems()
      if (!items.length) return
      const i = ((index % items.length) + items.length) % items.length
      for (const it of items) it.setAttribute('tabindex', '-1')
      items[i].setAttribute('tabindex', '0')
      items[i].focus()
    }

    function currentIndex(): number {
      return getItems().indexOf(document.activeElement as HTMLElement)
    }

    function setupItems() {
      getItems().forEach((el, idx) => {
        if (!el.hasAttribute('role')) el.setAttribute('role', 'menuitem')
        if (!el.hasAttribute('tabindex')) el.setAttribute('tabindex', idx === 0 ? '0' : '-1')
      })
    }

    // ── Public API ───────────────────────────────────────────────────────
    function doShow(x: number, y: number) {
      if (panelEl) {
        panelEl.style.top = `${y}px`
        panelEl.style.left = `${x}px`
      }
      panelEl?.showPopover?.()
    }

    function doClose() {
      panelEl?.hidePopover?.()
    }

    ;(host as HTMLElement & { show(x: number, y: number): void }).show = doShow
    ;(host as HTMLElement & { close(): void }).close = doClose

    // ── Events ───────────────────────────────────────────────────────────
    const onContextMenu = (e: MouseEvent) => {
      e.preventDefault()
      doShow(e.clientX, e.clientY)
    }

    const onToggle = (ev: Event) => {
      const te = ev as ToggleEvent
      const nowOpen = te.newState === 'open'
      if (nowOpen !== scope.open()) scope.open.set(nowOpen)
      host.setAttribute('data-state', nowOpen ? 'open' : 'closed')
      if (nowOpen) {
        host.dispatchEvent(new CustomEvent('sui-open', { bubbles: true, composed: true }))
        queueMicrotask(() => { setupItems(); focusItem(0) })
      } else {
        host.dispatchEvent(new CustomEvent('sui-close', { bubbles: true, composed: true }))
      }
    }

    const onItemClick = (e: Event) => {
      const target = (e.target as HTMLElement).closest('[data-value]') as HTMLElement | null
      if (!target) return
      host.dispatchEvent(new CustomEvent('sui-select', {
        detail: { value: target.dataset.value ?? '' },
        bubbles: true, composed: true,
      }))
      doClose()
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (!scope.open()) return
      const items = getItems()
      if (!items.length) return
      const i = currentIndex()
      if (e.key === 'ArrowDown') { e.preventDefault(); focusItem(i + 1) }
      else if (e.key === 'ArrowUp') { e.preventDefault(); focusItem(i - 1) }
      else if (e.key === 'Home') { e.preventDefault(); focusItem(0) }
      else if (e.key === 'End') { e.preventDefault(); focusItem(items.length - 1) }
      else if (e.key === 'Escape') { e.preventDefault(); doClose() }
      else if (e.key === 'Tab') { doClose() }
      else if ((e.key === 'Enter' || e.key === ' ') && i >= 0) { e.preventDefault(); items[i].click() }
      else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        const next = pushTypeahead(e.key)
        if (next >= 0) { e.preventDefault(); focusItem(next) }
      }
    }

    const wireTrigger = () => {
      const els = triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[] ?? []
      for (const el of els) {
        el.addEventListener('contextmenu', onContextMenu)
      }
    }

    triggerSlot?.addEventListener('slotchange', wireTrigger)
    wireTrigger()
    panelEl.addEventListener('toggle', onToggle)
    panelEl.addEventListener('click', onItemClick)
    host.addEventListener('keydown', onKeydown)

    const stopSync = effect(() => {
      host.setAttribute('data-state', scope.open() ? 'open' : 'closed')
    })

    return () => {
      const els = triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[] ?? []
      for (const el of els) el.removeEventListener('contextmenu', onContextMenu)
      triggerSlot?.removeEventListener('slotchange', wireTrigger)
      panelEl.removeEventListener('toggle', onToggle)
      panelEl.removeEventListener('click', onItemClick)
      host.removeEventListener('keydown', onKeydown)
      if (typeaheadTimer) clearTimeout(typeaheadTimer)
      stopSync()
    }
  },
})
