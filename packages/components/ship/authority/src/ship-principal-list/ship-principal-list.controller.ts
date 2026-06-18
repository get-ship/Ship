import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-principal-list.sui'

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

export default controller({
  scope: ({ principals }: ViewScope<Scope>, host) => {
    const list = derived<Row[]>(() => parse(principals()).map(buildRow))

    function edit(name: string): void {
      host?.dispatchEvent(
        new CustomEvent('ship-edit', { detail: { name }, bubbles: true, composed: true }),
      )
    }

    return { list, edit }
  },
})
