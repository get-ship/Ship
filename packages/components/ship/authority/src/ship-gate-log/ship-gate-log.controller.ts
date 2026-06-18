import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-gate-log.sui'

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

export default controller({
  scope: ({ space, entries, live }: ViewScope<Scope>) => {
    const rows = derived<GateLogEntry[]>(() => parseEntries(entries()))
    const isLive = derived<boolean>(() => live() !== false)
    const spaceLabel = derived<string>(() => (space() ? ` · ${space()}` : ''))
    return { rows, isLive, spaceLabel }
  },
})
