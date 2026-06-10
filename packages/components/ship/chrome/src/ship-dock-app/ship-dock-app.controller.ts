import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-dock-app.sui'

// One dock tile. Presentational except for a single intent: clicking the tile
// launches its app. The host owns the actual launch (open the app surface).
export default controller({
  scope: ({ label }: ViewScope<Scope>, host) => {
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
  },
})
