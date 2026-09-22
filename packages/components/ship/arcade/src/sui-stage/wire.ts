// Stage host wiring. Lives outside the .sail controller so DOM types stay
// out of the sail-pure-region typecheck. Tests mock `@ship/arcade-engine`.

import type { Signal } from '@ship/sail-runtime'
import { createStage, type CollideEvent, type Emitter, type EntityVisualInit, type Stage, type StageState } from '@ship/arcade-engine'
import { parseEmitterSize, readChildSpecs } from './children'
import { wireHudBindings } from './hud'

export interface StageProps {
  width: () => number
  height: () => number
  tickRate: () => number
  fps: () => number
  pixelRatio: () => number
  renderer: () => 'auto' | 'webgl' | 'canvas'
  background: () => string
}

export interface StageSignals {
  state: Signal<StageState>
  score: Signal<number>
}

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

export function bindStage(
  host: HTMLElement,
  getCanvas: () => Element | null,
  props: StageProps,
  signals: StageSignals,
): () => void {
  let stage: Stage | null = null
  let disposed = false
  const unsubs: Array<() => void> = []
  const bound = new Map<string, Signal<unknown>>()
  bound.set('state', signals.state as Signal<unknown>)
  bound.set('score', signals.score as Signal<unknown>)
  let hudCleanup: (() => void) | null = null
  const emitters = new Map<string, Emitter>()

  const ready = (async () => {
    let canvas = getCanvas() as HTMLCanvasElement | null
    if (!canvas) {
      await Promise.resolve()
      canvas = getCanvas() as HTMLCanvasElement | null
    }
    if (!canvas || disposed) return
    const dpr = props.pixelRatio()
    stage = await createStage({
      host,
      canvas,
      width: props.width(),
      height: props.height(),
      tickRate: props.tickRate(),
      fps: props.fps(),
      pixelRatio: dpr > 0 ? dpr : Math.min(globalThis.devicePixelRatio ?? 1, 2),
      background: 0x000000,
      renderer: props.renderer(),
    })
    if (disposed) {
      await stage.destroy()
      return
    }
    ;(host as HostApi).stage = stage

    const bgToken = props.background()
    const applyBg = () => {
      const bg = stage!.palette.resolve(bgToken)
      const bgClear = stage!.world.app.renderer.background
      if (bgClear) bgClear.color = bg
    }
    applyBg()
    unsubs.push(stage.palette.onChange(applyBg))

    const specs = readChildSpecs(host)
    for (const sp of specs.inputs) {
      unsubs.push(stage.input.bind(sp.keys, sp.action, { repeat: sp.repeat }))
    }
    for (const sp of specs.sfx) {
      if (sp.src) void stage.audio.preload(sp.src)
      unsubs.push(stage.input.on(sp.action, (phase) => {
        if (phase !== 'start' || !stage) return
        const [lo, hi] = sp.pitchRange
        const pitch = lo + Math.random() * (hi - lo)
        stage.audio.play(sp.src, { volume: sp.volume, pitch, bus: sp.bus })
      }))
    }
    for (const sp of specs.emitters) {
      const [start, end] = parseEmitterSize(sp.size)
      const emitter = stage.particles.add(stage.world, {
        kind: sp.kind,
        rate: sp.rate,
        burstCount: sp.burst,
        lifetime: sp.lifetime,
        spread: sp.spread,
        direction: sp.direction,
        speed: sp.speed,
        speedJitter: sp.speedJitter,
        gravity: sp.gravity,
        color: stage.palette.resolve(sp.color),
        sizeStart: start,
        sizeEnd: end,
        max: sp.max,
      }, sp.x, sp.y)
      if (sp.id) emitters.set(sp.id, emitter)
      if (sp.action) {
        unsubs.push(stage.input.on(sp.action, (phase, detail) => {
          if (phase !== 'start') return
          if (detail && typeof detail === 'object') {
            const d = detail as { x?: number; y?: number; count?: number }
            if (typeof d.x === 'number') emitter.x = d.x
            if (typeof d.y === 'number') emitter.y = d.y
            emitter.burst(d.count)
          } else {
            emitter.burst()
          }
        }))
      }
      unsubs.push(() => emitter.destroy())
    }
    for (const sp of specs.entities) {
      const visual: EntityVisualInit = sp.visual === 'sprite'
        ? { kind: 'sprite', src: sp.src, anchor: sp.anchor, frame: sp.frame, ...(sp.frames ? { frames: sp.frames } : {}) }
        : {
            kind: 'shape', shape: sp.shape, width: sp.width, height: sp.height,
            radius: sp.radius, points: sp.points, fill: stage.palette.resolve(sp.fill),
            strokeWidth: sp.strokeWidth, ...(sp.stroke ? { stroke: stage.palette.resolve(sp.stroke) } : {}),
          }
      const entity = stage.world.spawn({
        tag: sp.tag, group: sp.group,
        x: sp.x, y: sp.y, z: sp.z, vx: sp.vx, vy: sp.vy,
        rotation: sp.rotation, scale: sp.scale, alpha: sp.alpha,
        tint: sp.tint ? stage.palette.resolve(sp.tint) : null,
        visual,
      })
      if (sp.collider) {
        stage.collide.add(entity, {
          kind: sp.collider.kind,
          w: sp.collider.width || (sp.visual === 'shape' ? sp.width : 16),
          h: sp.collider.height || (sp.visual === 'shape' ? sp.height : 16),
          r: sp.collider.radius || (sp.visual === 'shape' ? sp.radius : 8),
          ox: sp.collider.offsetX,
          oy: sp.collider.offsetY,
          group: sp.collider.group,
          collidesWith: sp.collider.collidesWith,
          trigger: sp.collider.trigger,
        })
      }
    }

    hudCleanup = wireHudBindings(host, bound)
    unsubs.push(stage.onState((s) => {
      signals.state.set(s)
      host.dispatchEvent(new CustomEvent('sail-stage:state', {
        bubbles: true, composed: true, detail: { state: s },
      }))
    }))
    unsubs.push(stage.onTick((dt, idx) => {
      host.dispatchEvent(new CustomEvent('sail-stage:tick', { bubbles: false, detail: { dt, idx } }))
    }))
    unsubs.push(stage.onFrame((interp, framesPerSec) => {
      host.dispatchEvent(new CustomEvent('sail-stage:frame', { bubbles: false, detail: { interp, fps: framesPerSec } }))
    }))
    unsubs.push(stage.onCollide((ev: CollideEvent) => {
      host.dispatchEvent(new CustomEvent('sail-collide', { bubbles: true, composed: true, detail: ev }))
    }))
    host.dispatchEvent(new CustomEvent('sail-stage:ready', {
      bubbles: true, composed: true, detail: { stage },
    }))
  })()

  const api = host as HostApi
  api.start = () => { stage?.start() }
  api.pause = () => { stage?.pause() }
  api.resume = () => { stage?.resume() }
  api.reset = () => { stage?.reset(); signals.score.set(0) }
  api.fire = (action, detail) => { stage?.input.fire(action, detail) }
  api.bind = (name, sig) => {
    bound.set(name, sig)
    hudCleanup?.()
    hudCleanup = wireHudBindings(host, bound)
  }
  api.set = (name, value) => {
    const sig = bound.get(name)
    if (sig && 'set' in sig) (sig as Signal<unknown>).set(value)
  }

  return () => {
    disposed = true
    void ready
    hudCleanup?.()
    for (const off of unsubs) off()
    unsubs.length = 0
    emitters.clear()
    void stage?.destroy()
    delete api.start
    delete api.pause
    delete api.resume
    delete api.reset
    delete api.fire
    delete api.bind
    delete api.set
    delete api.stage
  }
}
