// <sui-pipeline-card> — derives the state label so the chip renders real DOM
// text (awaiting → "review"), not CSS ::after content.
import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-card.sui'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

export default controller({
  scope: ({ state }: ViewScope<Scope>) => ({
    stateLabel: derived(() => STATE_LABEL[String(state())] ?? ''),
  }),
})
