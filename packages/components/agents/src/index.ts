// @ship/agents — Ship Agents pack (ship-* tags, pre-bound to ship.agents.*).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/agents'` registers every tag globally.
//
// Host requirement: the host (shell main.ts) must call setShipClient()
// before any of these tags mount, otherwise the lazy `ship` Proxy will
// throw on first access.
import './ship-agents-table/ship-agents-table.compiled.js'
import './ship-agent-card/ship-agent-card.compiled.js'
