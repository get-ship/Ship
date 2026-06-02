import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { pixiMockModule, pixiMockState } from './_pixi-mock.ts'

// stage.ts wires world + loop + collide + input + audio + particles + palette
// into one handle. It pulls a lot of browser surface: Pixi (mocked), rAF +
// performance (faked clock so ticks are deterministic), and the palette's
// document/getComputedStyle/MutationObserver (stubbed minimally). With those in
// place we can exercise the real orchestration: state machine, per-tick physics
// integration, collide-event fanout, and teardown — no renderer, no real time.
vi.mock('../vendor/pixi.js/pixi.mjs', () => pixiMockModule())

let now = 0
let scheduled: ((t: number) => void) | null = null
function stepFrame(deltaMs: number) {
  now += deltaMs
  const cb = scheduled
  scheduled = null
  if (cb) cb(now)
}

function installEnv() {
  now = 0
  scheduled = null
  pixiMockState.reset()
  vi.stubGlobal('performance', { now: () => now })
  vi.stubGlobal('requestAnimationFrame', (cb: (t: number) => void) => { scheduled = cb; return 1 })
  vi.stubGlobal('cancelAnimationFrame', () => { scheduled = null })
  // Palette deps. No real window → input binds to host (an EventTarget).
  vi.stubGlobal('getComputedStyle', () => ({ getPropertyValue: () => '' }))
  vi.stubGlobal('document', { documentElement: {}, createElement: () => ({ getContext: () => null }) })
  class FakeMO { observe() {} disconnect() {} }
  vi.stubGlobal('MutationObserver', FakeMO)
}

beforeEach(() => { installEnv() })
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks() })

import { createStage, type Stage } from '../src/stage.ts'

function host(): HTMLElement {
  // EventTarget gives addEventListener/removeEventListener for the input bus.
  return new EventTarget() as unknown as HTMLElement
}

async function makeStage(over: Partial<Parameters<typeof createStage>[0]> = {}): Promise<Stage> {
  return createStage({
    host: host(),
    canvas: { replaceWith() {} } as unknown as HTMLCanvasElement,
    width: 320,
    height: 240,
    tickRate: 50, // 20ms ticks → clean stepping
    fps: 50,
    ...over,
  })
}

describe('stage: state machine', () => {
  it('starts idle and transitions to playing on start()', async () => {
    const stage = await makeStage()
    const states: string[] = []
    stage.onState((s) => states.push(s))
    expect(stage.state).toBe('idle')
    stage.start()
    expect(stage.state).toBe('playing')
    expect(states).toEqual(['playing'])
  })

  it('pause → paused, resume → playing (only from the matching state)', async () => {
    const stage = await makeStage()
    stage.start()
    stage.pause()
    expect(stage.state).toBe('paused')
    stage.resume()
    expect(stage.state).toBe('playing')
    // resume from playing is a no-op for state.
    stage.resume()
    expect(stage.state).toBe('playing')
  })

  it('does not emit duplicate state events for the same state', async () => {
    const stage = await makeStage()
    let count = 0
    stage.onState(() => count++)
    stage.state = 'playing'
    stage.state = 'playing' // same → no emit
    expect(count).toBe(1)
  })

  it('onState returns an unsubscribe', async () => {
    const stage = await makeStage()
    let count = 0
    const off = stage.onState(() => count++)
    stage.state = 'playing'
    off()
    stage.state = 'paused'
    expect(count).toBe(1)
  })
})

describe('stage: per-tick physics integration', () => {
  it('integrates entity velocity into position while playing', async () => {
    const stage = await makeStage()
    const e = stage.world.spawn({ visual: { kind: 'container' }, x: 0, y: 0, vx: 100, vy: 50 })
    stage.start()
    stepFrame(20) // one 20ms (0.02s) tick → x += 100*0.02 = 2, y += 50*0.02 = 1
    expect(e.x).toBeCloseTo(2)
    expect(e.y).toBeCloseTo(1)
  })

  it('does not integrate while paused', async () => {
    const stage = await makeStage()
    const e = stage.world.spawn({ visual: { kind: 'container' }, x: 0, y: 0, vx: 100, vy: 0 })
    stage.start()
    stepFrame(20)
    const afterOne = e.x
    stage.pause()
    stepFrame(1000) // lots of wall time, but paused → frozen
    expect(e.x).toBe(afterOne)
  })

  it('fires onTick with the fixed dt while playing', async () => {
    const stage = await makeStage()
    const dts: number[] = []
    stage.onTick((dt) => dts.push(dt))
    stage.start()
    stepFrame(20)
    stepFrame(20)
    expect(dts).toEqual([0.02, 0.02])
  })

  it('fires onFrame each rendered frame', async () => {
    const stage = await makeStage()
    let frames = 0
    stage.onFrame(() => frames++)
    stage.start()
    stepFrame(20)
    stepFrame(20)
    expect(frames).toBe(2)
  })
})

describe('stage: collision fanout', () => {
  it('emits collide events for registered overlapping colliders during play', async () => {
    const stage = await makeStage()
    const a = stage.world.spawn({ visual: { kind: 'container' }, x: 0, y: 0 })
    const b = stage.world.spawn({ visual: { kind: 'container' }, x: 4, y: 0 })
    stage.collide.add(a, { kind: 'aabb', w: 10, h: 10, group: 'p', collidesWith: ['e'] })
    stage.collide.add(b, { kind: 'aabb', w: 10, h: 10, group: 'e', collidesWith: ['p'] })
    const hits: unknown[] = []
    stage.onCollide((ev) => hits.push(ev))
    stage.start()
    stepFrame(20)
    expect(hits.length).toBeGreaterThanOrEqual(1)
  })

  it('does not run collision while idle (start not called)', async () => {
    const stage = await makeStage()
    const a = stage.world.spawn({ visual: { kind: 'container' }, x: 0, y: 0 })
    const b = stage.world.spawn({ visual: { kind: 'container' }, x: 4, y: 0 })
    stage.collide.add(a, { kind: 'aabb', w: 10, h: 10, group: 'p', collidesWith: ['e'] })
    stage.collide.add(b, { kind: 'aabb', w: 10, h: 10, group: 'e', collidesWith: ['p'] })
    let hits = 0
    stage.onCollide(() => hits++)
    // No start() → loop never runs.
    stepFrame(20)
    expect(hits).toBe(0)
  })
})

describe('stage: reset + teardown', () => {
  it('reset destroys all entities, flushes, and returns to idle', async () => {
    const stage = await makeStage()
    stage.world.spawn({ visual: { kind: 'container' } })
    stage.world.spawn({ visual: { kind: 'container' } })
    stage.start()
    expect(stage.world.entities.length).toBe(2)
    stage.reset()
    expect(stage.world.entities.length).toBe(0)
    expect(stage.state).toBe('idle')
  })

  it('destroy tears down subsystems and the Pixi app', async () => {
    const stage = await makeStage()
    stage.start()
    await stage.destroy()
    const app = pixiMockState.apps[0]! as unknown as { destroyed: boolean }
    expect(app.destroyed).toBe(true)
    // Loop stopped: a frame after destroy must not tick.
    let ticked = false
    stage.onTick(() => { ticked = true })
    stepFrame(20)
    expect(ticked).toBe(false)
  })

  it('renders through the world each frame (world.app.render called)', async () => {
    const stage = await makeStage()
    stage.start()
    const app = stage.world.app as { renderCount: number }
    stepFrame(20)
    stepFrame(20)
    expect(app.renderCount).toBe(2)
  })
})
