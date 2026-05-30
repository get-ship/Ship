// Stage: composes world + loop + collide + input + audio + particles + palette
// behind one cohesive surface. `<sui-stage>` mounts a Stage; the director
// works through the Stage handle and never reaches into subsystems directly.

import { createAudio, type AudioGraph } from './audio.ts'
import { createCollide, type CollideEvent, type CollideSystem } from './collide.ts'
import { createInput, type InputBus } from './input.ts'
import { createLoop, type Loop } from './loop.ts'
import { createPalette, type Palette } from './palette.ts'
import { createParticles, type ParticleSystem } from './particles.ts'
import { createWorld, type Entity, type World } from './world.ts'

export type StageState = 'idle' | 'playing' | 'paused' | 'over'

export interface StageOptions {
  host: HTMLElement
  canvas: HTMLCanvasElement
  width: number
  height: number
  tickRate?: number
  fps?: number
  pixelRatio?: number
  background?: number
  renderer?: 'auto' | 'webgl' | 'canvas'
}

export interface Stage {
  readonly world: World
  readonly input: InputBus
  readonly audio: AudioGraph
  readonly palette: Palette
  readonly collide: CollideSystem
  readonly particles: ParticleSystem
  readonly loop: Loop
  state: StageState
  onState(fn: (s: StageState) => void): () => void
  onCollide(fn: (e: CollideEvent) => void): () => void
  onTick(fn: (dt: number, idx: number) => void): () => void
  onFrame(fn: (interp: number, fps: number) => void): () => void
  start(): void
  pause(): void
  resume(): void
  reset(): void
  destroy(): Promise<void>
}

export async function createStage(opts: StageOptions): Promise<Stage> {
  const palette = createPalette(opts.host)
  const world = await createWorld({
    canvas: opts.canvas,
    width: opts.width,
    height: opts.height,
    background: opts.background ?? 0x000000,
    pixelRatio: opts.pixelRatio ?? Math.min(globalThis.devicePixelRatio ?? 1, 2),
    renderer: opts.renderer ?? 'auto',
  })
  const collide = createCollide()
  // Bind keyboard at the window so input works without canvas focus. The
  // input bus suppresses keys while a real text input has focus.
  const inputTarget: HTMLElement | Window = typeof globalThis.window !== 'undefined' ? globalThis.window : opts.host
  const input = createInput(inputTarget)
  const audio = createAudio()
  const particles = createParticles()

  let state: StageState = 'idle'
  const stateSubs = new Set<(s: StageState) => void>()
  const collideSubs = new Set<(e: CollideEvent) => void>()
  const tickSubs = new Set<(dt: number, idx: number) => void>()
  const frameSubs = new Set<(interp: number, fps: number) => void>()

  function setState(next: StageState) {
    if (state === next) return
    state = next
    for (const fn of stateSubs) fn(state)
  }

  const loop = createLoop({
    tickRate: opts.tickRate ?? 60,
    renderFps: opts.fps ?? 60,
    onTick(dt, idx) {
      input.pollGamepads()
      if (state !== 'playing') return
      for (const e of world.entities) {
        if (e.dead) continue
        e.x += e.vx * dt
        e.y += e.vy * dt
      }
      const events = collide.step()
      for (const ev of events) {
        for (const fn of collideSubs) fn(ev)
      }
      particles.step(dt)
      for (const fn of tickSubs) fn(dt, idx)
    },
    onRender(interp, fps) {
      world.flush()
      world.app.render()
      for (const fn of frameSubs) fn(interp, fps)
    },
  })

  const stage: Stage = {
    world,
    input,
    audio,
    palette,
    collide,
    particles,
    loop,
    get state() { return state },
    set state(v: StageState) { setState(v) },
    onState(fn) { stateSubs.add(fn); return () => stateSubs.delete(fn) },
    onCollide(fn) { collideSubs.add(fn); return () => collideSubs.delete(fn) },
    onTick(fn) { tickSubs.add(fn); return () => tickSubs.delete(fn) },
    onFrame(fn) { frameSubs.add(fn); return () => frameSubs.delete(fn) },
    start() {
      setState('playing')
      loop.start()
    },
    pause() {
      if (state === 'playing') setState('paused')
      loop.pause()
    },
    resume() {
      if (state === 'paused') setState('playing')
      loop.resume()
    },
    reset() {
      for (const e of [...world.entities]) e.destroy()
      world.flush()
      setState('idle')
    },
    async destroy() {
      loop.stop()
      input.destroy()
      audio.destroy()
      particles.destroy()
      palette.destroy()
      stateSubs.clear()
      collideSubs.clear()
      tickSubs.clear()
      frameSubs.clear()
      try {
        await world.app.destroy?.(true, { children: true, texture: true })
      } catch {
        // best-effort teardown
      }
    },
  }

  return stage
}

export type { Entity, World } from './world.ts'
