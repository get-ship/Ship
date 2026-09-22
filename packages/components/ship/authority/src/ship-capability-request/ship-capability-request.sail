<scope>
  props: {
    request = ''
    duration: 'once' | 'session' | 'always' = 'session'
    open = false
  }
</scope>
<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './ship-capability-request.sail'
import type { AuthorityRequest } from '../ship-approval-card/ship-approval-card.sail'

const EMPTY: AuthorityRequest = {
  id: '', agent: '', space: '', kind: 'capability', why: '', risk: 'low',
}

function parseRequest(raw: unknown): Partial<AuthorityRequest> {
  if (raw && typeof raw === 'object') return raw as Partial<AuthorityRequest>
  if (typeof raw !== 'string' || raw === '') return {}
  try {
    return JSON.parse(raw) as Partial<AuthorityRequest>
  } catch {
    return {}
  }
}

export default controller((ctx: Scope, host?: Element) => {
    const req = derived<AuthorityRequest>(() => ({ ...EMPTY, ...parseRequest(ctx.request()) }))
    const agent = derived(() => req().agent)
    const space = derived(() => req().space)
    const why = derived(() => req().why)
    const scopeLine = derived(() => [req().scope, req().path].filter(Boolean).join(' '))

    // sui-toggle-group emits sui-change { value } — mirror it into our prop.
    function onDurationChange(e: Event): void {
      const value = (e as CustomEvent<{ value: string }>).detail?.value
      if (value === 'once' || value === 'session' || value === 'always') ctx.duration.set(value)
    }

    function emitDecide(decision: 'deny' | 'allow'): void {
      host?.dispatchEvent(
        new CustomEvent('ship-decide', {
          detail: { id: req().id, decision, duration: ctx.duration() },
          bubbles: true,
          composed: true,
        }),
      )
      ctx.open.set(false)
    }
    function deny(): void { emitDecide('deny') }
    function allow(): void { emitDecide('allow') }

    function onClose(): void {
      if (ctx.open()) ctx.open.set(false)
    }

    return { req, agent, space, why, scopeLine, onDurationChange, deny, allow, onClose }
})
</controller>

<!--
  <ship-capability-request> — the capability-request modal body.

  Shown when the operator inspects a capability request from the queue. Lays
  out SCOPE / WHY / FOR-HOW-LONG and emits the decision. Presentational +
  event-out, like <ship-approval-card>: the host owns ship.authority.decide().

  `request` is a JSON string (SUI props carry no object type); the controller
  parses it. Same shape as <ship-approval-card>'s request.

  Props:
    request   JSON string — the capability request
    duration  'once' | 'session' | 'always'  (default 'session')
    open      bool — drives the inner <dialog>

  Events (bubble + composed):
    ship-decide  { detail: { id, decision: 'deny' | 'allow', duration } }
    ship-dismiss { detail: {} }   — backdrop / ESC closed the dialog

  Composition: <dialog> chrome, <button> footer actions, and
  <toggle-group>/<toggle> (single-select) for the duration picker.
-->

<dialog
  size="sm"
  role-override="alertdialog"
  no-close-button
  open={open()}
  on-close={onClose}
>
  <span slot="header">
    <p>Capability request</p>
  </span>

  <p data-test="sub">{agent()} · {space()}</p>

  <div>
    <p>Scope</p>
    <div data-test="scope">{scopeLine()}</div>
  </div>

  <div>
    <p>Why</p>
    <div data-test="why">{why()}</div>
  </div>

  <div>
    <p>For how long</p>
    <toggle-group
      type="single"
      variant="segmented"
      value={duration()}
      aria-label="Duration"
      on:sail-change={onDurationChange}>
      <toggle value="once">Once</toggle>
      <toggle value="session">This session</toggle>
      <toggle value="always">Always</toggle>
    </toggle-group>
  </div>

  <span slot="footer">
    <button variant="outline" tone="destructive" on:click={deny}>Deny</button>
    <button variant="solid" tone="primary" on:click={allow}>Allow</button>
  </span>
</dialog>
