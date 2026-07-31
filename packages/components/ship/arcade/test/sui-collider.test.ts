import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

afterEach(() => resetFakeStage())

describe('<sui-collider> — declares a hitbox on its parent entity', () => {
  it('registers as a custom element', () => {
    registerChild('sui-collider')
    expect(customElements.get('sui-collider')).toBeDefined()
  })

  it('reports an overlap of two grouped entities to the stage as sui-collide', async () => {
    // Two AABB shapes whose colliders mutually test groups. They start at the
    // same point (fully overlapping) so collide.step() — the REAL engine
    // narrow-phase — emits a CollideEvent, which the stage forwards as a DOM
    // `sui-collide`. Driving the injected tick is what runs collision.
    const { host } = await mountStage(`
      <sui-shape kind="rect" tag="ball" x="100" y="100" width="20" height="20">
        <sui-collider kind="aabb" width="20" height="20" group="ball" collides-with="wall"></sui-collider>
      </sui-shape>
      <sui-shape kind="rect" tag="brick" x="100" y="100" width="20" height="20">
        <sui-collider kind="aabb" width="20" height="20" group="wall" collides-with="ball"></sui-collider>
      </sui-shape>
    `)
    const { clock } = lastFakeStage()

    const hits: Array<{ aTag?: string; bTag?: string }> = []
    host.addEventListener('sail-collide', (e) => {
      const d = (e as CustomEvent).detail as { a: { tag?: string }; b: { tag?: string } }
      hits.push({ aTag: d.a.tag, bTag: d.b.tag })
    })

    host.start()
    clock.tick() // runs collide.step() -> emits a CollideEvent for the overlap

    expect(hits.length).toBe(1)
    const tags = [hits[0]!.aTag, hits[0]!.bTag].sort()
    expect(tags).toEqual(['ball', 'brick'])
  })

  it('does NOT report overlap when the colliders are far apart', async () => {
    const { host } = await mountStage(`
      <sui-shape kind="rect" tag="ball" x="0" y="0" width="10" height="10">
        <sui-collider kind="aabb" width="10" height="10" group="ball" collides-with="wall"></sui-collider>
      </sui-shape>
      <sui-shape kind="rect" tag="brick" x="500" y="500" width="10" height="10">
        <sui-collider kind="aabb" width="10" height="10" group="wall" collides-with="ball"></sui-collider>
      </sui-shape>
    `)
    const { clock } = lastFakeStage()
    let count = 0
    host.addEventListener('sail-collide', () => { count++ })
    host.start()
    clock.tick()
    expect(count).toBe(0)
  })

  it('separated entities collide only after motion brings them together', async () => {
    // Genuine end-to-end: a moving ball integrates toward a static brick under
    // the injected clock; no collision until its position overlaps the brick.
    const { host } = await mountStage(`
      <sui-shape kind="rect" tag="ball" x="0" y="0" width="20" height="20" vx="100">
        <sui-collider kind="aabb" width="20" height="20" group="ball" collides-with="wall"></sui-collider>
      </sui-shape>
      <sui-shape kind="rect" tag="brick" x="100" y="0" width="20" height="20">
        <sui-collider kind="aabb" width="20" height="20" group="wall" collides-with="ball"></sui-collider>
      </sui-shape>
    `)
    const { clock } = lastFakeStage()
    let count = 0
    host.addEventListener('sail-collide', () => { count++ })

    host.start()
    clock.tick(0.1) // ball x: 0 -> 10, edges 20 vs 90 -> no overlap
    expect(count).toBe(0)
    clock.tick(0.5) // ball x: 10 -> 60, edges 70 vs 90 ... still apart
    expect(count).toBe(0)
    clock.tick(0.4) // ball x: 60 -> 100, now centered on the brick -> overlap
    expect(count).toBe(1)
  })
})
