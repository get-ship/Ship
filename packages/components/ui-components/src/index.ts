// @ship/ui-components — minimal re-export shim.
//
// The four domain components that previously lived here have been rehomed:
//   sui-graph-canvas  → @ship/sui-canvas   (packages/components/sui/canvas)
//   sui-log-stream    → @ship/pipelines    (packages/components/ship/pipelines)
//   sui-harness-blob  → @ship/agents       (packages/components/ship/agents)
//   sui-auth-dialog   → @ship/authority    (packages/components/ship/authority)
//
// This shim exists ONLY because the /_sui/components.js umbrella endpoint is
// hard-coupled to this package's index.ts entry point in
// crates/deck/deck-bundler/src/targets.rs (the 'sui-components' framework
// singleton). The shim keeps the endpoint intact until the framework audit
// completes and the bundler target can be redirected or eliminated.
//
// DEFERRED: Once the framework audit (#266) closes, update targets.rs to
// replace the 'sui-components' entry and delete this package entirely.

import '@ship/sui-primitives'
import '@ship/sui-canvas'
import '@ship/pipelines'
import '@ship/agents'
import '@ship/authority'
