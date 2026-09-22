<scope>
  props: {
    type: 'agent' | 'gate' | 'merge' | 'human' = 'agent'
    state: 'ok' | 'running' | 'awaiting' | 'err' | 'none' = 'none'
    compact = false
  }
</scope>
<controller>
// <sui-pipeline-step> — derives the display labels from the type/state props
// so the template renders real DOM text (not CSS ::before/::after content).
import type { Scope } from './sui-pipeline-step.sail'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

export default controller((ctx: Scope) => {
  const kindLabel = derived(() => ctx.type())
  const stateLabel = derived(() => STATE_LABEL[ctx.state()] ?? '')
  return { kindLabel, stateLabel }
})
</controller>

<!--
  <sui-pipeline-step> — one pipeline step, standalone. Embeddable inline or
  inside a DAG/run. Type-coloured accent + kind label, optional state chip,
  sub line, and consume→produce artifact chips.

  Props:  type (agent|gate|merge|human) · state (ok|running|awaiting|err|none) · compact
  Slots:  default = step id/title · sub = sub line · arts = artifact chips
  Parts:  step · accent · kind · chip
-->

<div>
  <div>
    <span></span>
    <div>
      <div>
        <span>{kindLabel()}</span>
        <span><slot></slot></span>
        <span><span></span>{stateLabel()}</span>
      </div>
      <div><slot name="sub"></slot></div>
      <div><slot name="arts"></slot></div>
    </div>
  </div>
</div>
