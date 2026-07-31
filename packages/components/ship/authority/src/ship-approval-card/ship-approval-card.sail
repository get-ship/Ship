<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-approval-card.sail'

export interface AuthorityRequest {
  id: string
  agent: string
  space: string
  kind: 'capability' | 'command'
  scope?: string
  path?: string
  command?: string
  why: string
  risk: 'low' | 'medium' | 'high'
  ageLabel?: string
}

type Decision = 'deny' | 'allow_once' | 'allow_session'

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

// Risk maps onto the existing badge tone scale — no bespoke risk component.
const RISK_TONE: Record<AuthorityRequest['risk'], 'muted' | 'warning' | 'destructive'> = {
  low: 'muted',
  medium: 'warning',
  high: 'destructive',
}

export default controller(({ request }: ViewScope<Scope>, host) => {

    // `request` arrives as a JSON string (SUI props carry no object type).
    const req = derived<AuthorityRequest>(() => ({ ...EMPTY, ...parseRequest(request()) }))

    const isCapability = derived(() => req().kind === 'capability')
    const agent = derived(() => req().agent)
    const space = derived(() => req().space)
    const why = derived(() => req().why)
    const risk = derived(() => req().risk)
    const riskTone = derived(() => RISK_TONE[req().risk] ?? 'muted')
    const command = derived(() => req().command ?? '')
    const scopeLine = derived(() => [req().scope, req().path].filter(Boolean).join(' '))
    const kindLabel = derived(() =>
      req().kind === 'command' ? 'Wants to run' : 'Requests capability',
    )

    function decide(decision: Decision): void {
      host?.dispatchEvent(
        new CustomEvent('ship-decide', {
          detail: { id: req().id, decision },
          bubbles: true,
          composed: true,
        }),
      )
    }
    function deny(): void { decide('deny') }
    function allowOnce(): void { decide('allow_once') }
    function allowSession(): void { decide('allow_session') }

    function inspect(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-inspect', {
          detail: { id: req().id },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return {
      req, isCapability, agent, space, why, risk, riskTone, command, scopeLine, kindLabel,
      deny, allowOnce, allowSession, inspect,
    }
})
</controller>

<!--
  <ship-approval-card> — one pending authority request, rendered for a decision.

  Presentational + event-out: it takes a `request` object and emits the
  operator's choice. The host (views/authority) owns the
  ship.authority.pending() subscription and the ship.authority.decide()
  mutation — the card never touches the SDK, so it stays trivially testable
  and reusable.

  Request shape (narrowed to what the card renders):
    { id, agent, space, kind: 'capability' | 'command',
      scope?, path?, command?, why, risk: 'low' | 'medium' | 'high', ageLabel? }

  Events (bubble + composed):
    ship-decide  { detail: { id, decision: 'deny' | 'allow_once' | 'allow_session' } }
    ship-inspect { detail: { id } }   — fired when a capability agent is clicked,
                                         so the host can open <ship-capability-request>

  Composition: <badge> for the risk tag, <button> for the actions.
-->

<!-- `request` is a JSON string (SUI props carry no object type); the
     controller parses it. Hosts pass it via request={obj} (the runtime
     JSON-encodes) or a literal request='{…}' attribute. -->
<!-- <prop name="request" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<article data-test="approval-card">
  <div>
    <if cond={isCapability()}>
      <button type="button" data-test="inspect" on:click={inspect}>{agent()}</button>
    <else>
      <span>{agent()}</span>
    </else></if>
    <span aria-hidden="true">·</span>
    <span>{space()}</span>
    <span><badge tone={riskTone()} variant="soft" size="sm" data-test="risk">{risk()}</badge></span>
  </div>

  <div>{kindLabel()}</div>

  <if cond={isCapability()}>
    <div data-test="scope">{scopeLine()}</div>
  <else>
    <pre data-test="command">$ {command()}</pre>
  </else></if>

  <div>Why · {why()}</div>

  <div>
    <button variant="outline" tone="destructive" on:click={deny}>Deny</button>
    <button variant="outline" on:click={allowOnce}>Allow once</button>
    <button variant="solid" tone="primary" on:click={allowSession}>Allow this session</button>
  </div>
</article>
