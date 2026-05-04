import { controller, signal, effect } from '@ship/ui-runtime'
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

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

function tabHtml(t: DockTabDef, active: boolean): string {
  const liveDot = t.live && active ? '<span class="live-dot" aria-hidden="true"></span>' : ''
  const badge = t.count != null ? `<span class="count">${t.count.toLocaleString()}</span>` : ''
  return `<button class="tab" role="tab" type="button" aria-selected="${active ? 'true' : 'false'}" tabindex="${active ? '0' : '-1'}" data-tab="${esc(t.id)}"><sui-icon name="${esc(t.icon)}" size="sm"></sui-icon><span>${esc(t.label)}</span>${liveDot}${badge}</button>`
}

const TOOLS_HTML = `<span class="tools">
  <button class="tool" type="button" aria-label="Clear" title="Clear" data-action="clear"><sui-icon name="refresh-ccw" size="sm"></sui-icon></button>
  <button class="tool" type="button" aria-label="Pop out" title="Pop out" data-action="popout"><sui-icon name="maximize-2" size="sm"></sui-icon></button>
  <button class="tool" type="button" aria-label="Close panel" title="Close (⌘J)" data-action="close"><sui-icon name="x" size="sm"></sui-icon></button>
</span>`

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const tabstrip = scope.tabstrip() as HTMLElement
    const bodies = scope.bodies() as HTMLElement
    const resizeHandle = scope.resizeHandle() as HTMLElement

    const tabs = signal<DockTabDef[]>(DEFAULT_TABS)

    function readStored(suffix: string): string | null {
      try { return localStorage.getItem(`${scope.storageKey()}-${suffix}`) } catch { return null }
    }
    function writeStored(suffix: string, val: string): void {
      try { localStorage.setItem(`${scope.storageKey()}-${suffix}`, val) } catch { /* best-effort */ }
    }

    function applyHeight(px: number): void {
      document.documentElement.style.setProperty('--ship-dock-height', `${px}px`)
    }
    function currentHeight(): number {
      const v = getComputedStyle(document.documentElement).getPropertyValue('--ship-dock-height')
      const n = Number.parseInt(v, 10)
      return Number.isFinite(n) ? n : DEFAULT_HEIGHT
    }

    function buildTabstrip(): void {
      const active = scope.activeTab()
      tabstrip.innerHTML = tabs().map(t => tabHtml(t, t.id === active)).join('') + TOOLS_HTML
    }

    function activate(id: string): void {
      if (!tabs().some(t => t.id === id)) return
      if (scope.activeTab() === id) return
      scope.activeTab.set(id)
      writeStored('active', id)
      host.dispatchEvent(new CustomEvent('sui:dock:activate', { detail: { tab: id }, bubbles: true, composed: true }))
    }

    // Initial build
    buildTabstrip()

    // Restore persisted state
    const lastActive = readStored('active')
    if (lastActive && DEFAULT_TABS.some(t => t.id === lastActive)) scope.activeTab.set(lastActive)
    const h = Number.parseInt(readStored('height') ?? '', 10)
    applyHeight(Number.isFinite(h) ? h : DEFAULT_HEIGHT)

    // Reactive: sync DOM on active-tab change
    const stopSync = effect(() => {
      const active = scope.activeTab()

      // Update tab buttons (aria-selected, tabindex, live-dot)
      for (const btn of tabstrip.querySelectorAll<HTMLElement>('[data-tab]')) {
        const isActive = btn.dataset.tab === active
        btn.setAttribute('aria-selected', isActive ? 'true' : 'false')
        btn.setAttribute('tabindex', isActive ? '0' : '-1')
        const tabDef = tabs().find(t => t.id === btn.dataset.tab)
        const existing = btn.querySelector('.live-dot')
        if (tabDef?.live && isActive && !existing) {
          btn.insertAdjacentHTML('beforeend', '<span class="live-dot" aria-hidden="true"></span>')
        } else if ((!tabDef?.live || !isActive) && existing) {
          existing.remove()
        }
      }

      // Body slot visibility
      for (const slot of bodies.querySelectorAll<HTMLSlotElement>('slot[name]')) {
        if (slot.getAttribute('name') === active) slot.setAttribute('data-active', '')
        else slot.removeAttribute('data-active')
      }

      // Toolbar visibility
      const sr = host.shadowRoot!
      for (const tb of sr.querySelectorAll<HTMLElement>('[data-toolbar]')) {
        tb.hidden = tb.dataset.toolbar !== active
      }
    })

    // Tab strip click delegation
    const onStripClick = (e: MouseEvent) => {
      const btn = (e.target as Element).closest<HTMLElement>('[data-tab]')
      if (btn) { activate(btn.dataset.tab!); return }
      const action = (e.target as Element).closest<HTMLElement>('[data-action]')
      if (!action) return
      const act = action.dataset.action
      if (act === 'clear') {
        host.dispatchEvent(new CustomEvent('sui:dock:clear', { detail: { tab: scope.activeTab() }, bubbles: true, composed: true }))
      } else if (act === 'popout') {
        host.dispatchEvent(new CustomEvent('sui:dock:popout', { detail: { tab: scope.activeTab() }, bubbles: true, composed: true }))
      } else if (act === 'close') {
        host.dispatchEvent(new CustomEvent('sui:dock:close', { bubbles: true, composed: true }))
      }
    }
    tabstrip.addEventListener('click', onStripClick)

    // Toolbar slot content tracking
    const sr = host.shadowRoot!
    for (const tb of sr.querySelectorAll<HTMLElement>('[data-toolbar]')) {
      const slot = tb.querySelector<HTMLSlotElement>('slot')
      if (!slot) continue
      const sync = () => {
        const has = slot.assignedElements({ flatten: true }).length > 0
        tb.classList.toggle('has-content', has)
      }
      slot.addEventListener('slotchange', sync)
      queueMicrotask(sync)
    }

    // Resize drag
    resizeHandle.addEventListener('pointerdown', (ev: PointerEvent) => {
      ev.preventDefault()
      const startY = ev.clientY, startH = currentHeight()
      scope.resizing.set(true)
      resizeHandle.setPointerCapture(ev.pointerId)
      const onMove = (e: PointerEvent) => {
        const max = Math.round(window.innerHeight * 0.5)
        applyHeight(Math.min(max, Math.max(MIN_HEIGHT, startH + (startY - e.clientY))))
      }
      const onUp = (e: PointerEvent) => {
        scope.resizing.set(false)
        resizeHandle.releasePointerCapture(e.pointerId)
        resizeHandle.removeEventListener('pointermove', onMove)
        resizeHandle.removeEventListener('pointerup', onUp)
        resizeHandle.removeEventListener('pointercancel', onUp)
        const final = currentHeight()
        writeStored('height', String(final))
        host.dispatchEvent(new CustomEvent('sui:dock:resize', { detail: { height: final }, bubbles: true, composed: true }))
      }
      resizeHandle.addEventListener('pointermove', onMove)
      resizeHandle.addEventListener('pointerup', onUp)
      resizeHandle.addEventListener('pointercancel', onUp)
    })

    resizeHandle.addEventListener('dblclick', () => {
      applyHeight(DEFAULT_HEIGHT)
      writeStored('height', String(DEFAULT_HEIGHT))
      host.dispatchEvent(new CustomEvent('sui:dock:resize', { detail: { height: DEFAULT_HEIGHT }, bubbles: true, composed: true }))
    })

    // Public imperative API
    Object.defineProperty(host, 'activate', { value: activate, configurable: true })
    Object.defineProperty(host, 'show', { value: () => document.body.classList.add('dock-open'), configurable: true })
    Object.defineProperty(host, 'hide', { value: () => document.body.classList.remove('dock-open'), configurable: true })
    Object.defineProperty(host, 'toggle', { value: () => document.body.classList.toggle('dock-open'), configurable: true })
    Object.defineProperty(host, 'tabs', {
      get: () => tabs(),
      set: (val: DockTabDef[]) => { tabs.set(val); buildTabstrip() },
      configurable: true,
    })

    return () => {
      stopSync()
      tabstrip.removeEventListener('click', onStripClick)
    }
  },
})
