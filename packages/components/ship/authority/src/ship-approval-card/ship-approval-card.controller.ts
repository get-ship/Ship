import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-approval-card.sui'

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

export default controller({
  scope: ({ request }: ViewScope<Scope>, host) => {
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
  },
})
