import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-isolation-detail.sui'

interface StatusPayload {
  tier: string
  strategy: string
  reason: string
  downgraded: boolean
  preferredTier: string
}

const REASON_LABEL: Record<string, string> = {
  run_override: 'pinned per-run via ship use --isolation',
  agent_override: 'pinned in agent profile',
  user_default: 'user default',
  os_default: 'OS default',
  downgrade: 'preferred tier unavailable; fell back',
  already_in_sandbox: 'already inside someone else’s sandbox — no double-wrap',
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // Show the reason row when reason is non-empty. SUI's template
    // ternaries don't reliably evaluate inside JSX-style branches in
    // this compiler version, so we toggle CSS display from the
    // controller instead.
    const stopReason = effect(() => {
      const row = host.shadowRoot?.querySelector<HTMLElement>('.reason-row')
      if (!row) return
      row.style.display = scope.reason() ? 'flex' : 'none'
    })

    // Auto-fetch when tier is unknown — same pattern as
    // <sui-isolation-badge>. Views that already hold the data pass
    // it via attributes and skip the round-trip.
    if (scope.tier() === 'unknown') {
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
    const resp = await fetch('/api/v1/isolation/status', {
      headers: { accept: 'application/json' },
    })
    if (!resp.ok) return null
    const json = await resp.json()
    const decision = json?.data?.decision
    if (!decision || decision.error) return null
    const resolved = decision.resolved
    if (!resolved) return null
    const reasonKind = resolved.reason?.kind ?? ''
    const reasonLabel = REASON_LABEL[reasonKind] ?? reasonKind
    let preferredTier = ''
    if (reasonKind === 'downgrade') {
      preferredTier = String(resolved.reason?.from ?? '')
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
