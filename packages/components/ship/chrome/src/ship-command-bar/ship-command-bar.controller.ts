import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-command-bar.sui'

const MODES = ['find', 'search', 'create', 'ai'] as const
type Mode = (typeof MODES)[number]

function isMode(v: string | undefined): v is Mode {
  return v != null && (MODES as readonly string[]).includes(v)
}

function emit(host: HTMLElement | null, type: string, detail?: unknown): void {
  host?.dispatchEvent(
    new CustomEvent(type, { detail, bubbles: true, composed: true }),
  )
}

// The cmdk keystone. Modes are reflected for styling; the controller turns
// operator input into shell intents. ⌘K (global) toggles open; the host owns
// the scrim, but a scrim/esc dismiss reduces to the same ship-close.
export default controller({
  scope: ({ open, mode }: ViewScope<Scope>, host) => {
    function pickMode(e: Event): void {
      const next = (e.currentTarget as HTMLElement | null)?.dataset.m
      if (!isMode(next) || next === mode()) return
      mode.set(next)
      emit(host, 'ship-mode-change', { mode: next })
    }

    function onKey(e: KeyboardEvent): void {
      if (e.key === 'Enter') {
        const query = (e.target as HTMLInputElement | null)?.value ?? ''
        emit(host, 'ship-submit', { query })
      } else if (e.key === 'Escape') {
        open.set(false)
        emit(host, 'ship-close')
      }
    }

    return { pickMode, onKey }
  },

  mount(state, host) {
    function onGlobalKey(e: KeyboardEvent): void {
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault()
        const next = !state.open
        state.open = next
        emit(host, next ? 'ship-open' : 'ship-close')
      }
    }
    // A slotted result is chosen — its click bubbles to the host; lift the
    // nearest [data-id] up as ship-select so the host can act on the choice.
    function onResultClick(e: Event): void {
      const id = (e.target as Element | null)?.closest<HTMLElement>('[data-id]')?.dataset.id
      if (id != null) emit(host, 'ship-select', { id })
    }
    window.addEventListener('keydown', onGlobalKey)
    host.addEventListener('click', onResultClick)
    return () => {
      window.removeEventListener('keydown', onGlobalKey)
      host.removeEventListener('click', onResultClick)
    }
  },
})
