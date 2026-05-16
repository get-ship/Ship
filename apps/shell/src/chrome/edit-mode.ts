// chrome/edit-mode.ts — wires the per-view edit-mode signal into
// the shell's dev-panel chrome.
//
// Responsibilities
//   - Watch `editModeForView(currentView)`. When it flips:
//       on:  load the view-authoring island bundle, project the VA tab
//            content into the panel-dock, swap the dock's `tabs` to the
//            VA tab set, set `edit-mode-pill` + `edit-mode-on`, force
//            the dock open, and mount the annotation-pins overlay.
//       off: restore the default dock tabs + content, hide the pins.
//   - Listen for `sui:dock:edit-mode-toggle` from the pill click and
//     call `toggleEditMode(currentView)` so the persisted signal stays
//     in sync.
//   - Listen for `sui:edit-mode:change` events fired by the runtime
//     module (e.g. from a view component) and reconcile state.
//   - Track `window.__suiCurrentViewId`-style view-name changes so
//     each route mounts under the right (project, view) key.

import {
  editModeForView,
  toggleEditMode,
  setEditMode,
  effect,
  type Signal,
} from '@ship/sui-runtime'

const VA_ISLAND_URL = '/_sui/sui-components/sui-view-authoring.js'
const PICKER_ISLAND_URL = '/_sui/sui-components/sui-component-picker.js'

interface DockTabDef {
  id: string
  label: string
  icon: string
  badge?: string
  live?: boolean
  count?: number | null
}

interface PanelDock extends HTMLElement {
  show(): void
  hide(): void
  toggle(): void
  activate(tab: string): void
  tabs?: DockTabDef[]
}

const VA_TABS: DockTabDef[] = [
  { id: 'va-source',     label: 'Source',     icon: 'file-code', badge: 'EDIT' },
  { id: 'va-components', label: 'Components', icon: 'package' },
  { id: 'va-activity',   label: 'Activity',   icon: 'activity', live: true },
  { id: 'va-compile',    label: 'Compile',    icon: 'check-circle-2' },
  { id: 'terminal',      label: 'Terminal',   icon: 'terminal' },
]

const DEFAULT_TABS: DockTabDef[] = [
  { id: 'terminal',   label: 'Terminal',   icon: 'terminal', live: true },
  { id: 'source',     label: 'Source',     icon: 'file-code' },
  { id: 'signals',    label: 'Signals',    icon: 'radio', live: true },
  { id: 'lint',       label: 'Lint',       icon: 'list' },
  { id: 'schema',     label: 'Schema',     icon: 'network' },
  { id: 'components', label: 'Components', icon: 'package' },
]

interface VaWindow extends Window {
  __suiCurrentViewId?: string | null
}

let currentView: string | null = null
let activeStopEffect: (() => void) | null = null
let lastApplied: boolean | null = null

function ensureIslandLoaded(src: string, marker: string): void {
  if (document.querySelector(`script[data-sui-island="${marker}"]`)) return
  const s = document.createElement('script')
  s.type = 'module'
  s.src = src
  s.dataset['suiIsland'] = marker
  document.head.appendChild(s)
}

function getDock(): PanelDock | null {
  return document.getElementById('ship-panel-dock') as PanelDock | null
}

function getPinLayer(): HTMLElement | null {
  return document.getElementById('sui-va-pins')
}

function ensurePinLayer(): HTMLElement {
  let el = getPinLayer()
  if (el) return el
  el = document.createElement('sui-va-annotation-pins')
  el.id = 'sui-va-pins'
  document.body.appendChild(el)
  return el
}

function ensureVaSlots(view: string): void {
  const dock = getDock()
  if (!dock) return
  for (const tab of VA_TABS) {
    if (tab.id === 'terminal') continue
    if (dock.querySelector(`:scope > [slot="${tab.id}"]`)) {
      const existing = dock.querySelector(`:scope > [slot="${tab.id}"]`) as HTMLElement
      existing.setAttribute('view', view)
      continue
    }
    const tagFor: Record<string, string> = {
      'va-source':     'sui-va-source',
      'va-components': 'sui-va-components',
      'va-activity':   'sui-va-activity',
      'va-compile':    'sui-va-compile',
    }
    const tag = tagFor[tab.id]
    if (!tag) continue
    const el = document.createElement(tag)
    el.setAttribute('slot', tab.id)
    el.setAttribute('view', view)
    el.setAttribute('data-va-slot', '')
    dock.appendChild(el)
  }
}

function removeVaSlots(): void {
  const dock = getDock()
  if (!dock) return
  for (const el of dock.querySelectorAll(':scope > [data-va-slot]')) {
    el.remove()
  }
}

function applyEditMode(on: boolean): void {
  if (lastApplied === on) return
  lastApplied = on
  const dock = getDock()
  if (!dock) return

  if (on) {
    ensureIslandLoaded(PICKER_ISLAND_URL, 'sui-component-picker')
    ensureIslandLoaded(VA_ISLAND_URL, 'sui-view-authoring')
    ensureVaSlots(currentView ?? '')
    dock.setAttribute('edit-mode-pill', '')
    dock.setAttribute('edit-mode-on', '')
    dock.tabs = VA_TABS
    // Activate Source tab by default so the user lands in the editor.
    dock.activate('va-source')
    dock.show()
    document.body.classList.add('edit-mode')
    ensurePinLayer().removeAttribute('hidden')
  } else {
    dock.removeAttribute('edit-mode-on')
    // Keep the pill visible so the user can toggle back on; only hide
    // it when no view is active.
    if (!currentView) dock.removeAttribute('edit-mode-pill')
    dock.tabs = DEFAULT_TABS
    removeVaSlots()
    document.body.classList.remove('edit-mode')
    const pins = getPinLayer()
    if (pins) pins.setAttribute('hidden', '')
  }
}

function subscribeFor(view: string): void {
  activeStopEffect?.()
  lastApplied = null
  const sig: Signal<boolean> = editModeForView(view)
  activeStopEffect = effect(() => {
    applyEditMode(sig())
  })
}

function onPillToggle(e: Event): void {
  // The dock dispatches `sui:dock:edit-mode-toggle` with { on } from
  // its pill click. Sync with the persisted signal — the effect above
  // will pick up the change and re-apply.
  if (!currentView) return
  const detail = (e as CustomEvent).detail as { on?: boolean } | undefined
  if (typeof detail?.on === 'boolean') {
    setEditMode(currentView, detail.on)
  } else {
    toggleEditMode(currentView)
  }
}

function onEditModeChange(e: Event): void {
  // Forwarded by edit-mode.ts whenever any caller flips state — used
  // to bring the chrome in sync when toggled from outside the dock.
  const d = (e as CustomEvent).detail as { view?: string; on?: boolean } | undefined
  if (!d || d.view !== currentView) return
  if (typeof d.on === 'boolean') applyEditMode(d.on)
}

/** Called from main.ts on every successful mountRoute. Switches the
 *  dock's per-view subscription to `view`. Empty string clears it. */
export function setEditModeView(view: string | null): void {
  if (view === currentView) return
  // Tear down previous view's chrome.
  if (currentView) applyEditMode(false)
  currentView = view
  if (!view) {
    activeStopEffect?.()
    activeStopEffect = null
    return
  }
  subscribeFor(view)
  // Auto-on for views that come up while a scaffolded session is
  // active. The signal's persisted value already encodes that; this
  // hook just publishes the view name to chrome.
  const w = window as VaWindow
  if (w.__suiCurrentViewId) w.__suiCurrentViewId = w.__suiCurrentViewId
}

export function initEditModeChrome(): void {
  document.addEventListener('sui:dock:edit-mode-toggle', onPillToggle)
  document.addEventListener('sui:edit-mode:change', onEditModeChange)
}

// Re-export for keyboard handler in main.ts.
export { toggleEditMode }
