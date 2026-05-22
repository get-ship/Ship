// @ship/pipelines — SUI pipeline primitives registry.
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/pipelines'` registers every tag globally.
import './sui-pipeline-step/sui-pipeline-step.compiled.js'
import './sui-pipeline-trigger/sui-pipeline-trigger.compiled.js'
import './sui-pipeline-card/sui-pipeline-card.compiled.js'
