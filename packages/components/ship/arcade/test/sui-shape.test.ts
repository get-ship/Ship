import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage, entityInit } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

interface ShapeVisual {
  kind: string
  shape: string
  width?: number
  height?: number
  radius?: number
  points?: number[]
  fill?: number
  stroke?: number
  strokeWidth?: number
}

afterEach(() => resetFakeStage())

describe('<sui-shape> — declarative geometric entity spawned by the stage', () => {
  it('registers as a custom element', () => {
    registerChild('sui-shape')
    expect(customElements.get('sui-shape')).toBeDefined()
  })

  it('reflects rect dimensions + resolves the fill token through the palette', async () => {
    await mountStage(
      `<sui-shape kind="rect" tag="box" x="4" y="6" width="40" height="24" fill="accent"></sui-shape>`,
    )
    const fake = lastFakeStage()
    const e = [...fake.stage.world.entities].find((x) => x.tag === 'box')!
    const v = entityInit(e)!.visual as unknown as ShapeVisual
    expect(v.kind).toBe('shape')
    expect(v.shape).toBe('rect')
    expect(v.width).toBe(40)
    expect(v.height).toBe(24)
    // `accent` was resolved by the palette (fake maps it to 0x3366ff).
    expect(fake.resolved).toContain('accent')
    expect(v.fill).toBe(0x3366ff)
  })

  it('parses a polygon points list into numbers', async () => {
    await mountStage(
      `<sui-shape kind="polygon" tag="tri" points="0,0 10,0 5,8" fill="foreground"></sui-shape>`,
    )
    const e = [...lastFakeStage().stage.world.entities].find((x) => x.tag === 'tri')!
    const v = entityInit(e)!.visual as unknown as ShapeVisual
    expect(v.shape).toBe('polygon')
    expect(v.points).toEqual([0, 0, 10, 0, 5, 8])
  })

  it('resolves a stroke token only when stroke is set', async () => {
    await mountStage(
      `<sui-shape kind="circle" tag="dot" radius="9" fill="card" stroke="surface-fg" stroke-width="3"></sui-shape>`,
    )
    const e = [...lastFakeStage().stage.world.entities].find((x) => x.tag === 'dot')!
    const v = entityInit(e)!.visual as unknown as ShapeVisual
    expect(v.shape).toBe('circle')
    expect(v.radius).toBe(9)
    expect(v.stroke).toBe(0xdddddd) // surface-fg
    expect(v.strokeWidth).toBe(3)
  })

  it('a shape-with-collider falls back to the shape width/height for an unsized collider', async () => {
    // children.ts: an aabb collider with width=0 inherits the shape's width.
    // Two same-spot 30x30 shapes whose colliders are declared with no size
    // must still collide once the tick runs collision.
    const { host } = await mountStage(`
      <sui-shape kind="rect" tag="a" x="50" y="50" width="30" height="30">
        <sui-collider kind="aabb" group="a" collides-with="b"></sui-collider>
      </sui-shape>
      <sui-shape kind="rect" tag="b" x="50" y="50" width="30" height="30">
        <sui-collider kind="aabb" group="b" collides-with="a"></sui-collider>
      </sui-shape>
    `)
    const { clock } = lastFakeStage()
    let hits = 0
    host.addEventListener('sail-collide', () => { hits++ })
    host.start()
    clock.tick()
    expect(hits).toBe(1)
  })
})
