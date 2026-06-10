// @ship/arcade — SUI arcade primitives registry.
//
// Side-effect imports for the eight arcade components. Each
// `<tag>.compiled.js` calls `customElements.define()` on import, so a
// single `import '@ship/arcade'` at the consumer registers every tag in
// this pack globally.

import './sui-stage/sui-stage.compiled.js'
import './sui-sprite/sui-sprite.compiled.js'
import './sui-shape/sui-shape.compiled.js'
import './sui-collider/sui-collider.compiled.js'
import './sui-emitter/sui-emitter.compiled.js'
import './sui-input/sui-input.compiled.js'
import './sui-sfx/sui-sfx.compiled.js'
import './sui-hud/sui-hud.compiled.js'
import './ship-arcade/ship-arcade.compiled.js'
import './ship-game-frame/ship-game-frame.compiled.js'
