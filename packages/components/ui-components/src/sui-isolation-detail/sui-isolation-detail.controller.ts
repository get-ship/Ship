import { controller, effect } from '@ship/sui-runtime'
import { createShipConnectClient } from '@ship/sdk/connect'
import type { Scope } from './sui-isolation-detail.sui'

interface StatusPayload {
  tier: string
  strategy: string
  reason: string
  downgraded: boolean
  preferredTier: string
}

const REASON_LABEL: Record<string, string> = {
  runOverride: 'pinned per-run via ship use --isolation',
  agentOverride: 'pinned in agent profile',
  userDefault: 'user default',
  osDefault: 'OS default',
  downgrade: 'preferred tier unavailable; fell back',
  alreadyInSandbox: 'already inside someone else's sandbox — no double-wrap',
}

export default controller<Scope>({
  mount(state, host) {
    // Show the reason row when reason is non-empty. SUI's template
    // ternaries don't reliably evaluate inside JSX-style branches in
    // this compiler version, so we toggle CSS display from the
    // controller instead.
    const stopReason = effect(() => {
      const row = host.shadowRoot?.querySelector<HTMLElement>('.reason-row')
      if (!row) return
      row.style.display = state.reason ? 'flex' : 'none'
    })

    // Auto-fetch when tier is unknown — same pattern as
    // <sui-isolation-badge>. Views that already hold the data pass
    // it via attributes and skip the round-trip.
    if (state.tier === 'unknown') {
      void fetchIsolationStatus()
        .then((status) => {
          if (!status) return
          host.setAttribute('tier', status.tier)
          if (status.strategy) host.setAttribute('strategy', status.strategy)
          host.setAttribute('reason', status.reason)
          host.setAttribute('downgraded', status.downgraded ? 'true' : 'false')
          if (status.preferredTier) {
            host.setAttribute('preferred-tier', status.preferredTier)
          }
        })
        .catch(() => {
          // Silent — fall back to the unknown-tier rendering.
        })
    }

    return () => stopReason()
  },
})

async function fetchIsolationStatus(): Promise<StatusPayload | null> {
  try {
    // Same-origin Connect: the daemon's auth middleware reads the
    // session cookie, so no token plumbing. The wire returns the typed
    // `IsolationStatusResponse`; the detail view projects every
    // status-relevant field — tier, strategy, reason (discriminant →
    // label), downgraded, and the downgrade source when applicable.
    const connect = createShipConnectClient({ baseUrl: '' })
    const resp = await connect.isolation.status({})
    const resolved = resp.decision?.resolved
    if (!resolved) return null
    const reasonCase = resolved.reason?.case
    const reasonLabel = reasonCase ? (REASON_LABEL[reasonCase] ?? reasonCase) : ''
    let preferredTier = ''
    if (reasonCase === 'downgrade') {
      const down = resolved.reason?.value
      preferredTier = down?.from ? String(down.from) : ''
    }
    return {
      tier: String(resolved.tier ?? 'unknown'),
      strategy: String(resolved.strategy ?? ''),
      reason: reasonLabel,
      downgraded: Boolean(resolved.downgraded),
      preferredTier,
    }
  } catch {
    return null
  }
}
