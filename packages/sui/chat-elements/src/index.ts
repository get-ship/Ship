// @sui/chat-elements — unbound multi-modal chat-shape primitives.
//
// First pack in the `@sui/*` namespace: SUI framework lives in `@sui/*`,
// Ship product in `@ship/*`. These are pure `sui-*` primitives — no
// `ship` Proxy, no `setShipClient`, no SDK reads. Consumers (the
// `@ship/agents` ship-* wrappers, edit-mode side chats, the activity
// stream) bind them to their own data.
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@sui/chat-elements'` registers every tag globally.
import './sui-message/sui-message.compiled.js'
import './sui-message-list/sui-message-list.compiled.js'
import './sui-composer/sui-composer.compiled.js'
import './sui-thinking-dots/sui-thinking-dots.compiled.js'
import './sui-tool-call/sui-tool-call.compiled.js'
import './sui-citation/sui-citation.compiled.js'
