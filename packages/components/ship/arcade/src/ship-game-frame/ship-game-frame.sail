<controller>
// <ship-game-frame> — chrome only. The title-bar buttons emit
// `ship-action { action }` so the host (a game director) decides what to do.
// No engine contact; the game itself lives in the default slot.
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-game-frame.sail'

export default controller((_ctx: ViewScope<Scope>, host?: Element) => {

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
})
</controller>

<!--
  <ship-game-frame> — the in-game SURFACE chrome every arcade game mounts into.
  A title bar (back · name · pause/restart/close), a HUD strip (score · lives ·
  level), the game canvas (default slot — drop a <sui-stage> here), and a
  controls-hint footer. Game colors map to ship tone tokens so themes re-skin.
  Chrome only — it never touches the PixiJS engine. Prop-driven + slotted.

  Props:  name · score · lives · level · controls (hint text)
  Slots:  default = the game canvas (e.g. <sui-stage>)
  Events: ship-action { action: back|pause|restart|close }
  Parts:  surface · titlebar · hud · canvas · controls
-->
<!-- <prop name="name" type="string" default="Game" /> -->
<!-- <prop name="score" type="string" default="" /> -->
<!-- <prop name="lives" type="string" default="" /> -->
<!-- <prop name="level" type="string" default="" /> -->
<!-- <prop name="controls" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div>
    <button type="button" aria-label="Back" on:click={onBack}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 6l-6 6 6 6"/></svg></button>
    <span>{name()}</span>
    <span>
      <button type="button" aria-label="Pause" on:click={onPause}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 5v14M16 5v14"/></svg></button>
      <button type="button" aria-label="Restart" on:click={onRestart}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5"/></svg></button>
      <button type="button" aria-label="Close" on:click={onClose}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12M18 6L6 18"/></svg></button>
    </span>
  </div>
  <div>
    <span><span>Score</span><span>{score()}</span></span>
    <span><span>Lives</span><span>{lives()}</span></span>
    <span><span>Lv</span><span>{level()}</span></span>
  </div>
  <div><slot></slot></div>
  <div><span>{controls()}</span></div>
</div>
