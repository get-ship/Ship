import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

afterEach(() => resetFakeStage())

describe('<sui-emitter> — declarative particle emitter registered with the stage', () => {
  it('registers as a custom element', () => {
    registerChild('sui-emitter')
    expect(customElements.get('sui-emitter')).toBeDefined()
  })

  it('registers one pooled emitter at its x/y with the parsed spec', async () => {
    await mountStage(
      `<sui-emitter id="boom" kind="explosion" x="120" y="80" rate="0" burst="24"
         lifetime="500" spread="180" direction="-90" speed="200" speed-jitter="0.5"
         gravity="9" color="accent" size="6..1" max="128"></sui-emitter>`,
    )
    const fake = lastFakeStage()
    expect(fake.emitters.length).toBe(1)
    const em = fake.emitters[0]!
    expect(em.x).toBe(120)
    expect(em.y).toBe(80)
    expect(em.spec.kind).toBe('explosion')
    expect(em.spec.burstCount).toBe(24)
    expect(em.spec.lifetime).toBe(500)
    expect(em.spec.spread).toBe(180)
    expect(em.spec.speed).toBe(200)
    expect(em.spec.speedJitter).toBe(0.5)
    expect(em.spec.gravity).toBe(9)
    expect(em.spec.max).toBe(128)
  })

  it('parses size "start..end" into sizeStart/sizeEnd', async () => {
    await mountStage(`<sui-emitter id="s" size="6..1"></sui-emitter>`)
    const em = lastFakeStage().emitters[0]!
    expect(em.spec.sizeStart).toBe(6)
    expect(em.spec.sizeEnd).toBe(1)
  })

  it('resolves the color token through the palette', async () => {
    await mountStage(`<sui-emitter id="c" color="accent"></sui-emitter>`)
    const fake = lastFakeStage()
    expect(fake.resolved).toContain('accent')
    expect(fake.emitters[0]!.spec.color).toBe(0x3366ff)
  })

  it('fires a burst (relocated to detail x/y) when its bound action fires', async () => {
    const { host } = await mountStage(`<sui-emitter id="spark" on="hit" burst="10"></sui-emitter>`)
    const em = lastFakeStage().emitters[0]!

    // No detail -> default burst count (spec.burstCount).
    host.fire('hit')
    expect(em.bursts).toEqual([10])

    // Detail relocates the emitter and overrides the count.
    host.fire('hit', { x: 200, y: 30, count: 4 })
    expect(em.x).toBe(200)
    expect(em.y).toBe(30)
    expect(em.bursts).toEqual([10, 4])
  })

  it('an emitter without an `on` action does not burst on unrelated actions', async () => {
    const { host } = await mountStage(`<sui-emitter id="ambient" rate="20"></sui-emitter>`)
    const em = lastFakeStage().emitters[0]!
    host.fire('hit')
    expect(em.bursts).toEqual([])
  })
})
