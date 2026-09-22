<scope>
  props: {
    principals = '[]'
  }
</scope>
<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './ship-principal-list.sail'

type Tier = 'C0' | 'C1' | 'C2' | 'C3'
type Tone = 'primary' | 'accent' | 'info' | 'success' | 'warning'
const TONES: Tone[] = ['primary', 'accent', 'info', 'success', 'warning']

export interface Principal {
  name: string
  harness?: string
  initials?: string
  tone?: Tone
  grants?: { domain: string; tier: Tier }[]
  denials?: string[]
}

interface Row {
  name: string
  harness: string
  initials: string
  tileCls: string
  grants: { domain: string; tier: Tier }[]
  denials: string[]
}

function parse(raw: unknown): Principal[] {
  const arr = typeof raw === 'string' ? safeJson(raw) : raw
  return Array.isArray(arr) ? (arr as Principal[]) : []
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

const initialsOf = (name: string): string =>
  name
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((w) => w[0]!.toUpperCase())
    .slice(0, 2)
    .join('') || '··'

function buildRow(p: Principal): Row {
  const tone: Tone = p.tone && TONES.includes(p.tone) ? p.tone : 'primary'
  return {
    name: p.name ?? '',
    harness: p.harness ?? '',
    initials: p.initials ?? initialsOf(p.name ?? ''),
    tileCls: `tile t-${tone}`,
    grants: (p.grants ?? []).map((g) => ({ domain: g.domain ?? '', tier: normTier(g.tier) })),
    denials: (p.denials ?? []).filter((d): d is string => typeof d === 'string'),
  }
}

export default controller((ctx: Scope, host?: Element) => {
    const list = derived<Row[]>(() => parse(ctx.principals()).map(buildRow))

    function edit(name: string): void {
      host?.dispatchEvent(
        new CustomEvent('ship-edit', { detail: { name }, bubbles: true, composed: true }),
      )
    }

    return { list, edit }
})
</controller>

<!--
  <ship-principal-list> — @ship/authority · the principals (agents / roles) and what
  each is granted — the "who" of authority. Row = principal · harness · max-grant
  chips · edit. Editing a principal opens the bespoke tier-picker (the host's
  agent-editor); this list emits the intent.

  COMPOSES <ship-blast-tag> for each granted-tier chip (correction: no inlined
  blast-tier CSS); denials render as a muted lock chip.

  Data-driven: the design's fixture rows become the `principals` prop (a JSON
  array); the host binds it to ▸read principals + policy.

  Props:  principals  JSON string —
            [{ name, harness, initials, tone, grants:[{domain,tier}], denials:[label] }]
            tone ∈ primary | accent | info | success | warning  (the avatar hue)
  Part:   list
  Event (bubble + composed):
    ship-edit { detail: { name } }   — the row's "Edit grants" was pressed
-->

<div>
  <for {list()} as="pr" key={pr.name}>
    <div>
      <span>
        <span>{pr.initials}</span>
        <span>{pr.name}</span>
        <span>{pr.harness}</span>
      </span>
      <span>
        <for {pr.grants} as="g" key={g.domain}>
          <span><span>{g.domain}</span><ship-blast-tag tier={g.tier}></ship-blast-tag></span>
        </for>
        <for {pr.denials} as="d" key={d}>
          <span>{d} ✕</span>
        </for>
      </span>
      <button type="button" on:click={() => edit(pr.name)}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"></path></svg>Edit grants
      </button>
    </div>
  </for>
</div>
