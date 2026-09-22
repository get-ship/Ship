<scope>
  props: {
    state: 'ok' | 'running' | 'awaiting' | 'err' | 'none' = 'none'
  }
</scope>
<controller>
// <sui-pipeline-run> — derives the state label so the chip renders real DOM
// text (awaiting → "review"), not CSS ::after content.
import type { Scope } from './sui-pipeline-run.sail'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

export default controller((ctx: Scope) => {
  const stateLabel = derived(() => STATE_LABEL[ctx.state()] ?? '')
  return { stateLabel }
})
</controller>

<!--
  <sui-pipeline-run> — one run, live. Header (name + state + meta) over a
  horizontal rail of per-step cells. Rail cells are slotted (use
  <sui-pipeline-run-cell> or any node); each carries its own state via the
  `cell-state` attribute the cell styling reads.

  Props:  state (ok|running|awaiting|err|none)
  Slots:  default = name · meta = right-aligned meta · rail = the step rail
  Parts:  run · header · chip · rail
-->

<div>
  <div>
    <span><slot></slot></span>
    <span><span></span>{stateLabel()}</span>
    <span><slot name="meta"></slot></span>
  </div>
  <div><slot name="rail"></slot></div>
</div>
