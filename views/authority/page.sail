<controller>
import { controller, signal, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

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

export default controller((ctx: ViewScope) => {

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
})
</controller>

<client name="ship" />
<data name="pending" source="ship.authority.pending()" live />

<div>

  <header>
    <div>
      <h1>Authority</h1>
      <p>Capability requests, command approvals, and the identities your spaces speak for.</p>
    </div>
    <div>
      <button type="button" on:click={replayDecisions}>Replay decisions</button>
      <button type="button" on:click={openPolicy}>Policy…</button>
    </div>
  </header>

  <while-loading of="pending">
    <div aria-busy="true" role="status">Loading…</div>
  </while-loading>

  <while-error as="err" of="pending">
    <div role="alert">{err.message}</div>
  </while-error>

  <if cond={pending().kind === 'ok'}>
    <if cond={visiblePending().length > 0}>
      <section aria-label="Awaiting your decision">
        <div>
          <h2>Awaiting your decision</h2>
          <span>{visiblePending().length} pending</span>
        </div>
        <div>
          <for as="req" of={visiblePending()} key={req.id}>
            <article>
              <div>
                <if cond={req.kind === 'capability'}>
                  <button type="button" on:click={() => openModal(req.id)}>{req.agent}</button>
                <else>
                  <span>{req.agent}</span>
                </else></if>
                <span aria-hidden="true"> · </span>
                <span>{req.space}</span>
                <span data-risk={req.risk}>{req.risk}</span>
              </div>
              <div>{req.kind === 'command' ? 'WANTS TO RUN' : 'REQUESTS CAPABILITY'}</div>
              <if cond={req.kind === 'command'}>
                <pre>$ {req.command}</pre>
              <else>
                <div>{req.scope} {req.path}</div>
              </else></if>
              <div>Why · {req.why}</div>
              <div>
                <button type="button" on:click={() => decide(req.id, 'deny')}>Deny</button>
                <button type="button" on:click={() => decide(req.id, 'allow_once')}>Allow once</button>
                <button type="button" on:click={() => decide(req.id, 'allow_session')}>Allow this session</button>
              </div>
            </article>
          </for>
        </div>
      </section>
    <else>
      <div role="status">
        <p>You're all caught up</p>
        <p>No requests awaiting your decision.</p>
      </div>
    </else></if>
  </if>

  <if cond={modalOpen()}>
    <div role="dialog" aria-modal="true" aria-label="Capability request">
      <div>
        <p>CAPABILITY REQUEST</p>
        <if cond={modalRequest()}>
          <p>{modalRequest().agent} wants to access your filesystem</p>
          <div>
            <h3>SCOPE</h3>
            <div>{modalRequest().scope} {modalRequest().path}</div>
          </div>
          <div>
            <h3>WHY</h3>
            <div>{modalRequest().why}</div>
          </div>
          <div>
            <h3>FOR HOW LONG</h3>
            <div role="group" aria-label="Duration">
              <for as="d" of={durations()} key={d.id}>
                <button
                  data-auth-duration-btn--selected={modalDuration() === d.id}
                  type="button"
                  aria-pressed={modalDuration() === d.id ? 'true' : 'false'}
                  on:click={() => setModalDuration(d.id)}>
                  {d.label}
                </button>
              </for>
            </div>
          </div>
        </if>
        <footer>
          <span>⎋ to deny</span>
          <button type="button" on:click={modalDeny}>Deny</button>
          <button type="button" on:click={modalAllow}>Allow</button>
        </footer>
        <p>Decided in hub · never inside a space.</p>
      </div>
    </div>
  </if>

</div>
