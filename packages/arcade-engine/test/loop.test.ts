import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createLoop } from '../src/loop.ts'

// loop.ts reaches for three browser globals: requestAnimationFrame,
// cancelAnimationFrame, performance.now. We replace all three with a
// deterministic fake: a clock we advance by hand and a frame queue we pump by
// hand. No real time, no real rAF — every test is reproducible.

interface FakeRaf {
  /** Advance the clock and invoke the most-recently-scheduled frame callback. */
  step(deltaMs: number): void
  /** Current scheduled callback id, or 0 if none pending. */
  readonly pending: number
}

let now = 0
let nextRafId = 0
let scheduled: ((t: number) => void) | null = null

function installFakeClock(): FakeRaf {
  now = 0
  nextRafId = 0
  scheduled = null
  vi.stubGlobal('performance', { now: () => now })
  vi.stubGlobal('requestAnimationFrame', (cb: (t: number) => void) => {
    scheduled = cb
    return ++nextRafId
  })
  vi.stubGlobal('cancelAnimationFrame', () => {
    scheduled = null
  })
  return {
    step(deltaMs: number) {
      now += deltaMs
      const cb = scheduled
      scheduled = null
      if (cb) cb(now)
    },
    get pending() {
      return scheduled ? nextRafId : 0
    },
  }
}

let raf: FakeRaf
beforeEach(() => { raf = installFakeClock() })
afterEach(() => { vi.unstubAllGlobals() })

describe('loop: fixed-timestep accumulator', () => {
  it('runs exactly one tick per frame at the matched rate', () => {
    const ticks: number[] = []
    // tickRate 50 → tickDt = 0.02s = 20ms exactly, so 20ms frames divide
    // cleanly with no float drift (1000/60 does NOT, which would make the
    // tick-per-frame count jitter by one — not a loop bug, just IEEE-754).
    const loop = createLoop({
      tickRate: 50,
      renderFps: 50,
      onTick: (_dt, idx) => ticks.push(idx),
      onRender: () => {},
    })
    loop.start()
    // start() schedules the first frame but does not run it.
    expect(ticks).toHaveLength(0)
    // Each 20ms frame yields exactly one 50Hz tick.
    raf.step(20)
    raf.step(20)
    raf.step(20)
    expect(ticks).toEqual([0, 1, 2])
  })

  it('monotonically increments tickIndex across frames', () => {
    const ticks: number[] = []
    const loop = createLoop({
      tickRate: 60,
      renderFps: 60,
      onTick: (_dt, idx) => ticks.push(idx),
      onRender: () => {},
    })
    loop.start()
    // Run a second of sloppy 1000/60 frames; regardless of how float drift
    // distributes ticks across frames, the index sequence is contiguous.
    for (let i = 0; i < 60; i++) raf.step(1000 / 60)
    expect(ticks[0]).toBe(0)
    for (let i = 1; i < ticks.length; i++) expect(ticks[i]).toBe(ticks[i - 1]! + 1)
    // ~60 ticks in ~1s of 60Hz frames (allow ±2 for accumulator residue).
    expect(ticks.length).toBeGreaterThanOrEqual(58)
    expect(ticks.length).toBeLessThanOrEqual(60)
  })

  it('always ticks with the FIXED dt, never the frame dt', () => {
    const dts: number[] = []
    const loop = createLoop({
      tickRate: 50, // tickDt = 0.02s
      renderFps: 60,
      onTick: (dt) => dts.push(dt),
      onRender: () => {},
    })
    loop.start()
    raf.step(7)   // 0.007s — below one tick, accumulates
    raf.step(40)  // +0.040s → accum 0.047 → 2 ticks
    expect(dts.every((d) => d === 0.02)).toBe(true)
    expect(dts).toHaveLength(2)
  })

  it('drains a backlog into multiple ticks on a long frame', () => {
    let count = 0
    const loop = createLoop({
      tickRate: 60,
      renderFps: 60,
      onTick: () => { count++ },
      onRender: () => {},
    })
    loop.start()
    raf.step(50) // 0.05s / (1/60≈0.0167) → 3 ticks
    expect(count).toBe(3)
  })

  it('clamps the accumulator to avoid spiral-of-death (max 5 ticks)', () => {
    let count = 0
    const loop = createLoop({
      tickRate: 60,
      renderFps: 60,
      onTick: () => { count++ },
      onRender: () => {},
    })
    loop.start()
    // A 2-second backgrounded-tab frame would be ~120 ticks unclamped;
    // maxAccum = 5*tickDt so at most 5 ticks run.
    raf.step(2000)
    expect(count).toBe(5)
  })
})

describe('loop: render interpolation alpha', () => {
  it('passes leftover-accum / tickDt as interp in [0, 1)', () => {
    const alphas: number[] = []
    const loop = createLoop({
      tickRate: 100, // tickDt = 0.01s = 10ms
      renderFps: 60,
      onTick: () => {},
      onRender: (interp) => alphas.push(interp),
    })
    loop.start()
    raf.step(15) // 1 tick consumed (10ms), 5ms left → alpha 0.5
    expect(alphas).toHaveLength(1)
    expect(alphas[0]).toBeCloseTo(0.5)
    raf.step(10) // 5ms carried + 10ms = 15ms → 1 tick, 5ms left → 0.5 again
    expect(alphas[1]).toBeCloseTo(0.5)
  })

  it('reports alpha 0 when the frame lands exactly on a tick boundary', () => {
    const alphas: number[] = []
    const loop = createLoop({
      tickRate: 100,
      renderFps: 60,
      onTick: () => {},
      onRender: (interp) => alphas.push(interp),
    })
    loop.start()
    raf.step(10) // exactly one tick, zero remainder
    expect(alphas[0]).toBeCloseTo(0)
  })
})

describe('loop: start / stop / pause / resume', () => {
  it('start is idempotent — a second start does not double-schedule', () => {
    const loop = createLoop({ tickRate: 60, renderFps: 60, onTick: () => {}, onRender: () => {} })
    loop.start()
    const first = raf.pending
    loop.start()
    // No new frame queued by the second start (still the one rescheduled by the
    // frame callback, which hasn't run yet).
    expect(raf.pending).toBe(first)
  })

  it('stop halts ticking', () => {
    let count = 0
    const loop = createLoop({ tickRate: 60, renderFps: 60, onTick: () => { count++ }, onRender: () => {} })
    loop.start()
    raf.step(1000 / 60)
    expect(count).toBe(1)
    loop.stop()
    raf.step(1000 / 60) // no frame scheduled after cancel → no-op
    expect(count).toBe(1)
  })

  it('pause freezes ticking but keeps the rAF chain alive', () => {
    let count = 0
    const loop = createLoop({ tickRate: 60, renderFps: 60, onTick: () => { count++ }, onRender: () => {} })
    loop.start()
    raf.step(1000 / 60)
    expect(count).toBe(1)
    loop.pause()
    expect(loop.paused).toBe(true)
    raf.step(1000 / 60) // paused frame: reschedules, advances lastTime, no tick
    raf.step(1000 / 60)
    expect(count).toBe(1)
  })

  it('resume does not replay backlog accumulated while paused', () => {
    let count = 0
    const loop = createLoop({ tickRate: 60, renderFps: 60, onTick: () => { count++ }, onRender: () => {} })
    loop.start()
    raf.step(1000 / 60)
    expect(count).toBe(1)
    loop.pause()
    raf.step(5000) // 5s wall-time passes while paused — must NOT bank ticks
    loop.resume()
    expect(loop.paused).toBe(false)
    raf.step(1000 / 60) // one fresh frame after resume → exactly one tick
    expect(count).toBe(2)
  })
})

describe('loop: fps sampling', () => {
  it('does not report fps until the 0.5s sample window elapses, then reports', () => {
    const fpsSamples: number[] = []
    const loop = createLoop({
      tickRate: 50,
      renderFps: 50,
      onTick: () => {},
      onRender: (_i, fps) => fpsSamples.push(fps),
    })
    loop.start()
    // 20ms frames: the window (>= 0.5s) closes on the 25th frame.
    for (let i = 0; i < 50; i++) raf.step(20)
    expect(fpsSamples[0]).toBe(0)
    // By the time the window closes, a nonzero fps is reported.
    expect(fpsSamples.some((f) => f > 0)).toBe(true)
    // Steady 50Hz frames → measured fps close to 50.
    const last = fpsSamples.filter((f) => f > 0).at(-1)!
    expect(last).toBeGreaterThan(45)
    expect(last).toBeLessThan(55)
  })
})
