import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-isolation-badge.sui'

const TIER_LABELS: Record<string, string> = {
  none: 'Not sandboxed',
  process: 'Sandboxed',
  container: 'Container',
  vm: 'Virtual machine',
  cloud: 'Cloud',
  unknown: '…',
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    if (!host.hasAttribute('role')) host.setAttribute('role', 'status')

    // Auto-fetch when tier is unknown — components that already have
    // the data pass it via attributes and skip this round-trip.
    if (scope.tier() === 'unknown') {
      void fetchIsolationStatus()
        .then((status) => {
          if (!status) return
          host.setAttribute('tier', status.tier)
          if (status.strategy) host.setAttribute('strategy', status.strategy)
          host.setAttribute('downgraded', status.downgraded ? 'true' : 'false')
        })
        .catch(() => {
          // Silent — fall back to the 'unknown' chip rather than a
          // user-visible error in a passive badge.
        })
    }

    // Keep the accessible label in sync with whichever fields are set.
    const stopAria = effect(() => {
      const tier = scope.tier()
      const strategy = scope.strategy()
      const downgraded = scope.downgraded() === 'true'
      let label = TIER_LABELS[tier] ?? TIER_LABELS.unknown
      if (strategy) label += ` (${strategy})`
      if (downgraded) label += ' — downgraded'
      host.setAttribute('aria-label', label)
      host.title = label
    })

    return () => stopAria()
  },
})

interface IsolationStatusPayload {
  tier: string
  strategy: string
  downgraded: boolean
}

async function fetchIsolationStatus(): Promise<IsolationStatusPayload | null> {
  try {
    const resp = await fetch('/api/v1/isolation/status', {
      headers: { accept: 'application/json' },
    })
    if (!resp.ok) return null
    const json = await resp.json()
    const decision = json?.data?.decision?.resolved
    if (!decision) return null
    return {
      tier: String(decision.tier ?? 'unknown'),
      strategy: String(decision.strategy ?? ''),
      downgraded: Boolean(decision.downgraded),
    }
  } catch {
    return null
  }
}
