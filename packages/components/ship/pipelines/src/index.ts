// @ship/pipelines — SUI pipeline primitives registry.
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/pipelines'` registers every tag globally.
import './sui-pipeline-step/sui-pipeline-step.compiled.js'
import './sui-pipeline-card/sui-pipeline-card.compiled.js'
import './sui-pipeline-run/sui-pipeline-run.compiled.js'
import './sui-pipeline-dag/sui-pipeline-dag.compiled.js'
import './sui-log-stream/sui-log-stream.compiled.js'
import './ship-pipeline-graph/ship-pipeline-graph.compiled.js'
import './ship-run-history/ship-run-history.compiled.js'
