import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-help-overlay.sui'

/** One shortcut line: the keys to press and what they do. `keys` is the
 *  ordered list of caps rendered as <kbd> joined by +. */
interface Shortcut {
  keys: string[]
  desc: string
}

/** A labelled cluster of shortcuts. */
interface ShortcutGroup {
  label: string
  rows: Shortcut[]
}

/** The reference content. Every binding here is one the shell's
 *  document-level keydown handler in apps/shell/src/main.ts emits — keep
 *  this in lockstep with that handler, not with aspirational features. */
const GROUPS: ShortcutGroup[] = [
  {
    label: 'Global',
    rows: [
      { keys: ['⌘', 'K'], desc: 'Open command palette' },
      { keys: ['⌘', 'P'], desc: 'Find / jump to anything' },
      { keys: ['⌘', 'N'], desc: 'Create…' },
      { keys: ['⌘', '.'], desc: 'Open Studio' },
      { keys: ['⌘', 'B'], desc: 'Toggle navigation rail' },
      { keys: ['⌘', 'J'], desc: 'Toggle panel dock' },
      { keys: ['⌃', '`'], desc: 'Toggle terminal' },
      { keys: ['⌘', '?'], desc: 'Open this help' },
      { keys: ['Esc'], desc: 'Return to canvas / close overlay' },
    ],
  },
  {
    label: 'Editing',
    rows: [
      { keys: ['⌥', 'E'], desc: 'Toggle edit mode for the current view' },
      { keys: ['⇧', '⌥', 'E'], desc: 'Toggle annotation mode' },
    ],
  },
]

export default controller({
  scope: (state: ViewScope<Scope>, host?: Element) => {
    const hostEl = host as HTMLElement
    const dialogEl = ref<HTMLDialogElement>(null)
    const groups = signal<ShortcutGroup[]>(GROUPS)

    const close = () => {
      hostEl.dispatchEvent(new CustomEvent('sui-help-close', { bubbles: true, composed: true }))
      // `scope:` receives the RAW scope — `open` is the prop Signal here,
      // not the unwrapped MutableState view that `mount`/`on:` handlers get.
      if (state.open()) state.open.set(false)
    }

    return { dialogEl, groups, onClose: () => close() }
  },

  mount(state, host) {
    const dialogEl = state.dialogEl
    if (!dialogEl) return

    const close = state.onClose

    // The `open` signal is the source of truth; native showModal() drives
    // the top layer + ::backdrop, mirroring <sui-command-palette>.
    const stopOpenSync = effect(() => {
      const wantOpen = state.open
      if (wantOpen && !dialogEl.open) {
        try { dialogEl.showModal() } catch { /* jsdom lacks showModal */ }
      } else if (!wantOpen && dialogEl.open) {
        dialogEl.close()
      }
    })

    // Native dialog closes itself on Esc / cancel — keep `open` in sync
    // and emit our close intent. Backdrop click lands on the <dialog>
    // element itself when the user hits the dimmed margin.
    const onNativeClose = () => close()
    const onDialogClick = (e: MouseEvent) => {
      if (e.target === dialogEl) close()
    }

    dialogEl.addEventListener('close', onNativeClose)
    dialogEl.addEventListener('cancel', onNativeClose)
    dialogEl.addEventListener('click', onDialogClick)

    host.expose!({
      show: () => { state.open = true },
      hide: () => close(),
    })

    return () => {
      stopOpenSync()
      dialogEl.removeEventListener('close', onNativeClose)
      dialogEl.removeEventListener('cancel', onNativeClose)
      dialogEl.removeEventListener('click', onDialogClick)
    }
  },
})
