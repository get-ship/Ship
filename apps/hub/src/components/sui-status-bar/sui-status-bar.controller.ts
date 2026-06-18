// <sui-status-bar> controller — exposes the connection + build + job
// state to the template. Reads the actual RPC surface:
//   - ship.runtime.state() → connection state, build identity
//   - ship.agents.summaries() → active job count
// No hand-rolled `window.ship`, no KernelBridge.

import { controller, signal, effect, shared, derived, type ViewScope } from '@ship/sui-runtime'
import { getShipClient } from '@ship/sdk-client'
import type { Scope } from './sui-status-bar.sui'

interface RuntimeState {
  build?: { sha?: string; dirty?: boolean; channel?: string }
  connected?: boolean
}

interface AgentSummary {
  id?: string
  status?: string
}

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => {
    const ship = getShipClient()

    const connState = shared(signal<string>('disconnected'))
    const connLabel = shared(signal<string>('Connecting...'))
    const buildLabel = shared(signal<string>('—'))
    const buildTitle = shared(signal<string>('No build'))
    const jobsLabel = shared(signal<string>('0 jobs'))
    const jobTooltip = shared(signal<string>(''))

    effect(() => {
      const src = ship.runtime.state()
      const apply = (s: RuntimeState) => {
        const connected = s?.connected === true
        connState.set(connected ? 'connected' : 'disconnected')
        connLabel.set(connected ? 'Connected' : 'Disconnected')
        const build = s?.build?.sha ?? ''
        const dirty = s?.build?.dirty === true
        const channel = s?.build?.channel ?? 'dev'
        if (build) {
          const sha = build.slice(0, 7)
          buildLabel.set(`${channel} · ${dirty ? 'dirty' : sha}`)
          buildTitle.set(`Build: ${build}${dirty ? ' (dirty)' : ''}\nChannel: ${channel}`)
        } else {
          buildLabel.set(channel)
          buildTitle.set(`Channel: ${channel}`)
        }
      }
      const initial = (src as { value?: RuntimeState }).value
      if (initial) apply(initial)
      const sub = (src as { subscribe?: (cb: (v: unknown) => void) => () => void }).subscribe
      if (typeof sub === 'function') {
        return sub((v) => apply(v as RuntimeState))
      }
    })

    effect(() => {
      const src = ship.agents.summaries()
      const apply = (s: { items?: AgentSummary[] } | AgentSummary[] | undefined) => {
        const items = Array.isArray(s) ? s : s?.items ?? []
        const active = items.filter((a) => a.status === 'running' || a.status === 'pending').length
        jobsLabel.set(`${active} ${active === 1 ? 'job' : 'jobs'}`)
        jobTooltip.set(items.map((a) => `${a.id}: ${a.status}`).join('\n') || 'No active jobs')
      }
      const initial = (src as { value?: unknown }).value
      if (initial !== undefined) apply(initial as Parameters<typeof apply>[0])
      const sub = (src as { subscribe?: (cb: (v: unknown) => void) => () => void }).subscribe
      if (typeof sub === 'function') {
        return sub((v) => apply(v as Parameters<typeof apply>[0]))
      }
    })

    return { connState, connLabel, buildLabel, buildTitle, jobsLabel, jobTooltip }
  },
})
