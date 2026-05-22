// <sui-pipeline-card> — derives the state label so the chip renders real DOM
// text (awaiting → "review"), not CSS ::after content.
import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-card.sui'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

const read = (v: unknown): string => (typeof v === 'function' ? (v as () => unknown)() : v) as string

export default controller<Scope>({
  derived: {
    stateLabel: ({ state }) => STATE_LABEL[read(state)] ?? '',
  },
})
