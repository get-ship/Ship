import { controller, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-capability-request.sui'
import type { AuthorityRequest } from '../ship-approval-card/ship-approval-card.controller'

type Duration = 'once' | 'session' | 'always'

const EMPTY: AuthorityRequest = {
  id: '', agent: '', space: '', kind: 'capability', why: '', risk: 'low',
}

const OPTIONS: { id: Duration; label: string }[] = [
  { id: 'once', label: 'Once' },
  { id: 'session', label: 'This session' },
  { id: 'always', label: 'Always' },
]

function parseRequest(raw: unknown): Partial<AuthorityRequest> {
  if (raw && typeof raw === 'object') return raw as Partial<AuthorityRequest>
  if (typeof raw !== 'string' || raw === '') return {}
  try {
    return JSON.parse(raw) as Partial<AuthorityRequest>
  } catch {
    return {}
  }
}

export default controller({
  scope: ({ request, duration, open }: ViewScope<Scope>, host) => {
    const dialogRef = ref<HTMLElement>(null)
    const req = derived<AuthorityRequest>(() => ({ ...EMPTY, ...parseRequest(request()) }))
    const agent = derived(() => req().agent)
    const space = derived(() => req().space)
    const why = derived(() => req().why)
    const scopeLine = derived(() => [req().scope, req().path].filter(Boolean).join(' '))
    const options = derived(() => OPTIONS)

    function pickDuration(d: Duration): void {
      duration.set(d)
    }

    function emitDecide(decision: 'deny' | 'allow'): void {
      host?.dispatchEvent(
        new CustomEvent('ship-decide', {
          detail: { id: req().id, decision, duration: duration() },
          bubbles: true,
          composed: true,
        }),
      )
      open.set(false)
    }
    function deny(): void { emitDecide('deny') }
    function allow(): void { emitDecide('allow') }

    return { dialogRef, req, agent, space, why, scopeLine, options, pickDuration, deny, allow }
  },

  mount(state, host) {
    type DialogEl = HTMLElement & { show(): void; showModal(): void; close(): void }

    // Drive the inner sui-dialog from our `open` signal.
    const stopOpenSync = effect(() => {
      const wantOpen = state.open
      const dlg = state.dialogRef as DialogEl | null
      if (!dlg) return
      if (wantOpen && typeof dlg.show === 'function') dlg.show()
      else if (!wantOpen && typeof dlg.close === 'function') dlg.close()
    })

    // Inner dialog closed by ESC / backdrop → mirror open=false and notify host.
    const onDialogClose = () => {
      if (state.open) {
        state.open = false
        host.dispatchEvent(new CustomEvent('ship-dismiss', { detail: {}, bubbles: true, composed: true }))
      }
    }
    host.addEventListener('sui-close', onDialogClose)

    return () => {
      stopOpenSync()
      host.removeEventListener('sui-close', onDialogClose)
    }
  },
})
