import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-agent-card.sui'

// `ship.agents.detail()` is typed `LiveSource<unknown>` in the SDK
// surface (no exported row type yet), so narrow locally to the fields
// this card renders. Mirrors the daemon `AgentDetail` shape.
interface Capability {
  label: string
  tier: string // C0 | C1 | C2 | C3
}
interface AgentDetail {
  id: string
  name: string
  description: string
  source: string
  tags: string[]
  harness: string
  model: string
  status: string // idle | active | busy | …
  capabilities: Capability[]
}

const EMPTY: AgentDetail = {
  id: '', name: '', description: '', source: '', tags: [],
  harness: '', model: '', status: '', capabilities: [],
}

// Cross-pack composition note: the design summarises capability weight with the
// @ship/authority `ship-blast-tag` atom. Cross-pack *component* composition is not
// an established pattern in this repo (packs only cross-import engine modules), so
// the tier is shown with a toned `sui-chip` here. The blast-tier → chip-tone map:
const TIER_TONE: Record<string, string> = {
  C0: 'muted', C1: 'info', C2: 'warning', C3: 'destructive',
}
// Harness → chip tone (floki=primary, claude=accent, codex/other=info/muted).
const HARNESS_TONE: Record<string, string> = {
  floki: 'primary', claude: 'accent', codex: 'info',
}

interface TonedCap extends Capability {
  tone: string
}

export default controller({
  scope: ({ id }: ViewScope<Scope>, host) => {
    const agent = live(() => ship.agents.detail({ id: id() }))
    const data = derived<AgentDetail>(() => ({ ...EMPTY, ...(unwrapOr(agent, EMPTY) as Partial<AgentDetail>) }))
    const name = derived(() => data().name || data().id)
    const description = derived(() => data().description)
    const source = derived(() => data().source)
    const tags = derived<string[]>(() => (Array.isArray(data().tags) ? data().tags : []))

    const harness = derived(() => data().harness)
    const harnessTone = derived(() => HARNESS_TONE[data().harness] ?? 'muted')
    const model = derived(() => data().model)
    const status = derived(() => data().status)
    // Reflected status dot: running-ish states read primary, otherwise success.
    const statusDotClass = derived(() =>
      data().status === 'active' || data().status === 'busy' ? 'sdot busy' : 'sdot on',
    )
    const capabilities = derived<TonedCap[]>(() =>
      (Array.isArray(data().capabilities) ? data().capabilities : []).map((c) => ({
        ...c,
        tone: TIER_TONE[c.tier] ?? 'muted',
      })),
    )

    function start(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-start', {
          detail: { id: data().id || id() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return {
      agent, data, name, description, source, tags,
      harness, harnessTone, model, status, statusDotClass, capabilities, start,
    }
  },
})
