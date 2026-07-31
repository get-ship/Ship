<controller>
import { controller, signal, ref } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-dock.sail'
import { ensureResizePillStyles } from './resize-pill-styles'

function ensureTerminalSlotFilled(host: HTMLElement): void {
  // If an outer surface already projects a terminal child, defer to it —
  // index.html / app shells may want to project a real engine elsewhere.
  const existing = host.querySelector(':scope > [slot="terminal"]')
  if (existing) return
  const term = document.createElement('ship-terminal')
  term.setAttribute('slot', 'terminal')
  term.setAttribute('fit', '')
  // Light-DOM child of the dock → projected into <slot name="terminal">.
  host.appendChild(term)
}

export interface DockTabDef {
  id: string
  label: string
  icon: string
  live?: boolean
  count?: number | null
  /** Short label rendered as a chip to the right of `count`. Used for
   *  "MOCK" / "EDIT" / etc. labels surfaced by view-authoring tabs. */
  badge?: string
}

const DEFAULT_TABS: DockTabDef[] = [
  { id: 'terminal', label: 'Terminal', icon: 'terminal', live: true },
  { id: 'source', label: 'Source', icon: 'file-code' },
  { id: 'signals', label: 'Signals', icon: 'radio', live: true },
  { id: 'components', label: 'Components', icon: 'package' },
]

const DEFAULT_HEIGHT = 240
const MIN_HEIGHT = 120

export default controller({
  scope: (state: ViewScope<Scope>, host?: Element) => {
    const tabs = signal<DockTabDef[]>(DEFAULT_TABS)
    const dragHeight = signal<number>(DEFAULT_HEIGHT)
    const resizeHandle = ref<HTMLElement>(null)
    const hostEl = host as HTMLElement | null

    // NB: `scope:` receives the RAW scope — prop entries are `Signal`s
    // (read `state.x()`, write `state.x.set(v)`). The unwrapped
    // `state.x = v` form exists only on the MutableState proxy that
    // `on:` handlers and `mount`/`unmount` receive.
    const writeStored = (suffix: string, val: string): void => {
      try { localStorage.setItem(`${state.storageKey()}-${suffix}`, val) }
      catch { /* best-effort */ }
    }

    function activate(id: string): void {
      if (!tabs().some((t) => t.id === id)) return
      if (state.activeTab() === id) return
      state.activeTab.set(id)
      writeStored('active', id)
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:activate', {
          detail: { tab: id }, bubbles: true, composed: true,
        }),
      )
    }
    function onPopout(): void {
      // Open the active tab's surface in a standalone window so users
      // can dock the source/signals/etc. alongside their editor. Each
      // tab declares its own URL via a known map; unknown tabs fall
      // back to a generic /sui/<tab> path.
      const tab = state.activeTab()
      const url = ({
        terminal: '/sui/terminal',
        source: '/studio/components',
        signals: '/sui/signals',
        schema: '/sui/schema',
        'va-source': '/studio/components',
        'va-components': '/studio/components',
        'va-activity': '/events',
        'va-compile': '/studio/components',
      } as Record<string, string>)[tab] ?? `/sui/${tab}`
      window.open(url, '_blank', 'noopener')
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:popout', {
          detail: { tab, url }, bubbles: true, composed: true,
        }),
      )
    }
    function onClose(): void {
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:close', { bubbles: true, composed: true }),
      )
    }
    function onEditModeToggle(): void {
      const next = !state.editModeOn()
      state.editModeOn.set(next)
      hostEl?.dispatchEvent(
        new CustomEvent('sui:dock:edit-mode-toggle', {
          detail: { on: next }, bubbles: true, composed: true,
        }),
      )
    }

    return {
      tabs, dragHeight, resizeHandle,
      activate, onPopout, onClose, onEditModeToggle,
    }
  },

  mount(state, host) {
    const tabsSig = state.$signals.tabs
    const activeTabSig = state.$signals.activeTab
    const resizingSig = state.$signals.resizing
    const dragHeightSig = state.$signals.dragHeight
    const storageKey = state.$signals.storageKey
    const activate = state.activate
    const resizeHandle = state.resizeHandle

    ensureResizePillStyles()
    ensureTerminalSlotFilled(host)

    const readStored = (suffix: string): string | null => {
      try { return localStorage.getItem(`${storageKey()}-${suffix}`) }
      catch { return null }
    }
    const writeStored = (suffix: string, val: string): void => {
      try { localStorage.setItem(`${storageKey()}-${suffix}`, val) }
      catch { /* best-effort */ }
    }

    const applyHeight = (px: number): void => {
      document.documentElement.style.setProperty('--ship-dock-height', `${px}px`)
      dragHeightSig.set(px)
    }
    const currentHeight = (): number => {
      const v = getComputedStyle(document.documentElement).getPropertyValue('--ship-dock-height')
      const n = Number.parseInt(v, 10)
      return Number.isFinite(n) ? n : DEFAULT_HEIGHT
    }

    // Hydrate persisted state.
    const lastActive = readStored('active')
    if (lastActive && tabsSig().some((t: DockTabDef) => t.id === lastActive)) {
      activeTabSig.set(lastActive)
    }
    const h = Number.parseInt(readStored('height') ?? '', 10)
    applyHeight(Number.isFinite(h) ? h : DEFAULT_HEIGHT)

    // Toolbar slot content tracking — adds .has-content when assigned.
    // {#for tabs()} re-renders the toolbar list when tabs change, so we
    // re-attach listeners after each render by observing shadowRoot.
    const sr = host.shadowRoot!
    const toolbarCleanups: Array<() => void> = []
    const trackToolbar = (tb: HTMLElement): void => {
      const slot = tb.querySelector<HTMLSlotElement>('slot')
      if (!slot) return
      const sync = (): void => {
        const has = slot.assignedElements({ flatten: true }).length > 0
        tb.classList.toggle('has-content', has)
      }
      slot.addEventListener('slotchange', sync)
      queueMicrotask(sync)
      toolbarCleanups.push(() => slot.removeEventListener('slotchange', sync))
    }
    for (const tb of sr.querySelectorAll<HTMLElement>('[data-toolbar]')) trackToolbar(tb)
    const mo = new MutationObserver((records) => {
      for (const r of records) {
        for (const n of r.addedNodes) {
          if (n.nodeType !== 1) continue
          const el = n as HTMLElement
          if (el.matches?.('[data-toolbar]')) trackToolbar(el)
        }
      }
    })
    mo.observe(sr, { childList: true, subtree: true })

    // Resize drag. `bind:this={resizeHandle}` targets the static `.resize`
    // div, present at mount; guard once so the dependent block narrows and
    // its teardown is registered only when the handle exists.
    const resizeCleanups: Array<() => void> = []
    if (resizeHandle) {
      const handle = resizeHandle
      const onPointerDown = (ev: PointerEvent): void => {
        ev.preventDefault()
        const startY = ev.clientY
        const startH = currentHeight()
        resizingSig.set(true)
        handle.setPointerCapture(ev.pointerId)
        const onMove = (e: PointerEvent): void => {
          const max = Math.round(window.innerHeight * 0.5)
          applyHeight(
            Math.min(max, Math.max(MIN_HEIGHT, startH + (startY - e.clientY))),
          )
        }
        const onUp = (e: PointerEvent): void => {
          resizingSig.set(false)
          handle.releasePointerCapture(e.pointerId)
          handle.removeEventListener('pointermove', onMove)
          handle.removeEventListener('pointerup', onUp)
          handle.removeEventListener('pointercancel', onUp)
          const final = currentHeight()
          writeStored('height', String(final))
          host.dispatchEvent(
            new CustomEvent('sui:dock:resize', {
              detail: { height: final }, bubbles: true, composed: true,
            }),
          )
        }
        handle.addEventListener('pointermove', onMove)
        handle.addEventListener('pointerup', onUp)
        handle.addEventListener('pointercancel', onUp)
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
      handle.addEventListener('pointerdown', onPointerDown)
      handle.addEventListener('dblclick', onDblClick)
      resizeCleanups.push(() => {
        handle.removeEventListener('pointerdown', onPointerDown)
        handle.removeEventListener('dblclick', onDblClick)
      })
    }

    // Imperative API parity with <ship-panel-dock>.
    Object.defineProperty(host, 'activate', { value: activate, configurable: true })
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
      mo.disconnect()
      for (const cleanup of resizeCleanups) cleanup()
    }
  },
})
</controller>

<!--
  <ship-dock> — @ship/chrome · the bottom panel dock. Tab strip across the top,
  body slot for the active tab, optional toolbar:* slot per tab, drag-to-resize
  handle on the top edge, and a teleported height pill during active drag.

  Prop / event / method parity
  ─────────────────────────────────────────────────────────────────────────────
  active-tab     (string, 'terminal', reflect) — active tab id
  storage-key    (string, 'ship-dock')         — localStorage prefix
  resizing       (bool, false, reflect)        — set during resize drag
  edit-mode-pill (bool, false, reflect)        — show edit-mode toggle pill
  edit-mode-on   (bool, false, reflect)        — pill ON visual state
  tabs           (JS property only)            — DockTabDef[]; overrides defaults
  ─────────────────────────────────────────────────────────────────────────────
  Events: sui:dock:activate {tab}, sui:dock:close, sui:dock:clear {tab},
          sui:dock:popout {tab}, sui:dock:resize {height},
          sui:dock:edit-mode-toggle {on}                     (bubbles, composed)
  ─────────────────────────────────────────────────────────────────────────────
  Methods: show(), hide(), toggle(), activate(tab)
  ─────────────────────────────────────────────────────────────────────────────
  Slots: <id> — body for tab id (terminal, source, signals, schema, components)
         toolbar:<id> — slim toolbar row for tab id (optional)

  Runtime-exceed notes
  ─────────────────────────────────────────────────────────────────────────────
  Tabstrip, toolbars, and body slots all render from a single `<for>` over
  the `tabs()` signal — the j2 Lit version only iterates the tabstrip and
  hard-codes the toolbar/body slot ids, so a custom `tabs` array silently
  drops projection. Here, setting `host.tabs` rebuilds the whole strip.

  Drag-height pill: while a resize drag is in progress, a small floating
  badge teleports to <body> via <sui-portal> and fades in via
  `transition:fade`. Lit had no comparable affordance — height changed
  silently during drag.
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="active-tab" type="string" default="terminal" reflect /> -->
<!-- <prop name="storage-key" type="string" default="ship-dock" /> -->
<!-- <prop name="resizing" type="bool" default="false" reflect /> -->
<!-- <prop name="edit-mode-pill" type="bool" default="false" reflect /> -->
<!-- <prop name="edit-mode-on" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div role="separator" aria-orientation="horizontal" aria-label="Resize panel" bind:this={resizeHandle}></div>
<div role="tablist">
  <for as="t" of={tabs()} key={t.id}>
    <button
      role="tab"
      type="button"
      aria-selected={t.id === activeTab() ? 'true' : 'false'}
      tabindex={t.id === activeTab() ? '0' : '-1'}
      data-tab={t.id}
      on:click={() => activate(t.id)}>
      <icon name={t.icon} size="sm"></icon>
      <span>{t.label}</span>
      <if cond={t.live && t.id === activeTab()}>
        <span aria-hidden="true"></span>
      </if>
      <if cond={t.count != null}>
        <span>{t.count.toLocaleString()}</span>
      </if>
      <if cond={t.badge}>
        <span>{t.badge}</span>
      </if>
    </button>
  </for>
  <span>
    <if cond={editModePill()}>
      <button
        type="button"
        role="switch"
        aria-checked={editModeOn() ? 'true' : 'false'}
        data-action="edit-mode-toggle"
        title={editModeOn() ? 'Exit edit mode' : 'Enter edit mode'}
        on:click={onEditModeToggle}>
        <icon name={editModeOn() ? 'pen-line' : 'pen'} size="sm"></icon>
        <span>{editModeOn() ? 'Editing' : 'Edit'}</span>
      </button>
    </if>
    <button type="button" aria-label="Pop out" title="Pop out" data-action="popout" on:click={onPopout}><icon name="maximize" size="sm"></icon></button>
    <button type="button" aria-label="Close panel" title="Close (⌘J)" data-action="close" on:click={onClose}><icon name="x" size="sm"></icon></button>
  </span>
</div>
<for as="t" of={tabs()} key={t.id}>
  <div data-toolbar={t.id} hidden={t.id !== activeTab()}>
    <slot name={'toolbar:' + t.id}></slot>
  </div>
</for>
<div>
  <for as="t" of={tabs()} key={t.id}>
    <slot name={t.id} data-active={t.id === activeTab()}></slot>
  </for>
</div>

<!--
  Drag-height pill: teleported to <body> so it escapes the dock's own
  stacking context and lives above any view content. Mounts only while
  the user is actively dragging the resize handle; `transition:fade`
  drives entry/exit.
-->
<sui-portal target="body">
  <if cond={resizing()}>
    <div transition:fade={{ duration: 120 }}>
      {dragHeight()}px
    </div>
  </if>
</sui-portal>
