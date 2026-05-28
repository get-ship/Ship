import { controller, ref, signal } from '@ship/sui-runtime'
import type { Signal, ViewScope } from '@ship/sui-runtime'
import { createStage, type CollideEvent, type Emitter, type EntityVisualInit, type Stage, type StageState } from '@ship/arcade-engine'
import type { Scope } from './sui-stage.sui'
import { parseEmitterSize, readChildSpecs } from './children.ts'
import { wireHudBindings } from './hud.ts'

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

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const canvasRef = ref<HTMLCanvasElement>(null)
    const state = signal<StageState>('idle')
    const score = signal<number>(0)
    return { canvasRef, state, score }
  },

  mount(state, host) {
    let stage: Stage | null = null
    let disposed = false
    const unsubs: Array<() => void> = []
    const bound = new Map<string, Signal<unknown>>()
    bound.set('state', state.state as unknown as Signal<unknown>)
    bound.set('score', state.score as unknown as Signal<unknown>)
    let hudCleanup: (() => void) | null = null
    let emitters = new Map<string, Emitter>()

    const ready = (async () => {
      // Component-mount state preserves signals as callables — call the ref
      // to read the canvas element (the bind:this populates it post-render).
      const canvasSignal = state.canvasRef as unknown as () => HTMLCanvasElement | null
      const canvas = typeof canvasSignal === 'function' ? canvasSignal() : (state.canvasRef as unknown as HTMLCanvasElement | null)
      if (!canvas) return
      const numWidth = Number(host.getAttribute('width') ?? 640)
      const numHeight = Number(host.getAttribute('height') ?? 480)
      const tickRate = Number(host.getAttribute('tick-rate') ?? 60)
      const fps = Number(host.getAttribute('fps') ?? 60)
      const declaredDpr = Number(host.getAttribute('pixel-ratio') ?? 0)
      const pixelRatio = declaredDpr > 0 ? declaredDpr : Math.min(globalThis.devicePixelRatio ?? 1, 2)
      const bgToken = host.getAttribute('background') ?? 'card'
      const renderer = (host.getAttribute('renderer') ?? 'auto') as 'auto' | 'webgl' | 'canvas'

      stage = await createStage({
        host,
        canvas,
        width: numWidth,
        height: numHeight,
        tickRate,
        fps,
        pixelRatio,
        background: 0x000000,
        renderer,
      })
      if (disposed) {
        await stage.destroy()
        return
      }

      ;(host as HostApi).stage = stage

      // Resolve the real background through the stage's palette (token-aware,
      // oklch-safe) and re-resolve on theme change so the canvas keeps up.
      const applyBg = () => {
        const bg = stage!.palette.resolve(bgToken)
        const bgClear = stage!.world.app.renderer.background
        if (bgClear) bgClear.color = bg
      }
      applyBg()
      unsubs.push(stage.palette.onChange(applyBg))

      // Children: inputs, sfx, emitters, sprites, shapes.
      const specs = readChildSpecs(host)

      for (const sp of specs.inputs) {
        const off = stage.input.bind(sp.keys, sp.action, { repeat: sp.repeat })
        unsubs.push(off)
      }

      for (const sp of specs.sfx) {
        if (sp.src) void stage.audio.preload(sp.src)
        const off = stage.input.on(sp.action, (phase) => {
          if (phase !== 'start' || !stage) return
          const [lo, hi] = sp.pitchRange
          const pitch = lo + Math.random() * (hi - lo)
          stage.audio.play(sp.src, { volume: sp.volume, pitch, bus: sp.bus })
        })
        unsubs.push(off)
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
          const off = stage.input.on(sp.action, (phase, detail) => {
            if (phase !== 'start') return
            if (detail && typeof detail === 'object') {
              const d = detail as { x?: number; y?: number; count?: number }
              if (typeof d.x === 'number') emitter.x = d.x
              if (typeof d.y === 'number') emitter.y = d.y
              emitter.burst(d.count)
            } else {
              emitter.burst()
            }
          })
          unsubs.push(off)
        }
        unsubs.push(() => emitter.destroy())
      }

      for (const sp of specs.entities) {
        const visual: EntityVisualInit = sp.visual === 'sprite'
          ? { kind: 'sprite', src: sp.src, anchor: sp.anchor, frames: sp.frames, frame: sp.frame }
          : { kind: 'shape', shape: sp.shape, width: sp.width, height: sp.height, radius: sp.radius, points: sp.points, fill: stage.palette.resolve(sp.fill), stroke: sp.stroke ? stage.palette.resolve(sp.stroke) : undefined, strokeWidth: sp.strokeWidth }
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
        state.state.set(s)
        host.dispatchEvent(new CustomEvent('sui-stage:state', { bubbles: true, composed: true, detail: { state: s } }))
      }))
      unsubs.push(stage.onTick((dt, idx) => {
        host.dispatchEvent(new CustomEvent('sui-stage:tick', { bubbles: false, detail: { dt, idx } }))
      }))
      unsubs.push(stage.onFrame((interp, framesPerSec) => {
        host.dispatchEvent(new CustomEvent('sui-stage:frame', { bubbles: false, detail: { interp, fps: framesPerSec } }))
      }))
      unsubs.push(stage.onCollide((ev: CollideEvent) => {
        host.dispatchEvent(new CustomEvent('sui-collide', { bubbles: true, composed: true, detail: ev }))
      }))

      // Forward input bus → DOM as sui-action so the director can listen
      // via standard addEventListener instead of holding the InputBus.
      // (Director can still use stage.input for finer control.)
      // We don't blanket-subscribe to every action; the director registers
      // its own input.on() callbacks for the actions it cares about.

      host.dispatchEvent(new CustomEvent('sui-stage:ready', { bubbles: true, composed: true, detail: { stage } }))
    })()

    const api = host as HostApi
    api.start = () => { stage?.start() }
    api.pause = () => { stage?.pause() }
    api.resume = () => { stage?.resume() }
    api.reset = () => { stage?.reset(); state.score.set(0) }
    api.fire = (action, detail) => { stage?.input.fire(action, detail) }
    api.bind = (name, sig) => {
      bound.set(name, sig)
      // re-run HUD wiring so newly-registered names take effect.
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
  },
})

