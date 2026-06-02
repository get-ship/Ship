import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

afterEach(() => {
  resetFakeStage()
  vi.restoreAllMocks()
})

describe('<sui-sfx> — one-shot audio binding triggered by a stage action', () => {
  it('registers as a custom element', () => {
    registerChild('sui-sfx')
    expect(customElements.get('sui-sfx')).toBeDefined()
  })

  it('preloads its src at wire time', async () => {
    await mountStage(`<sui-sfx src="/jump.wav" on="jump"></sui-sfx>`)
    expect(lastFakeStage().preloads).toContain('/jump.wav')
  })

  it('plays the clip on the bound action with the declared volume + bus', async () => {
    const { host } = await mountStage(
      `<sui-sfx src="/coin.wav" on="pickup" volume="0.6" bus="ui"></sui-sfx>`,
    )
    const fake = lastFakeStage()
    expect(fake.plays.length).toBe(0)

    host.fire('pickup')
    expect(fake.plays.length).toBe(1)
    expect(fake.plays[0]!.src).toBe('/coin.wav')
    expect(fake.plays[0]!.volume).toBe(0.6)
    expect(fake.plays[0]!.bus).toBe('ui')
  })

  it('a fixed pitch-range (1..1) plays at exactly pitch 1 — RNG-independent', async () => {
    // lo === hi, so `lo + rand*(hi-lo)` collapses to lo regardless of any RNG.
    const { host } = await mountStage(`<sui-sfx src="/x.wav" on="hit" pitch-range="1..1"></sui-sfx>`)
    host.fire('hit')
    expect(lastFakeStage().plays[0]!.pitch).toBe(1)
  })

  it('samples a pitch inside the declared range from an injected RNG', async () => {
    // Drive the random source deterministically (never the unseeded global):
    // with Math.random() pinned to 0.5 and range 0.8..1.2, pitch === 1.0.
    vi.spyOn(Math, 'random').mockReturnValue(0.5)
    const { host } = await mountStage(`<sui-sfx src="/v.wav" on="boom" pitch-range="0.8..1.2"></sui-sfx>`)
    host.fire('boom')
    expect(lastFakeStage().plays[0]!.pitch).toBeCloseTo(1.0) // 0.8 + 0.5*(0.4)
  })

  it('does not play when an unrelated action fires', async () => {
    const { host } = await mountStage(`<sui-sfx src="/a.wav" on="left"></sui-sfx>`)
    host.fire('right')
    expect(lastFakeStage().plays.length).toBe(0)
  })
})
