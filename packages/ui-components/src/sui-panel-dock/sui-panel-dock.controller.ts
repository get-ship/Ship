import { controller, signal } from '@ship/ui-runtime'
import type { Signal } from '@ship/ui-runtime'
import type { Scope } from './sui-panel-dock.sui'

export interface DockTabDef {
  id: string
  label: string
  icon: string
  live?: boolean
  count?: number | null
}

const DEFAULT_TABS: DockTabDef[] = [
  { id: 'terminal', label: 'Terminal', icon: 'terminal', live: true },
  { id: 'source', label: 'Source', icon: 'file-code' },
  { id: 'signals', label: 'Signals', icon: 'radio', live: true },
  { id: 'lint', label: 'Lint', icon: 'list' },
  { id: 'schema', label: 'Schema', icon: 'network' },
  { id: 'components', label: 'Components', icon: 'package' },
]

const DEFAULT_HEIGHT = 240
const MIN_HEIGHT = 120

export default controller<Scope>({
  scope: (state, host) => {
    const tabs = signal<DockTabDef[]>(DEFAULT_TABS)
    const hostEl = host as HTMLElement | null
    const activeTabSig = (state as { activeTab: Signal<string> }).activeTab

    function activate(id: string): void {
      if (!tabs().some((t) => t.id === id)) return
      if (activeTabSig() === id) return
      activeTabSig.set(id)
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:activate', {
          detail: { tab: id }, bubbles: true, composed: true,
        }),
      )
    }
    function onClear(): void {
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:clear', {
          detail: { tab: activeTabSig() }, bubbles: true, composed: true,
        }),
      )
    }
    function onPopout(): void {
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:popout', {
          detail: { tab: activeTabSig() }, bubbles: true, composed: true,
        }),
      )
    }
    function onClose(): void {
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:close', { bubbles: true, composed: true }),
      )
    }

    return { tabs, activate, onClear, onPopout, onClose }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const tabsSig = (scope as unknown as { tabs: Signal<DockTabDef[]> }).tabs
    const activeTabSig = (scope as unknown as { activeTab: Signal<string> }).activeTab
    const resizingSig = (scope as unknown as { resizing: Signal<boolean> }).resizing
    const storageKeySig = (scope as unknown as { storageKey: () => string }).storageKey
    const activate = (scope as unknown as { activate: (id: string) => void }).activate
    const resizeHandle = (scope as unknown as { resizeHandle: () => HTMLElement }).resizeHandle()

    function readStored(suffix: string): string | null {
      try { return localStorage.getItem(`${storageKeySig()}-${suffix}`) }
      catch { return null }
    }
    function writeStored(suffix: string, val: string): void {
      try { localStorage.setItem(`${storageKeySig()}-${suffix}`, val) }
      catch { /* best-effort */ }
    }

    function applyHeight(px: number): void {
      document.documentElement.style.setProperty('--ship-dock-height', `${px}px`)
    }
    function currentHeight(): number {
      const v = getComputedStyle(document.documentElement).getPropertyValue('--ship-dock-height')
      const n = Number.parseInt(v, 10)
      return Number.isFinite(n) ? n : DEFAULT_HEIGHT
    }

    // Restore persisted state
    const lastActive = readStored('active')
    if (lastActive && DEFAULT_TABS.some((t) => t.id === lastActive)) {
      activeTabSig.set(lastActive)
    }
    const h = Number.parseInt(readStored('height') ?? '', 10)
    applyHeight(Number.isFinite(h) ? h : DEFAULT_HEIGHT)

    // Persist active tab on user-driven changes (via activate()).
    // We intercept the activate handler to write through.
    const wrappedActivate = (id: string): void => {
      const prev = activeTabSig()
      activate(id)
      if (activeTabSig() !== prev) writeStored('active', id)
    }

    // Toolbar slot content tracking — adds .has-content when assigned.
    const sr = host.shadowRoot!
    const toolbarCleanups: Array<() => void> = []
    for (const tb of sr.querySelectorAll<HTMLElement>('[data-toolbar]')) {
      const slot = tb.querySelector<HTMLSlotElement>('slot')
      if (!slot) continue
      const sync = (): void => {
        const has = slot.assignedElements({ flatten: true }).length > 0
        tb.classList.toggle('has-content', has)
      }
      slot.addEventListener('slotchange', sync)
      queueMicrotask(sync)
      toolbarCleanups.push(() => slot.removeEventListener('slotchange', sync))
    }

    // Resize drag
    const onPointerDown = (ev: PointerEvent): void => {
      ev.preventDefault()
      const startY = ev.clientY
      const startH = currentHeight()
      resizingSig.set(true)
      resizeHandle.setPointerCapture(ev.pointerId)
      const onMove = (e: PointerEvent): void => {
        const max = Math.round(window.innerHeight * 0.5)
        applyHeight(
          Math.min(max, Math.max(MIN_HEIGHT, startH + (startY - e.clientY))),
        )
      }
      const onUp = (e: PointerEvent): void => {
        resizingSig.set(false)
        resizeHandle.releasePointerCapture(e.pointerId)
        resizeHandle.removeEventListener('pointermove', onMove)
        resizeHandle.removeEventListener('pointerup', onUp)
        resizeHandle.removeEventListener('pointercancel', onUp)
        const final = currentHeight()
        writeStored('height', String(final))
        host.dispatchEvent(
          new CustomEvent('sui:dock:resize', {
            detail: { height: final }, bubbles: true, composed: true,
          }),
        )
      }
      resizeHandle.addEventListener('pointermove', onMove)
      resizeHandle.addEventListener('pointerup', onUp)
      resizeHandle.addEventListener('pointercancel', onUp)
    }
    const onDblClick = (): void => {
      applyHeight(DEFAULT_HEIGHT)
      writeStored('height', String(DEFAULT_HEIGHT))
      host.dispatchEvent(
        new CustomEvent('sui:dock:resize', {
          detail: { height: DEFAULT_HEIGHT }, bubbles: true, composed: true,
        }),
      )
    }
    resizeHandle.addEventListener('pointerdown', onPointerDown)
    resizeHandle.addEventListener('dblclick', onDblClick)

    // Public imperative API
    Object.defineProperty(host, 'activate', { value: wrappedActivate, configurable: true })
    Object.defineProperty(host, 'show', {
      value: () => document.body.classList.add('dock-open'), configurable: true,
    })
    Object.defineProperty(host, 'hide', {
      value: () => document.body.classList.remove('dock-open'), configurable: true,
    })
    Object.defineProperty(host, 'toggle', {
      value: () => document.body.classList.toggle('dock-open'), configurable: true,
    })
    Object.defineProperty(host, 'tabs', {
      get: () => tabsSig(),
      set: (val: DockTabDef[]) => tabsSig.set(val),
      configurable: true,
    })

    return () => {
      for (const cleanup of toolbarCleanups) cleanup()
      resizeHandle.removeEventListener('pointerdown', onPointerDown)
      resizeHandle.removeEventListener('dblclick', onDblClick)
    }
  },
})
