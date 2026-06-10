import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-capability-bar.sui'

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

export default controller({
  scope: ({ can, cant }: ViewScope<Scope>) => {
    const canList = derived<Grant[]>(() =>
      parse<Grant>(can()).map((g) => ({ domain: g.domain ?? '', tier: normTier(g.tier) })),
    )
    const cantList = derived<Denial[]>(() =>
      parse<Denial>(cant()).map((d) => ({
        domain: d.domain ?? '',
        tier: normTier(d.tier),
        why: d.why ?? '',
      })),
    )
    const hasDenied = derived<boolean>(() => cantList().length > 0)
    return { canList, cantList, hasDenied }
  },
})
