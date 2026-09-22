<scope>
  props: {
    rows = '[]'
  }
</scope>
<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './ship-policy-matrix.sail'

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

export default controller((ctx: Scope) => {
    const grid = derived<GridRow[]>(() => parse(ctx.rows()).map(buildRow))
    return { grid }
})
</controller>

<!--
  <ship-policy-matrix> — @ship/authority · the capability × blast grid. For each
  domain: the max granted blast tier (a check at every tier ≤ the grant) and a lock
  beyond it. The "what's allowed" companion to capability-bar's at-a-glance summary.

  Data-driven: the design's fixture rows become the `rows` prop (a JSON array); the
  host binds it to ▸read policy. This is the teaching grid where the C0–C3 columns
  are LABELLED (Read/Write/Elevated/Critical mapping) — the per-column tone is the
  axis itself, not a tag, so it is the one authority surface that does NOT compose
  <ship-blast-tag> (the atom renders a chip, not a grid cell).

  Props:  rows  JSON string — [{ domain, granted: 'C0'|'C1'|'C2'|'C3'|'', note }]
                granted = highest allowed tier ('' = operator-only, all locked)
  Part:   matrix
-->

<div>
  <table>
    <thead><tr><th>domain</th><th>C0</th><th>C1</th><th>C2</th><th>C3</th><th>scope</th></tr></thead>
    <tbody>
      <for {grid()} as="r" key={r.domain}>
        <tr>
          <td>{r.domain}</td>
          <for {r.cells} as="cell" key={cell.tone}>
            <td>
              <if {cell.granted}>
                <span><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"></path></svg></span>
              {else =>
                <span><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="11" width="14" height="9" rx="2"></rect><path d="M8 11V7a4 4 0 0 1 8 0v4"></path></svg></span>
              }</if>
            </td>
          </for>
          <td>{r.note}</td>
        </tr>
      </for>
    </tbody>
  </table>
</div>
