import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-history-row.sui'

const SUBJECT_RE = /^([a-z]+(?:\([^)]+\))?!?):\s*(.+)$/

export default controller({
  scope: ({ sha, subject, author, active }: ViewScope<Scope>, host) => {
    const scopePrefix = derived<string>(() => {
      const m = subject().match(SUBJECT_RE)
      return m ? `${m[1]}:` : ''
    })
    const subjectRest = derived<string>(() => {
      const m = subject().match(SUBJECT_RE)
      return m ? m[2] : subject()
    })
    const initial = derived<string>(() => (author() || '?').charAt(0).toUpperCase())
    const dotR = derived<number>(() => (active() ? 6 : 4.5))

    function emitSelect(): void {
      host?.dispatchEvent(
        new CustomEvent('sui-select', {
          detail: { sha: sha() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { scopePrefix, subjectRest, initial, dotR, emitSelect }
  },

  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'button')
    if (!host.hasAttribute('tabindex')) host.setAttribute('tabindex', '0')

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        host.dispatchEvent(
          new CustomEvent('sui-select', {
            detail: { sha: state.sha },
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
