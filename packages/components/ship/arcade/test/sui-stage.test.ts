import { afterEach, describe, expect, it, vi } from 'vitest'

// Replace the engine boundary with the deterministic, injected-clock double
// BEFORE the controller (which imports `createStage`) is loaded.
import { makeFakeCreateStage, lastFakeStage, resetFakeStage } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage } from './harness.ts'

afterEach(() => resetFakeStage())

describe('<sui-stage> — registration', () => {
  it('registers the custom element when its .sui compiles + mounts', async () => {
    await mountStage()
    expect(customElements.get('sui-stage')).toBeDefined()
  })
})

describe('<sui-stage> — controller exposes the imperative host API', () => {
  it('attaches start/pause/resume/reset/fire/bind/set + stage handle on mount', async () => {
    const { host } = await mountStage()
    expect(typeof host.start).toBe('function')
    expect(typeof host.pause).toBe('function')
    expect(typeof host.resume).toBe('function')
    expect(typeof host.reset).toBe('function')
    expect(typeof host.fire).toBe('function')
    expect(typeof host.bind).toBe('function')
    expect(typeof host.set).toBe('function')
    expect(host.stage).toBeDefined()
  })

  it('removes the API + stage handle on unmount (disconnect)', async () => {
    const { handle, host } = await mountStage()
    handle.unmount()
    expect(host.start).toBeUndefined()
    expect(host.stage).toBeUndefined()
  })
})

describe('<sui-stage> — lifecycle drives stage state + emits sui-stage:state', () => {
  it('start() -> playing, pause() -> paused, resume() -> playing, reset() -> idle', async () => {
    const { host } = await mountStage()
    const stage = lastFakeStage().stage
    const seen: string[] = []
    host.addEventListener('sui-stage:state', (e) => {
      seen.push((e as CustomEvent<{ state: string }>).detail.state)
    })

    host.start()
    expect(stage.state).toBe('playing')
    host.pause()
    expect(stage.state).toBe('paused')
    host.resume()
    expect(stage.state).toBe('playing')
    host.reset()
    expect(stage.state).toBe('idle')

    expect(seen).toEqual(['playing', 'paused', 'playing', 'idle'])
  })

  it('emits sui-stage:ready once the scene is built', async () => {
    let ready = false
    // onHost attaches the listener before the async createStage IIFE settles.
    await mountStage('', {}, (host) => {
      host.addEventListener('sui-stage:ready', () => { ready = true })
    })
    expect(ready).toBe(true)
  })
})

describe('<sui-stage> — drives a deterministic tick + emits sui-stage:tick', () => {
  it('integrates a moving entity by vx*dt per injected tick (no real frame timer)', async () => {
    // A <sui-shape> child with velocity is spawned into the world; ticking the
    // injected clock must advance its position exactly vx*dt — proving the
    // game loop is wired and driven by the injected clock, not wall time.
    const { host } = await mountStage(
      `<sui-shape kind="rect" tag="mover" x="0" y="0" vx="60" vy="0" width="8" height="8"></sui-shape>`,
    )
    const { stage, clock } = lastFakeStage()
    const ticks: number[] = []
    host.addEventListener('sui-stage:tick', (e) => {
      ticks.push((e as CustomEvent<{ idx: number }>).detail.idx)
    })

    const mover = [...stage.world.entities].find((e) => e.tag === 'mover')
    expect(mover).toBeDefined()
    expect(mover!.x).toBe(0)

    host.start()
    clock.tick(0.5) // dt = 0.5s
    expect(mover!.x).toBeCloseTo(30) // 60 * 0.5
    clock.tick(0.5)
    expect(mover!.x).toBeCloseTo(60)

    // tick subs fired with monotonically increasing indices.
    expect(ticks).toEqual([0, 1])
  })

  it('does not integrate while idle (loop only steps when playing)', async () => {
    const { host } = await mountStage(
      `<sui-shape kind="rect" tag="mover" x="5" y="0" vx="100" vy="0"></sui-shape>`,
    )
    const { stage, clock } = lastFakeStage()
    const mover = [...stage.world.entities].find((e) => e.tag === 'mover')!
    clock.tick(1) // never started -> idle -> no integration
    expect(mover.x).toBe(5)
    void host
  })
})

describe('<sui-stage> — registers light-DOM children via children.ts', () => {
  it('spawns one world entity per sui-sprite / sui-shape child', async () => {
    const { host } = await mountStage(`
      <sui-shape kind="rect" tag="wall" x="10" y="20" width="4" height="4"></sui-shape>
      <sui-sprite src="/x.png" tag="hero" x="1" y="2"></sui-sprite>
    `)
    const tags = [...lastFakeStage().stage.world.entities].map((e) => e.tag).sort()
    expect(tags).toEqual(['hero', 'wall'])
    void host
  })

  it('binds sui-stage-input key maps + sui-sfx audio bindings into the stage buses', async () => {
    const { host } = await mountStage(`
      <sui-stage-input map="Space" action="jump"></sui-stage-input>
      <sui-sfx src="/boom.wav" on="jump" volume="0.8" bus="sfx"></sui-sfx>
    `)
    const fake = lastFakeStage()
    // sfx preloads its src at wire time.
    expect(fake.preloads).toContain('/boom.wav')

    // Firing the bound action through the stage's input bus plays the sfx.
    fake.stage.input.fire('jump')
    expect(fake.plays.length).toBe(1)
    expect(fake.plays[0]!.src).toBe('/boom.wav')
    expect(fake.plays[0]!.volume).toBe(0.8)
    expect(fake.plays[0]!.bus).toBe('sfx')
    void host
  })

  it('registers a sui-emitter and fires its burst on the bound action', async () => {
    const { host } = await mountStage(`
      <sui-emitter id="spark" kind="explosion" on="hit" burst="12"></sui-emitter>
    `)
    const fake = lastFakeStage()
    expect(fake.emitters.length).toBe(1)

    host.fire('hit', { x: 50, y: 60, count: 7 })
    const em = fake.emitters[0]!
    expect(em.x).toBe(50)
    expect(em.y).toBe(60)
    expect(em.bursts).toEqual([7])
  })
})
