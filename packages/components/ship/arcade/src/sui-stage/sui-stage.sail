<!--
  <sui-stage> — root of an arcade game scene.

  Owns the canvas, the fixed-timestep loop, the Pixi renderer, the input
  bus, the audio graph, the palette, the collide system, and the particle
  system. Light-DOM children describe the static scene; the director
  script imperatively spawns dynamics.

  Recognized children: sui-stage-input, sui-sfx, sui-emitter, sui-sprite,
  sui-shape, sui-hud (sprites/shapes may nest sui-collider).

  Host events: sail-stage:ready | tick | frame | state, sail-collide.
  Host API: start/pause/resume/reset/fire/bind/set + stage handle.
-->
<scope>
  props: {
    width = 640
    height = 480
    'tick-rate' = 60
    fps = 60
    'pixel-ratio' = 0
    paused = false
    background = 'surface'
    renderer: 'auto' | 'webgl' | 'canvas' = 'auto'
    bounds: 'clamp' | 'wrap' | 'open' = 'open'
  }
</scope>

<controller>
import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sui-stage.sail'
import type { StageState } from './engine'
import { bindStage } from './wire'

export default controller((ctx: ControllerScope<Scope>) => {
  const canvasEl = signal<unknown>(null)
  const captureCanvas: Action<void> = (node) => {
    canvasEl.set(node)
    return { destroy: () => canvasEl.set(null) }
  }
  const state = signal<StageState>('idle')
  const score = signal<number>(0)

  mount((host) => bindStage(
    host,
    () => canvasEl() as never,
    {
      width: () => ctx.width(),
      height: () => ctx.height(),
      tickRate: () => ctx.tickRate(),
      fps: () => ctx.fps(),
      pixelRatio: () => ctx.pixelRatio(),
      renderer: () => ctx.renderer(),
      background: () => ctx.background(),
    },
    { state, score },
  ))

  return { captureCanvas, state, score }
})
</controller>

<canvas use:captureCanvas width={width()} height={height()} tabindex="0"></canvas>
<div>
  <slot></slot>
</div>
