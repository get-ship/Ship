<controller>
// <sui-pipeline-card> — derives the state label so the chip renders real DOM
// text (awaiting → "review"), not CSS ::after content.
import { controller, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './sui-pipeline-card.sail'

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
</controller>

<!--
  <sui-pipeline-card> — card form for pipeline lists. Identity + last-run state
  (left rail + chip) + step-type summary + recent-runs sparkline.

  Props:  state (ok|running|awaiting|err|none) · interactive
  Slots:  default = name · desc · steps (type-chip summary) · recent (sparkline)
  Parts:  card · chip
-->
<!-- <prop name="state" values="ok|running|awaiting|err|none" default="none" reflect /> -->
<!-- <prop name="interactive" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<article>
  <div>
    <span><slot></slot></span>
    <span><span></span>{stateLabel()}</span>
  </div>
  <p><slot name="desc"></slot></p>
  <div><slot name="steps"></slot></div>
  <div><slot name="recent"></slot></div>
</article>
