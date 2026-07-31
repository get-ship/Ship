<controller>
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-dock-app.sail'

// One dock tile. Presentational except for a single intent: clicking the tile
// launches its app. The host owns the actual launch (open the app surface).
export default controller(({ label }: ViewScope<Scope>, host) => {

    function launch(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-launch', {
          detail: { label: label() },
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
  Props: label · active · hue(css color). Icon slotted (an <svg> or sui-icon).
  Controller emits ship-launch {label} on click. Tokens only.
-->
<!-- <prop name="label" type="string" default="" /> -->
<!-- <prop name="active" type="bool" default="false" reflect /> -->
<!-- <prop name="hue" type="string" default="" /> -->
<!-- style block removed: use sx / external CSS -->
<button type="button" title="{label()}" on:click={launch}><slot></slot></button>
