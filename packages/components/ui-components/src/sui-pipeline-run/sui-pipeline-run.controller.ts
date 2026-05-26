import { controller, derived, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-run.sui'

type RunState =
  | 'dispatched'
  | 'gatepending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'merged'
  | 'pending'

type Tone =
  | 'default'
  | 'primary'
  | 'info'
  | 'success'
  | 'warning'
  | 'destructive'
  | 'accent'
  | 'muted'

const STATE_TO_TONE: Record<RunState, Tone> = {
  dispatched: 'info',
  running: 'info',
  gatepending: 'warning',
  completed: 'success',
  failed: 'destructive',
  merged: 'accent',
  pending: 'muted',
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    // `ctx.state` is the run-state prop (string); accessing as a signal call.
    const tone = derived<Tone>(() => STATE_TO_TONE[ctx.state() as RunState] ?? 'muted')
    return { tone }
  },

  mount(state, host) {
    if (!host.hasAttribute('role')) host.setAttribute('role', 'article')

    const stopLabel = effect(() => {
      const slug = state.slug
      const agent = state.agent
      const s = state.state
      host.setAttribute(
        'aria-label',
        `Pipeline run ${slug || ''}${agent ? ` on ${agent}` : ''}: ${s}`,
      )
    })

    return () => {
      stopLabel()
    }
  },
})
