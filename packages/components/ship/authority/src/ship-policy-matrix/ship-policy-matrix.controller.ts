import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-policy-matrix.sui'

type Tier = 'C0' | 'C1' | 'C2' | 'C3'
const TIERS: Tier[] = ['C0', 'C1', 'C2', 'C3']

export interface PolicyRow {
  domain: string
  /** Highest allowed tier; '' = operator-only (every column locked). */
  granted?: Tier | ''
  note?: string
}

interface Cell {
  tone: string
  granted: boolean
  cls: string
}
interface GridRow {
  domain: string
  note: string
  cells: Cell[]
}

function parse(raw: unknown): PolicyRow[] {
  const arr = typeof raw === 'string' ? safeJson(raw) : raw
  return Array.isArray(arr) ? (arr as PolicyRow[]) : []
}

function safeJson(raw: string): unknown {
  if (raw === '') return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

function buildRow(r: PolicyRow): GridRow {
  const grantedIdx = r.granted ? TIERS.indexOf(r.granted) : -1
  const cells = TIERS.map((tier, i) => {
    const tone = tier.toLowerCase()
    const granted = i <= grantedIdx
    return {
      tone,
      granted,
      cls: granted ? `cell grant ${tone}` : 'cell lock',
    }
  })
  return { domain: r.domain ?? '', note: r.note ?? '', cells }
}

export default controller({
  scope: ({ rows }: ViewScope<Scope>) => {
    const grid = derived<GridRow[]>(() => parse(rows()).map(buildRow))
    return { grid }
  },
})
