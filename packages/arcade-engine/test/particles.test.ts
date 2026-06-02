import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { pixiMockModule, pixiMockState } from './_pixi-mock.ts'

// particles.ts `import * as PIXI from '../vendor/pixi.js/pixi.mjs'` — mock it so
// the 800 KB renderer never loads; we only need Graphics/Container stubs that
// record transforms. spawnParticle reads Math.random for angle/jitter; we drive
// that with a SEEDED PRNG (mulberry32) so spawn/age/expire is reproducible —
// never the global RNG (banned in this test env).
vi.mock('../vendor/pixi.js/pixi.mjs', () => pixiMockModule())

import { createParticles, type EmitterSpec } from '../src/particles.ts'

/** Deterministic, seedable [0,1) source — replaces Math.random in tests. */
function mulberry32(seed: number): () => number {
  let a = seed >>> 0
  return () => {
    a |= 0
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

function seedRandom(seed: number) {
  const rng = mulberry32(seed)
  vi.spyOn(Math, 'random').mockImplementation(rng)
}

function fakeWorld() {
  const mod = pixiMockModule()
  const app = new mod.Application()
  return { app }
}

const baseSpec = (over: Partial<EmitterSpec> = {}): EmitterSpec => ({
  kind: 'radial',
  rate: 0,
  burstCount: 5,
  lifetime: 1000, // ms → 1s life
  spread: 60,
  direction: 0,
  speed: 100,
  speedJitter: 0,
  gravity: 0,
  color: 0xff0000,
  sizeStart: 4,
  sizeEnd: 0,
  max: 16,
  ...over,
})

beforeEach(() => { pixiMockState.reset() })
afterEach(() => { vi.restoreAllMocks() })

function aliveCount(container: { children: { visible: boolean }[] }): number {
  return container.children.filter((c) => c.visible).length
}

describe('particles: emitter pool', () => {
  it('pre-allocates `max` hidden particle nodes', () => {
    seedRandom(1)
    const sys = createParticles()
    const world = fakeWorld()
    sys.add(world, baseSpec({ max: 8 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    expect(container.children).toHaveLength(8)
    expect(aliveCount(container)).toBe(0) // all start hidden
  })

  it('burst spawns exactly burstCount particles, capped by pool size', () => {
    seedRandom(7)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ burstCount: 5, max: 16 }), 10, 20)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    em.burst()
    expect(aliveCount(container)).toBe(5)
  })

  it('burst caps at pool max when asked for more than `max`', () => {
    seedRandom(3)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ max: 4 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    em.burst(100) // request 100, only 4 slots
    expect(aliveCount(container)).toBe(4)
  })

  it('burst(n) overrides burstCount', () => {
    seedRandom(11)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ burstCount: 5, max: 16 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    em.burst(3)
    expect(aliveCount(container)).toBe(3)
  })
})

describe('particles: age + expire over N steps', () => {
  it('expires every particle after its lifetime elapses', () => {
    seedRandom(42)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ lifetime: 500, max: 8 }), 0, 0) // 0.5s life
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    em.burst(4)
    expect(aliveCount(container)).toBe(4)
    // 5 steps of 0.1s = 0.5s → age >= life → all expire.
    for (let i = 0; i < 5; i++) sys.step(0.1)
    expect(aliveCount(container)).toBe(0)
  })

  it('keeps particles alive until just before expiry', () => {
    seedRandom(9)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ lifetime: 1000, max: 8 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    em.burst(3)
    for (let i = 0; i < 9; i++) sys.step(0.1) // 0.9s < 1s → still alive
    expect(aliveCount(container)).toBe(3)
    // One more 0.1s lands at ~0.9999 (IEEE-754 sum of 0.1s never hits exactly
    // 1.0), so the engine keeps them alive — faithful, not a bug.
    sys.step(0.1)
    expect(aliveCount(container)).toBe(3)
    sys.step(0.1) // crosses life → expires
    expect(aliveCount(container)).toBe(0)
  })

  it('fades alpha from 1 toward 0 across the lifetime', () => {
    seedRandom(5)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ lifetime: 1000, max: 4 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean; alpha: number }[] }
    em.burst(1)
    sys.step(0.25) // t = 0.25 → alpha 0.75
    const p = container.children.find((c) => c.visible)!
    expect(p.alpha).toBeCloseTo(0.75)
  })
})

describe('particles: continuous rate emission', () => {
  it('spawns rate*dt particles per second from the accumulator', () => {
    seedRandom(13)
    const sys = createParticles()
    const world = fakeWorld()
    sys.add(world, baseSpec({ rate: 10, lifetime: 100000, max: 64 }), 0, 0) // long life so none expire
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    // rate 10/s over 1s of 0.1s steps → ~10 particles.
    for (let i = 0; i < 10; i++) sys.step(0.1)
    expect(aliveCount(container)).toBe(10)
  })

  it('does not emit on rate when rate is 0 (burst-only emitter)', () => {
    seedRandom(2)
    const sys = createParticles()
    const world = fakeWorld()
    sys.add(world, baseSpec({ rate: 0, max: 16 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[] }
    for (let i = 0; i < 20; i++) sys.step(0.1)
    expect(aliveCount(container)).toBe(0)
  })
})

describe('particles: determinism by seed', () => {
  it('produces identical kinematics for the same seed, different for another', () => {
    function run(seed: number): number[] {
      const restore = vi.spyOn(Math, 'random').mockImplementation(mulberry32(seed))
      const sys = createParticles()
      const world = fakeWorld()
      const em = sys.add(world, baseSpec({ kind: 'radial', speed: 100, max: 4 }), 0, 0)
      const container = world.app.stage.children[0] as { children: { visible: boolean; x: number; y: number }[] }
      em.burst(2)
      sys.step(0.1)
      const out = container.children.filter((c) => c.visible).flatMap((c) => [c.x, c.y])
      restore.mockRestore()
      return out
    }
    const a1 = run(100)
    const a2 = run(100)
    const b = run(999)
    expect(a1).toEqual(a2) // same seed → identical positions
    expect(a1).not.toEqual(b) // different seed → different positions
  })
})

describe('particles: kinematics + lifecycle', () => {
  it('applies gravity to vy and integrates position', () => {
    seedRandom(1)
    const sys = createParticles()
    const world = fakeWorld()
    // direction 0 (along +x), no jitter, gravity pulls +y over time.
    const em = sys.add(world, baseSpec({ kind: 'cone', spread: 0, direction: 0, speed: 100, speedJitter: 0, gravity: 200, lifetime: 100000, max: 4 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean; x: number; y: number }[] }
    em.burst(1)
    sys.step(0.1)
    const p = container.children.find((c) => c.visible)!
    // vx = cos(0)*100 = 100 → x ≈ 10 after 0.1s; gravity bends y downward.
    expect(p.x).toBeCloseTo(10)
    expect(p.y).toBeGreaterThan(0)
  })

  it('setColor repaints every pooled particle', () => {
    seedRandom(1)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ max: 4 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { ops: string[] }[] }
    em.setColor(0x00ff00)
    // Each particle gets clear + circle + fill recorded.
    for (const c of container.children) {
      expect(c.ops).toContain('clear')
      expect(c.ops.filter((o) => o === 'fill').length).toBeGreaterThan(0)
    }
  })

  it('destroy() removes the emitter from the system', () => {
    seedRandom(1)
    const sys = createParticles()
    const world = fakeWorld()
    const em = sys.add(world, baseSpec({ rate: 10, lifetime: 100000, max: 16 }), 0, 0)
    const container = world.app.stage.children[0] as { children: { visible: boolean }[]; destroyed: boolean }
    em.destroy()
    expect(container.destroyed).toBe(true)
    // After destroy, stepping must not resurrect particles in that container.
    for (let i = 0; i < 10; i++) sys.step(0.1)
    expect(aliveCount(container)).toBe(0)
  })

  it('system destroy() tears down all emitters', () => {
    seedRandom(1)
    const sys = createParticles()
    const world = fakeWorld()
    const c1 = sys.add(world, baseSpec(), 0, 0)
    const c2 = sys.add(world, baseSpec(), 0, 0)
    expect(world.app.stage.children).toHaveLength(2)
    sys.destroy()
    void c1
    void c2
    expect((world.app.stage.children[0] as { destroyed: boolean }).destroyed).toBe(true)
    expect((world.app.stage.children[1] as { destroyed: boolean }).destroyed).toBe(true)
  })
})
