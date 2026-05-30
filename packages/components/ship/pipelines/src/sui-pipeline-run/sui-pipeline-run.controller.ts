// <sui-pipeline-run> — derives the state label so the chip renders real DOM
// text (awaiting → "review"), not CSS ::after content.
import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-run.sui'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

export default controller<Scope>({
  scope: ({ state }: ViewScope<Scope>) => {
    const stateLabel = derived(() => STATE_LABEL[state()] ?? '')
    return { stateLabel }
  },
})
