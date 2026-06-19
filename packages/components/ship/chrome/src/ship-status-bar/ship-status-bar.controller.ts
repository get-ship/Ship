// Status-bar controller — wires the chrome bottom strip.
// Migrated from apps/shell/src/components/sui-status-bar; the launch-chrome
// surface (branch/theme/online/agents/mode/project props, status slot, click
// intents) is the source of truth.
//
// Responsibilities:
//   1. Click handlers for the project, daemon-dot, and theme buttons —
//      each dispatches a composed CustomEvent the shell listens for.
//   2. A view-extensible status zone driven by `chrome.status.set(...)`
//      (the runtime emits sui:status:set / sui:status:clear). Entries
//      survive across re-mounts of the host because the controller
//      subscribes at the document level and re-renders from the live map.
//   3. Sets host a11y attributes (role=contentinfo) so screen-reader
//      navigation announces the bar.

import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-status-bar.sui'

interface StatusValue {
  label: string
  icon?: string
  tone?: 'default' | 'info' | 'success' | 'warning' | 'error'
  tooltip?: string
}

interface ExtensionEntry extends StatusValue {
  key: string
}

interface SetDetail { key?: string; value?: StatusValue }
interface ClearDetail { key?: string }

function dispatch(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail: detail ?? null,
    bubbles: true,
    composed: true,
  }))
}

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const extensions = signal<ExtensionEntry[]>([])
    const hostEl = host as HTMLElement

    function onDotClick(): void {
      dispatch(hostEl, 'sui:dock:toggle')
    }
    function onProjectClick(): void {
      dispatch(hostEl, 'sui:project-switcher:open')
    }
    function onThemeClick(): void {
      dispatch(hostEl, 'sui:theme-picker:open')
    }

    return { extensions, onDotClick, onProjectClick, onThemeClick }
  },

  mount(state, host) {
    const hostEl = host as HTMLElement

    if (!hostEl.hasAttribute('role')) hostEl.setAttribute('role', 'contentinfo')
    if (!hostEl.hasAttribute('aria-label')) hostEl.setAttribute('aria-label', 'Status bar')

    const entries = new Map<string, ExtensionEntry>()
    const repaint = (): void => {
      state.$signals.extensions.set(Array.from(entries.values()))
    }

    const onSet = (e: Event): void => {
      const d = ((e as CustomEvent).detail ?? {}) as SetDetail
      if (!d.key || !d.value) return
      entries.set(d.key, { key: d.key, ...d.value })
      repaint()
    }
    const onClear = (e: Event): void => {
      const d = ((e as CustomEvent).detail ?? {}) as ClearDetail
      if (!d.key) return
      if (entries.delete(d.key)) repaint()
    }

    document.addEventListener('sui:status:set', onSet)
    document.addEventListener('sui:status:clear', onClear)

    return () => {
      document.removeEventListener('sui:status:set', onSet)
      document.removeEventListener('sui:status:clear', onClear)
    }
  },
})
