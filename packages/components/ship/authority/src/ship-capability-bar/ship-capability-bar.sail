<scope>
  props: {
    can = '[]'
    cant = '[]'
  }
</scope>
<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './ship-capability-bar.sail'

type Tier = 'C0' | 'C1' | 'C2' | 'C3'

export interface Grant {
  domain: string
  tier: Tier
}
export interface Denial extends Grant {
  why?: string
}

function parse<T>(raw: unknown): T[] {
  const arr = typeof raw === 'string' ? safeJson(raw) : raw
  return Array.isArray(arr) ? (arr as T[]) : []
}

function safeJson(raw: string): unknown {
  if (raw === '') return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

const normTier = (t: unknown): Tier =>
  t === 'C1' || t === 'C2' || t === 'C3' ? t : 'C0'

export default controller((ctx: Scope) => {
    const canList = derived<Grant[]>(() =>
      parse<Grant>(ctx.can()).map((g) => ({ domain: g.domain ?? '', tier: normTier(g.tier) })),
    )
    const cantList = derived<Denial[]>(() =>
      parse<Denial>(ctx.cant()).map((d) => ({
        domain: d.domain ?? '',
        tier: normTier(d.tier),
        why: d.why ?? '',
      })),
    )
    const hasDenied = derived<boolean>(() => cantList().length > 0)
    return { canList, cantList, hasDenied }
})
</controller>

<!--
  <ship-capability-bar> — @ship/authority · an agent's authority made legible: the
  GRANTS (what it CAN do — domain × max blast tier) and, explicitly, what it CAN'T
  (denied domains carry the reason). The standing-grant companion to op-card's
  per-op gate verdict.

  COMPOSES <ship-blast-tag> for every tier chip so the tier vocabulary reads
  identically across the pack (correction: no inlined blast-tier CSS).

  Data-driven: the design's fixture chips become the `can` / `cant` props (JSON
  arrays); the host binds them to ▸read agent.capabilities. Read-only summary —
  the agent-editor owns the bespoke editable tier-picker (SPEC open item resolved:
  bar stays read-only, no `editable` variant).

  Props:  can   JSON string — [{ domain, tier }]            (granted)
          cant  JSON string — [{ domain, tier, why }]       (denied)
  Part:   capabilities
-->

<div>
  <div>
    <span>can</span>
    <for {canList()} as="c" key={c.domain}>
      <span><span>{c.domain}</span><ship-blast-tag tier={c.tier}></ship-blast-tag></span>
    </for>
  </div>
  <if {hasDenied()}>
    <span></span>
    <div>
      <span>can't</span>
      <for {cantList()} as="d" key={d.domain}>
        <span>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="11" width="14" height="9" rx="2"></rect><path d="M8 11V7a4 4 0 0 1 8 0v4"></path></svg>
          <span>{d.domain}</span><ship-blast-tag tier={d.tier}></ship-blast-tag>
          <if {d.why}><span>{d.why}</span></if>
        </span>
      </for>
    </div>
  </if>
</div>
