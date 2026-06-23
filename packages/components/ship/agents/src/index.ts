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
//
// <sui-agent-attach> composes `<ship-terminal>` from `@ship/code` — same
// contract: the host registers that pack (`import '@ship/code'`) so the
// embedded terminal tag is defined when the attach element mounts.
import './ship-agents-table/ship-agents-table.compiled.js'
import './ship-agent-card/ship-agent-card.compiled.js'
import './ship-agent-list/ship-agent-list.compiled.js'
import './ship-agent-detail/ship-agent-detail.compiled.js'
import './ship-agent-chat/ship-agent-chat.compiled.js'
import './ship-agent-thread/ship-agent-thread.compiled.js'
import './sui-harness-blob/sui-harness-blob.compiled.js'
import './ship-op-card/ship-op-card.compiled.js'
import './ship-agent-controls/ship-agent-controls.compiled.js'
import './ship-usage-chip/ship-usage-chip.compiled.js'
import './ship-session-metrics/ship-session-metrics.compiled.js'
import './ship-agent-canvas/ship-agent-canvas.compiled.js'
import './ship-agent-slot/ship-agent-slot.compiled.js'
import './ship-session-list/ship-session-list.compiled.js'
import './sui-agent-attach/sui-agent-attach.compiled.js'
