// @ship/pipelines — SUI pipeline primitives registry.
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/pipelines'` registers every tag globally.
import './sui-pipeline-step/sui-pipeline-step.compiled.js'
import './sui-pipeline-trigger/sui-pipeline-trigger.compiled.js'
import './sui-pipeline-card/sui-pipeline-card.compiled.js'
import './sui-pipeline-run/sui-pipeline-run.compiled.js'
import './sui-pipeline-history/sui-pipeline-history.compiled.js'
import './sui-pipeline-output/sui-pipeline-output.compiled.js'
import './sui-pipeline-definition/sui-pipeline-definition.compiled.js'
import './sui-pipeline-dag/sui-pipeline-dag.compiled.js'
