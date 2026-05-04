import { controller, effect, signal } from '@ship/ui-runtime'
import type { Scope } from './sui-tabs.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // Version counter bumped on slotchange to force the sync effect to re-run.
    const slotVersion = signal(0)

    const getTabs = (): Element[] => {
      const slot = scope.tabSlot()
      if (!slot) return Array.from(host.querySelectorAll(':scope > sui-tab'))
      return (slot as HTMLSlotElement).assignedElements()
        .filter(el => el.tagName.toLowerCase() === 'sui-tab')
    }

    const activate = (name: string) => {
      if (scope.active() === name) return
      scope.active.set(name)
      host.dispatchEvent(new CustomEvent('sui-tab-change', {
        detail: { name }, bubbles: true, composed: true,
      }))
    }

    // Sync active + aria-controls state onto <sui-tab> and panel children.
    // Runs reactively on active changes and on slotchange (via slotVersion).
    const stopSync = effect(() => {
      slotVersion()  // track so effect re-runs on slot mutations
      const activeName = scope.active()
      const tabs = getTabs()

      // Default: first tab active when active is unset and tabs exist.
      let resolved = activeName
      if (!resolved && tabs.length > 0) {
        resolved = tabs[0]!.getAttribute('name') ?? ''
        if (resolved) scope.active.set(resolved)
      }

      for (const tab of tabs) {
        const name = tab.getAttribute('name') ?? ''
        if (name === resolved) {
          tab.setAttribute('active', '')
        } else {
          tab.removeAttribute('active')
        }
        tab.id = `sui-tab-${name}`
        tab.setAttribute('aria-controls', `sui-tabpanel-${name}`)
      }

      // Sync any sui-tab-panel or role=tabpanel children.
      for (const panel of Array.from(
        host.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]'),
      )) {
        const name = panel.getAttribute('name') ?? ''
        if (name === resolved) {
          panel.removeAttribute('hidden')
          panel.setAttribute('active', '')
        } else {
          panel.setAttribute('hidden', '')
          panel.removeAttribute('active')
        }
      }
    })

    const onKeydown = (e: KeyboardEvent) => {
      const vertical = scope.orientation() === 'vertical'
      const nextKey = vertical ? 'ArrowDown' : 'ArrowRight'
      const prevKey = vertical ? 'ArrowUp' : 'ArrowLeft'
      const navKeys = [nextKey, prevKey, 'Home', 'End']
      const activateKeys = ['Enter', ' ']

      if (!navKeys.includes(e.key) && !activateKeys.includes(e.key)) return
      e.preventDefault()

      const tabs = getTabs().filter(t => !t.hasAttribute('disabled'))
      if (!tabs.length) return

      const names = tabs.map(t => t.getAttribute('name') ?? '')
      const current = scope.active()
      const i = Math.max(0, names.indexOf(current))
      let next = i

      if (e.key === nextKey) next = (i + 1) % names.length
      else if (e.key === prevKey) next = (i - 1 + names.length) % names.length
      else if (e.key === 'Home') next = 0
      else if (e.key === 'End') next = names.length - 1

      const targetName = names[next]!
      activate(targetName)

      queueMicrotask(() => {
        const targetTab = tabs[next] as HTMLElement
        const btn = targetTab.shadowRoot?.querySelector<HTMLElement>('button')
        btn ? btn.focus() : targetTab.focus()
      })
    }

    const onClick = (e: MouseEvent) => {
      const target = (e.target as Element).closest('sui-tab')
      if (!target || target.hasAttribute('disabled')) return
      const name = target.getAttribute('name') ?? ''
      if (name) activate(name)
    }

    const onSlotChange = () => slotVersion.set(slotVersion() + 1)

    const strip = scope.strip()
    const tabSlot = scope.tabSlot()

    if (strip) {
      strip.addEventListener('keydown', onKeydown)
      strip.addEventListener('click', onClick)
    }
    if (tabSlot) {
      tabSlot.addEventListener('slotchange', onSlotChange)
    }

    return () => {
      stopSync()
      if (strip) {
        strip.removeEventListener('keydown', onKeydown)
        strip.removeEventListener('click', onClick)
      }
      if (tabSlot) {
        tabSlot.removeEventListener('slotchange', onSlotChange)
      }
    }
  },
})
