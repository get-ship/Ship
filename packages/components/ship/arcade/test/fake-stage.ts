// Deterministic test double for `@ship/arcade-engine`'s `createStage`.
//
// The arcade pack's controllers (`sui-stage.controller.ts` et al.) compose the
// real engine `Stage`, which stands up a PixiJS WebGL renderer and a
// requestAnimationFrame loop — neither available in the test env. This double
// replaces ONLY that Pixi/WebGL/RAF boundary:
//
//   • collision + input use the REAL engine subsystems (`createCollide`,
//     `createInput`) — both are pure logic, so `sui-collider` overlap and
//     `sui-stage-input` key→action mapping are genuine behavior, not stubs.
//   • world / loop / audio / particles / palette are faithful fakes that record
//     what the controller asks of them and integrate entities exactly as the
//     real loop does (`e.x += e.vx * dt`).
//   • the loop is driven by an INJECTED clock — `clock.tick(dt)` /
//     `clock.render()` — never requestAnimationFrame or wall-clock time, so
//     every tick is deterministic and the test owns the frame cadence.
//
// `installFakeStage()` returns the captured Stage handle + clock so a test can
// drive ticks and assert against recorded audio/particle/collision/state.

// Direct relative imports of the two PURE engine subsystems (no Pixi, no RAF):
// the package's exports map only surfaces the `.` barrel, and the barrel
// re-exports collide/input as TYPES only — so the real factories are reached
// by path. Keeps collision + input genuine while everything Pixi is faked.
import { createCollide, type CollideEvent } from '../../../../arcade-engine/src/collide.ts'
import { createInput, type InputBus } from '../../../../arcade-engine/src/input.ts'
import type {
  Emitter,
  EmitterSpec,
  Entity,
  EntityInit,
  Stage,
  StageState,
} from '@ship/arcade-engine'

export interface FakeClock {
  /** Run one fixed sim tick: integrate, collide, particles, fire tick subs. */
  tick(dt?: number): void
  /** Run one render frame: fire frame subs with the given interp + fps. */
  render(interp?: number, fps?: number): void
}

export interface RecordedPlay {
  src: string
  volume?: number
  pitch?: number
  bus?: string
}

export interface FakeEmitter extends Emitter {
  /** Total particles requested across every burst() call. */
  bursts: number[]
  setColorCalls: number[]
}

export interface FakeStageHandle {
  stage: Stage
  clock: FakeClock
  plays: RecordedPlay[]
  preloads: string[]
  emitters: FakeEmitter[]
  /** Tokens the palette was asked to resolve, in order. */
  resolved: string[]
  /** Resolve a token to a fixed int — overridable per test if needed. */
  paletteMap: Map<string, number>
}

// Module-level capture: the controller calls the mocked `createStage` during
// its async mount; the test reads the handle the mock recorded here.
let current: FakeStageHandle | null = null

export function lastFakeStage(): FakeStageHandle {
  if (!current) throw new Error('fake-stage: createStage was not called yet (flush the mount first)')
  return current
}

export function resetFakeStage(): void {
  current = null
}

/** The full EntityInit the controller passed to world.spawn() for an entity
 *  — including the visual init (sprite src/frame/anchor, or shape fill/dims).
 *  Returned undefined for entities not spawned through the fake world. */
export function entityInit(e: Entity): EntityInit | undefined {
  return e.data.get('__init') as EntityInit | undefined
}

const TOKEN_INTS: Record<string, number> = {
  card: 0x111111,
  surface: 0x121212,
  accent: 0x3366ff,
  foreground: 0xeeeeee,
  'surface-fg': 0xdddddd,
}

function makeWorld() {
  const entities: Entity[] = []
  let nextId = 1
  const world = {
    // The controller reads `world.app.renderer.background` to apply the bg.
    app: { renderer: { background: { color: 0 } }, render() {}, async destroy() {} },
    spawn(init: EntityInit): Entity {
      const e: Entity = {
        id: `e${nextId++}`,
        tag: init.tag,
        group: init.group,
        x: init.x ?? 0,
        y: init.y ?? 0,
        z: init.z ?? 0,
        vx: init.vx ?? 0,
        vy: init.vy ?? 0,
        rotation: init.rotation ?? 0,
        scale: init.scale ?? 1,
        alpha: init.alpha ?? 1,
        tint: init.tint ?? null,
        data: new Map(),
        display: {},
        dead: false,
        destroy() {
          this.dead = true
          const i = entities.indexOf(this)
          if (i >= 0) entities.splice(i, 1)
        },
      }
      // Record the full spawn init (incl. the visual init the controller built
      // from the markup child) so tests can assert what was materialized.
      e.data.set('__init', init)
      entities.push(e)
      return e
    },
    destroy(e: Entity) { e.destroy() },
    byId: (id: string) => entities.find((e) => e.id === id),
    byTag: (tag: string) => entities.filter((e) => e.tag === tag),
    byGroup: (g: string) => entities.filter((e) => e.group === g),
    get entities() { return entities as ReadonlyArray<Entity> },
    async loadTexture() { return {} },
    repaintComposite() {},
    repaintShape() {},
    flush() {},
  }
  return world
}

function makeEmitter(spec: EmitterSpec, x: number, y: number): FakeEmitter {
  return {
    x,
    y,
    spec,
    bursts: [],
    setColorCalls: [],
    burst(n?: number) { this.bursts.push(n ?? spec.burstCount) },
    setColor(c: number) { this.setColorCalls.push(c) },
    destroy() {},
  }
}

/**
 * Replace `@ship/arcade-engine`'s `createStage` with a deterministic double.
 * Call inside a `vi.mock('@ship/arcade-engine', ...)` factory's `createStage`.
 */
export function makeFakeCreateStage() {
  return async function createStage(): Promise<Stage> {
    const handle: Partial<FakeStageHandle> = {
      plays: [],
      preloads: [],
      emitters: [],
      resolved: [],
      paletteMap: new Map(Object.entries(TOKEN_INTS)),
    }

    const world = makeWorld()
    const collide = createCollide()
    const inputTarget: HTMLElement | Window =
      typeof globalThis.window !== 'undefined' ? globalThis.window : document.body
    const input: InputBus = createInput(inputTarget)

    let state: StageState = 'idle'
    const stateSubs = new Set<(s: StageState) => void>()
    const collideSubs = new Set<(e: CollideEvent) => void>()
    const tickSubs = new Set<(dt: number, idx: number) => void>()
    const frameSubs = new Set<(interp: number, fps: number) => void>()
    let tickIndex = 0

    function setState(next: StageState) {
      if (state === next) return
      state = next
      for (const fn of stateSubs) fn(state)
    }

    const palette = {
      resolve(token: string): number {
        handle.resolved!.push(token)
        return handle.paletteMap!.get(token) ?? 0x000000
      },
      onChange(_fn: () => void) { return () => {} },
      destroy() {},
    }

    const audio = {
      async preload(src: string) { handle.preloads!.push(src) },
      play(src: string, opts?: { volume?: number; pitch?: number; bus?: string }) {
        handle.plays!.push({ src, ...opts })
      },
      setBusVolume() {},
      suspend() {},
      resume() {},
      destroy() {},
    }

    const particles = {
      add(_world: unknown, spec: EmitterSpec, x: number, y: number): Emitter {
        const em = makeEmitter(spec, x, y)
        handle.emitters!.push(em)
        return em
      },
      step() {},
      destroy() {},
    }

    // Loop driven by the injected clock. Mirrors stage.ts's onTick/onRender:
    // poll gamepads, integrate live entities, step collision, step particles,
    // fan out tick/frame subs. No requestAnimationFrame, no performance.now().
    const clock: FakeClock = {
      tick(dt = 1 / 60) {
        input.pollGamepads()
        if (state !== 'playing') return
        for (const e of world.entities) {
          if (e.dead) continue
          e.x += e.vx * dt
          e.y += e.vy * dt
        }
        const events = collide.step()
        for (const ev of events) for (const fn of collideSubs) fn(ev)
        particles.step()
        for (const fn of tickSubs) fn(dt, tickIndex++)
      },
      render(interp = 0, fps = 60) {
        for (const fn of frameSubs) fn(interp, fps)
      },
    }

    const loop = {
      start() {}, stop() {}, pause() {}, resume() {}, get paused() { return false },
    }

    const stage: Stage = {
      world: world as unknown as Stage['world'],
      input,
      audio: audio as unknown as Stage['audio'],
      palette: palette as unknown as Stage['palette'],
      collide: collide as unknown as Stage['collide'],
      particles: particles as unknown as Stage['particles'],
      loop: loop as unknown as Stage['loop'],
      get state() { return state },
      set state(v: StageState) { setState(v) },
      onState(fn) { stateSubs.add(fn); return () => stateSubs.delete(fn) },
      onCollide(fn) { collideSubs.add(fn); return () => collideSubs.delete(fn) },
      onTick(fn) { tickSubs.add(fn); return () => tickSubs.delete(fn) },
      onFrame(fn) { frameSubs.add(fn); return () => frameSubs.delete(fn) },
      start() { setState('playing') },
      pause() { if (state === 'playing') setState('paused') },
      resume() { if (state === 'paused') setState('playing') },
      reset() {
        for (const e of [...world.entities]) e.destroy()
        setState('idle')
      },
      async destroy() {
        input.destroy()
        stateSubs.clear(); collideSubs.clear(); tickSubs.clear(); frameSubs.clear()
      },
    }

    handle.stage = stage
    handle.clock = clock
    current = handle as FakeStageHandle
    return stage
  }
}
