// @ship/agents — Ship Agents pack (ship-* tags, pre-bound to ship.agents.*).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/agents'` registers every tag globally.
//
// Host requirement: the host (shell main.ts) must call setShipClient()
// before any of these tags mount, otherwise the lazy `ship` Proxy will
// throw on first access.
//
// <ship-agent-thread> additionally composes `@ship/sui-chat-elements`
// primitives (<sui-message-list>, <sui-composer>); the host registers
// that pack (`import '@ship/sui-chat-elements'`) — it is a declared dependency.
import './ship-agents-table/ship-agents-table.compiled.js'
import './ship-agent-card/ship-agent-card.compiled.js'
import './ship-agent-thread/ship-agent-thread.compiled.js'
import './sui-harness-blob/sui-harness-blob.compiled.js'
