<!--
  <ship-game-frame> — the in-game SURFACE chrome every arcade game mounts into.
  A title bar (back · name · pause/restart/close), a HUD strip (score · lives ·
  level), the game canvas (default slot — drop a <sui-stage> here), and a
  controls-hint footer. Chrome only — it never touches the PixiJS engine.

  Props:  name · score · lives · level · controls (hint text)
  Slots:  default = the game canvas (e.g. <sui-stage>)
  Events: ship-action { action: back|pause|restart|close }
-->
<scope>
  props: {
    name = 'Game'
    score = ''
    lives = ''
    level = ''
    controls = ''
  }
  style bar = { pad({ x: 3, y: 2 }).border({ width: 1, color: border-subtle, edge: b }).bg(surface-1) }
  style title = { type({ size: sm, weight: semibold }) }
  style hud = { pad({ x: 3, y: 2 }).border({ width: 1, color: border-subtle, edge: b }) }
  style key = { type({ size: xs, family: mono, case: uppercase }).fg(muted-foreground) }
  style val = { type({ size: xs, family: mono }) }
  style canvas = { size({ w: full, min-h: 48 }) }
  style foot = { pad({ x: 3, y: 2 }).border({ width: 1, color: border-subtle, edge: t }).bg(surface-1) }
  style hint = { type({ size: xs, family: mono }).fg(muted-foreground) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './ship-game-frame.sail'
import { emitAction, type Host } from './host'

export default controller((_ctx: ControllerScope<Scope>, host?: Host) => {
  const onBack = () => emitAction(host, 'back')
  const onPause = () => emitAction(host, 'pause')
  const onRestart = () => emitAction(host, 'restart')
  const onClose = () => emitAction(host, 'close')
  return { onBack, onPause, onRestart, onClose }
})
</controller>

<column gap="0">
  <row gap="2" align="center" style(bar)>
    <button type="button" label="Back" on:click={onBack}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 6l-6 6 6 6"/></svg>
    </button>
    <span style(title)>{name()}</span>
    <row gap="1" align="center">
      <button type="button" label="Pause" on:click={onPause}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 5v14M16 5v14"/></svg>
      </button>
      <button type="button" label="Restart" on:click={onRestart}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5"/></svg>
      </button>
      <button type="button" label="Close" on:click={onClose}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12M18 6L6 18"/></svg>
      </button>
    </row>
  </row>
  <row gap="4" align="center" style(hud)>
    <span><span style(key)>Score</span><span style(val)>{score()}</span></span>
    <span><span style(key)>Lives</span><span style(val)>{lives()}</span></span>
    <span><span style(key)>Lv</span><span style(val)>{level()}</span></span>
  </row>
  <div style(canvas)><slot></slot></div>
  <div style(foot)><span style(hint)>{controls()}</span></div>
</column>
