import { controller, derived, effect } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
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

export default controller<Scope>({
  scope: (state) => {
    const s = state as unknown as {
      state: Signal<RunState>
      slug: Signal<string>
      agent: Signal<string>
    }
    const tone = derived<Tone>(() => STATE_TO_TONE[s.state()] ?? 'muted')
    return { tone }
  },

  mount(state, host) {
    const scope = state as unknown as {
      state: Signal<RunState>
      slug: Signal<string>
      agent: Signal<string>
    }
    if (!host.hasAttribute('role')) host.setAttribute('role', 'article')

    const stopLabel = effect(() => {
      const slug = scope.slug()
      const agent = scope.agent()
      const s = scope.state()
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
