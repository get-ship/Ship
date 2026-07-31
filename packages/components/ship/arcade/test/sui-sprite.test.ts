import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage, entityInit } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

const srcSprite = () =>
  readFileSync(resolve(process.cwd(), 'src/sui-sprite/sui-sprite.sail'), 'utf8')

interface SpriteVisual {
  kind: string
  src?: string
  anchor?: [number, number]
  frames?: [number, number]
  frame?: number
}

afterEach(() => resetFakeStage())

describe('<sui-sprite> — declarative image entity spawned by the stage', () => {
  it('registers as a custom element', () => {
    registerChild('sui-sprite')
    expect(customElements.get('sui-sprite')).toBeDefined()
  })

  it('reflects the declared reflect props (src/tag/group) prop->attr', async () => {
    // src/group/tag are `reflect`: writing the JS property must mirror to the
    // attribute (so the stage's querySelector-based child parse sees it).
    const { renderComponent } = await import('@ship/test-setup')
    const handle = renderComponent(srcSprite(), 'sui-sprite', {})
    const el = handle.el as HTMLElement & { src: string; tag: string; group: string }
    el.src = '/h.png'
    el.tag = 'hero'
    el.group = 'players'
    expect(el.getAttribute('src')).toBe('/h.png')
    expect(el.getAttribute('tag')).toBe('hero')
    expect(el.getAttribute('group')).toBe('players')
    handle.unmount()
  })

  it('reflects its position/velocity/transform props onto the spawned entity', async () => {
    await mountStage(
      `<sui-sprite src="/hero.png" tag="hero" group="players"
         x="32" y="48" z="2" vx="5" vy="-3" scale="2" rotation="1.5" alpha="0.5"></sui-sprite>`,
    )
    const e = [...lastFakeStage().stage.world.entities].find((x) => x.tag === 'hero')!
    expect(e).toBeDefined()
    expect(e.x).toBe(32)
    expect(e.y).toBe(48)
    expect(e.z).toBe(2)
    expect(e.vx).toBe(5)
    expect(e.vy).toBe(-3)
    expect(e.scale).toBe(2)
    expect(e.rotation).toBe(1.5)
    expect(e.alpha).toBe(0.5)
    expect(e.group).toBe('players')
  })

  it('reflects src + frame + sheet grid into a sprite visual init', async () => {
    await mountStage(
      `<sui-sprite src="/sheet.png" tag="anim" frames="4x2" frame="3" anchor="top-left"></sui-sprite>`,
    )
    const e = [...lastFakeStage().stage.world.entities].find((x) => x.tag === 'anim')!
    const visual = entityInit(e)!.visual as unknown as SpriteVisual
    expect(visual.kind).toBe('sprite')
    expect(visual.src).toBe('/sheet.png')
    expect(visual.frame).toBe(3)
    expect(visual.frames).toEqual([4, 2]) // "4x2" parsed into [cols, rows]
    expect(visual.anchor).toEqual([0, 0]) // "top-left" anchor preset
  })

  it('moves under the injected tick by vx/vy*dt (entity is live in the world)', async () => {
    const { host } = await mountStage(
      `<sui-sprite src="/x.png" tag="mover" x="0" y="0" vx="10" vy="20"></sui-sprite>`,
    )
    const { stage, clock } = lastFakeStage()
    const e = [...stage.world.entities].find((x) => x.tag === 'mover')!
    host.start()
    clock.tick(0.5)
    expect(e.x).toBeCloseTo(5)
    expect(e.y).toBeCloseTo(10)
  })
})
