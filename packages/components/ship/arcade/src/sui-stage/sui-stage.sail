<controller>
import { controller, ref, signal } from '@ship/sail-runtime'
import type { Signal, ViewScope } from '@ship/sail-runtime'
import { createStage, type CollideEvent, type Emitter, type EntityVisualInit, type Stage, type StageState } from '@ship/arcade-engine'
import type { Scope } from './sui-stage.sail'
import { parseEmitterSize, readChildSpecs } from './children'
import { wireHudBindings } from './hud'

interface HostApi extends HTMLElement {
  start?(): void
  pause?(): void
  resume?(): void
  reset?(): void
  fire?(action: string, detail?: unknown): void
  bind?(name: string, sig: Signal<unknown>): void
  set?(name: string, value: unknown): void
  stage?: Stage
}

export default controller((_ctx: ViewScope<Scope>) => {

    const canvasRef = ref<HTMLCanvasElement>(null)
    const state = signal<StageState>('idle')
    const score = signal<number>(0)
    return { canvasRef, state, score }
})

</controller>

<!--
  <sui-stage> — root of an arcade game scene.

  Owns the canvas, the fixed-timestep loop, the Pixi renderer, the input
  bus, the audio graph, the palette (theme-aware token resolver), the
  collide system, and the particle system. Light-DOM children describe
  the static scene; the director script imperatively spawns dynamics.

  Recognized child elements (parsed once at init):
    <sui-stage-input> — key→action bindings
    <sui-sfx>      — audio bindings (preloaded, triggered by action)
    <sui-emitter>  — particle emitters (registered, optionally action-triggered)
    <sui-sprite>   — markup-declared sprite entities
    <sui-shape>    — markup-declared geometric entities
    <sui-hud>      — DOM overlay (passes through to slot)

  Sprites/shapes may have a child <sui-collider> describing their hitbox.

  Events on the host:
    sui-stage:ready        — Pixi up, assets preloaded, scene built
    sui-stage:tick         — every fixed sim tick { detail: { dt, idx } }
    sui-stage:frame        — every render frame   { detail: { interp, fps } }
    sui-action             — input bus           { detail: { action, phase } }
    sui-collide            — collision event     { detail: { a, b, ... } }
    sui-stage:state        — state changed       { detail: { state } }

  Public methods on the host:
    start() | pause() | resume() | reset() | fire(action, detail?)
    bind(name, signal)     — register a named signal for HUD data-bind
    set(name, value)       — convenience: update a bound signal's value
    get stage              — the underlying Stage handle (for director scripts)
-->

<!-- <prop name="width" type="number" default="640" /> -->
<!-- <prop name="height" type="number" default="480" /> -->
<!-- <prop name="tick-rate" type="number" default="60" /> -->
<!-- <prop name="fps" type="number" default="60" /> -->
<!-- <prop name="pixel-ratio" type="number" default="0" /> -->
<!-- <prop name="paused" type="bool" default="false" reflect /> -->
<!-- <prop name="background" type="string" default="surface" /> -->
<!-- <prop name="renderer" values="auto|webgl|canvas" default="auto" /> -->
<!-- <prop name="bounds" values="clamp|wrap|open" default="open" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<canvas bind:this={canvasRef} width={width()} height={height()} tabindex="0"></canvas>
<div>
  <slot></slot>
</div>
