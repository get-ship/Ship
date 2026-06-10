// <sui-pipeline-step> — derives the display labels from the type/state props
// so the template renders real DOM text (not CSS ::before/::after content).
import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-step.sui'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

export default controller<Scope>({
  scope: ({ type, state }: ViewScope<Scope>) => {
    const kindLabel = derived(() => type())
    const stateLabel = derived(() => STATE_LABEL[state()] ?? '')
    return { kindLabel, stateLabel }
  },
})
