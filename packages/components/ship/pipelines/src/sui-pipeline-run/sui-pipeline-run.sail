<controller>
// <sui-pipeline-run> — derives the state label so the chip renders real DOM
// text (awaiting → "review"), not CSS ::after content.
import { controller, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './sui-pipeline-run.sail'

const STATE_LABEL: Record<string, string> = {
  ok: 'ok',
  running: 'running',
  awaiting: 'review',
  err: 'err',
  none: '',
}

export default controller(({ state }: ViewScope<Scope>) => {

    const stateLabel = derived(() => STATE_LABEL[state()] ?? '')
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
<!-- <prop name="state" values="ok|running|awaiting|err|none" default="none" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div>
    <span><slot></slot></span>
    <span><span></span>{stateLabel()}</span>
    <span><slot name="meta"></slot></span>
  </div>
  <div><slot name="rail"></slot></div>
</div>
