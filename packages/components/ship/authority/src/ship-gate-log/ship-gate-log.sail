<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-gate-log.sail'

export interface GateLogEntry {
  /** Stable key for the keyed loop. Falls back to time+session+op. */
  id: string
  time: string
  session: string
  op: string
  blast: 'C0' | 'C1' | 'C2' | 'C3'
  decision: 'allowed' | 'approved' | 'needs-approval' | 'denied' | 'running'
}

function parseEntries(raw: unknown): GateLogEntry[] {
  const arr = typeof raw === 'string' ? safeJson(raw) : raw
  if (!Array.isArray(arr)) return []
  return arr.map((r, i) => {
    const e = (r ?? {}) as Partial<GateLogEntry>
    const time = e.time ?? ''
    const session = e.session ?? ''
    const op = e.op ?? ''
    return {
      id: e.id ?? `${time}|${session}|${op}|${i}`,
      time,
      session,
      op,
      blast: (e.blast ?? 'C0') as GateLogEntry['blast'],
      decision: (e.decision ?? 'allowed') as GateLogEntry['decision'],
    }
  })
}

function safeJson(raw: string): unknown {
  if (raw === '') return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

export default controller(({ space, entries, live }: ViewScope<Scope>) => {

    const rows = derived<GateLogEntry[]>(() => parseEntries(entries()))
    const isLive = derived<boolean>(() => live() !== false)
    const spaceLabel = derived<string>(() => (space() ? ` · ${space()}` : ''))
    return { rows, isLive, spaceLabel }
})
</controller>

<!--
  <ship-gate-log> — @ship/authority · the live stream of gate verdicts (audit /
  observability). Row = time · session · op · blast-tier · gate-verdict. The honest
  record beneath the rendered UI: every effect, every decision.

  COMPOSES the pack atoms — <ship-blast-tag> for the tier and <ship-gate-badge> for
  the verdict — so the vocabulary reads identically to op-card and capability-bar.

  Data-driven: the design's fixture rows become the `entries` prop (a JSON array);
  the host binds it to the live ▸stream of gate verdicts. Presentational (no
  events) — the controller only parses the rows.

  Props:  space    string — the surface label in the header
          entries  JSON string — [{ time, session, op, blast, decision }]
          live     bool — header pulses while the stream is live (default true)
  Part:   log
-->
<!-- <prop name="space" type="string" default="" /> -->
<!-- <prop name="entries" type="string" default="[]" /> -->
<!-- <prop name="live" type="bool" default="true" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div>
    <if cond={isLive()}><span><span></span>live</span><span>· gate verdicts{spaceLabel()}</span><else><span>gate verdicts{spaceLabel()}</span></else></if>
  </div>
  <for as="e" of={rows()} key={e.id}>
    <div>
      <span>{e.time}</span>
      <span>{e.session}</span>
      <span>{e.op}</span>
      <ship-blast-tag tier={e.blast}></ship-blast-tag>
      <ship-gate-badge decision={e.decision}></ship-gate-badge>
    </div>
  </for>
</div>
