import { controller, effect } from '@ship/sui-runtime'
import { createShipConnectClient } from '@ship/sdk/connect'
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
    if (!host.hasAttribute('role')) host.setAttribute('role', 'status')

    // Auto-fetch when tier is unknown — components that already have
    // the data pass it via attributes and skip this round-trip.
    if (state.tier === 'unknown') {
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
      const tier = state.tier
      const strategy = state.strategy
      const downgraded = state.downgraded === 'true'
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
    // Same-origin Connect: the daemon's auth middleware reads the
    // session cookie, so no token plumbing. The wire returns the typed
    // `IsolationStatusResponse` — `decision.resolved` is the resolved
    // (strategy, tier, reason, downgraded) tuple the badge needs.
    const connect = createShipConnectClient({ baseUrl: '' })
    const resp = await connect.isolation.status({})
    const resolved = resp.decision?.resolved
    if (!resolved) return null
    return {
      tier: String(resolved.tier ?? 'unknown'),
      strategy: String(resolved.strategy ?? ''),
      downgraded: Boolean(resolved.downgraded),
    }
  } catch {
    return null
  }
}
