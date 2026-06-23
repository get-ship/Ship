import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { AgentDetail } from '@ship/sdk-client'
import type { Scope } from './ship-agent-card.sui'

// `ship.agents.detail()` is typed against the proto-generated AgentDetail (SD-8),
// so new proto fields show up here without a local edit. The controller still
// picks only the fields the template renders — `Pick<...>` keeps the union
// narrow while preserving every shape `ship.agents.detail()` can return
// (otherwise a missing field would fall through to the EMPTY default).
type AgentDetailFields = Pick<
  AgentDetail,
  'id' | 'name' | 'description' | 'source' | 'tags' | 'harness' | 'model' | 'status' | 'capabilities'
>
interface Capability {
  label: string
  tier: string // C0 | C1 | C2 | C3
}
interface TonedCap extends Capability {
  tone: string
}

const EMPTY: AgentDetailFields = {
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

export default controller({
  scope: ({ id }: ViewScope<Scope>, host) => {
    const agent = live(() => ship.agents.detail({ id: id() }))
    const data = derived<AgentDetailFields>(() => ({
      ...EMPTY,
      ...(unwrapOr(agent, EMPTY) as Partial<AgentDetailFields>),
    }))
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
