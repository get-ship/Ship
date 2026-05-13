import { controller, derived } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-history-row.sui'

const SUBJECT_RE = /^([a-z]+(?:\([^)]+\))?!?):\s*(.+)$/

export default controller<Scope>({
  scope: (state, host) => {
    const s = state as unknown as {
      sha: Signal<string>
      subject: Signal<string>
      author: Signal<string>
      active: Signal<boolean>
    }

    const scopePrefix = derived<string>(() => {
      const m = s.subject().match(SUBJECT_RE)
      return m ? `${m[1]}:` : ''
    })
    const subjectRest = derived<string>(() => {
      const m = s.subject().match(SUBJECT_RE)
      return m ? m[2] : s.subject()
    })
    const initial = derived<string>(() => (s.author() || '?').charAt(0).toUpperCase())
    const dotR = derived<number>(() => (s.active() ? 6 : 4.5))

    function emitSelect(): void {
      host.dispatchEvent(
        new CustomEvent('sui-select', {
          detail: { sha: s.sha() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { scopePrefix, subjectRest, initial, dotR, emitSelect }
  },

  mount(state, host) {
    const scope = state as unknown as {
      sha: Signal<string>
    }
    if (!host.hasAttribute('role')) host.setAttribute('role', 'button')
    if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        host.dispatchEvent(
          new CustomEvent('sui-select', {
            detail: { sha: scope.sha() },
            bubbles: true,
            composed: true,
          }),
        )
      }
    }
    host.addEventListener('keydown', onKey)

    return () => {
      host.removeEventListener('keydown', onKey)
    }
  },
})
