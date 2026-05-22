// <sui-pipeline-step> — derives the display labels from the type/state props
// so the template renders real DOM text (not CSS ::before/::after content).
import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-step.sui'

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
    kindLabel: ({ type }) => read(type),
    stateLabel: ({ state }) => STATE_LABEL[read(state)] ?? '',
  },
})
