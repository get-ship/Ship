<scope>
  props: {
    label = ''
    active = false
    hue = ''
  }
</scope>
<controller>
import { controller } from '@ship/sail-runtime'
import type { Scope } from './ship-dock-app.sail'

// One dock tile. Presentational except for a single intent: clicking the tile
// launches its app. The host owns the actual launch (open the app surface).
export default controller((ctx: Scope, host?: Element) => {
  function launch(): void {
    host?.dispatchEvent(
      new CustomEvent('ship-launch', {
        detail: { label: ctx.label() },
        bubbles: true,
        composed: true,
      }),
    )
  }
  return { launch }
})
</controller>

<!--
  ship-dock-app.sail — @ship/chrome · one dock tile. Icon-only launcher; active dot below.
  Props: label · active · hue(css color). Icon slotted (an <svg> or icon).
  Controller emits ship-launch {label} on click. Tokens only.
-->
<button type="button" title="{label()}" on:click={launch}><slot></slot></button>
