// <ship-game-frame> — chrome only. The title-bar buttons emit
// `ship-action { action }` so the host (a game director) decides what to do.
// No engine contact; the game itself lives in the default slot.
import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-game-frame.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const fire = (action: string) => {
      host?.dispatchEvent(new CustomEvent('ship-action', {
        detail: { action }, bubbles: true, composed: true,
      }))
    }
    const onBack = () => fire('back')
    const onPause = () => fire('pause')
    const onRestart = () => fire('restart')
    const onClose = () => fire('close')
    return { onBack, onPause, onRestart, onClose }
  },
})
