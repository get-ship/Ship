import { controller, signal, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface AuthorityRequest {
  id: string
  agent: string
  space: string
  kind: 'command' | 'capability'
  command?: string
  scope?: string
  path?: string
  why: string
  risk: 'low' | 'medium' | 'high'
  ageLabel: string
}

interface Duration { id: string; label: string }

const DURATIONS: Duration[] = [
  { id: 'once', label: 'Just this file' },
  { id: 'session', label: 'Until I close Space' },
  { id: 'always', label: 'Always' },
]

const EMPTY: AuthorityRequest[] = []

export default controller<Scope>({
  scope: (ctx: ViewScope<Scope>) => {
    const { ship } = ctx

    const pending = live<AuthorityRequest[]>(() =>
      (ship as any).authority.pending()
    )

    const decidedIds = signal(new Set<string>())
    const modalRequestId = signal<string | null>(null)
    const modalDuration = signal('session')

    const visiblePending = derived((): AuthorityRequest[] => {
      const all = unwrapOr(pending, EMPTY)
      const decided = decidedIds()
      return all.filter((r) => !decided.has(r.id))
    })

    const modalRequest = derived((): AuthorityRequest | null => {
      const id = modalRequestId()
      if (!id) return null
      return unwrapOr(pending, EMPTY).find((r) => r.id === id) ?? null
    })

    const modalOpen = derived(() => modalRequestId() !== null)

    const durations = derived(() => DURATIONS)

    const decide = (id: string, verdict: string) => {
      const next = new Set(decidedIds())
      next.add(id)
      decidedIds.set(next)
      ;(ship as any).authority?.decide?.({ id, verdict })
    }

    const openModal = (id: string) => {
      modalRequestId.set(id)
      modalDuration.set('session')
    }

    const setModalDuration = (id: string) => modalDuration.set(id)

    const modalDeny = () => {
      const id = modalRequestId()
      if (id) decide(id, 'deny')
      modalRequestId.set(null)
    }

    const modalAllow = () => {
      const id = modalRequestId()
      if (id) {
        const verdict = modalDuration() === 'once' ? 'allow_once' : 'allow_session'
        decide(id, verdict)
      }
      modalRequestId.set(null)
    }

    const replayDecisions = () => {}
    const openPolicy = () => {}

    return {
      pending,
      visiblePending,
      modalOpen,
      modalRequest,
      modalDuration,
      durations,
      decide,
      openModal,
      setModalDuration,
      modalDeny,
      modalAllow,
      replayDecisions,
      openPolicy,
    }
  },
})
