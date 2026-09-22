import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { pixiMockModule, pixiMockState } from './_pixi-mock.ts'

// world.ts `import * as PIXI from 'pixi.js'` and does a full renderer init.
// We mock Pixi so no WebGL/canvas is needed; createWorld stays async (it
// awaits app.init), so tests await it. The placeholder canvas only needs
// replaceWith (world swaps it for Pixi's canvas).
vi.mock('pixi.js', () => pixiMockModule())

import { createWorld, type ShapeInit, type World } from '../src/world.ts'

function fakeCanvas() {
  return { replaceWith: vi.fn() } as unknown as HTMLCanvasElement
}

async function makeWorld(): Promise<World> {
  return createWorld({
    canvas: fakeCanvas(),
    width: 320,
    height: 240,
    background: 0x101010,
    pixelRatio: 1,
    renderer: 'auto',
  })
}

beforeEach(() => { pixiMockState.reset() })
afterEach(() => { vi.restoreAllMocks() })

describe('world: initialization', () => {
  it('inits a Pixi app with the requested dimensions and swaps the canvas', async () => {
    const canvas = fakeCanvas()
    const world = await createWorld({ canvas, width: 800, height: 600, background: 0x222, pixelRatio: 2, renderer: 'auto' })
    expect(pixiMockState.apps).toHaveLength(1)
    const app = pixiMockState.apps[0]!
    expect(app.inited).toBe(true)
    expect((app.initOpts as { width: number }).width).toBe(800)
    expect((app.initOpts as { height: number }).height).toBe(600)
    expect((canvas.replaceWith as ReturnType<typeof vi.fn>)).toHaveBeenCalledOnce()
    expect(world.app).toBe(app)
  })
})

describe('world: spawn + query', () => {
  it('spawns an entity with defaulted transform and adds its display to the stage', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'shape', shape: 'rect', width: 8, height: 8 } })
    expect(e.x).toBe(0)
    expect(e.y).toBe(0)
    expect(e.scale).toBe(1)
    expect(e.alpha).toBe(1)
    expect(e.dead).toBe(false)
    expect(world.app.stage.children).toContain(e.display)
    expect(world.entities).toContain(e)
  })

  it('applies provided init fields to the entity', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'container' }, x: 5, y: -3, vx: 2, vy: 1, z: 4, rotation: 0.5, scale: 2, alpha: 0.5, tint: 0xff0000, tag: 't', group: 'g' })
    expect([e.x, e.y, e.vx, e.vy, e.z, e.rotation, e.scale, e.alpha, e.tint]).toEqual([5, -3, 2, 1, 4, 0.5, 2, 0.5, 0xff0000])
  })

  it('indexes by id, tag, and group', async () => {
    const world = await makeWorld()
    const a = world.spawn({ visual: { kind: 'container' }, id: 'hero', tag: 'player', group: 'allies' })
    world.spawn({ visual: { kind: 'container' }, tag: 'player', group: 'allies' })
    expect(world.byId('hero')).toBe(a)
    expect([...world.byTag('player')]).toHaveLength(2)
    expect([...world.byGroup('allies')]).toHaveLength(2)
  })

  it('returns empty iterables for unknown tag/group and undefined for unknown id', async () => {
    const world = await makeWorld()
    expect([...world.byTag('nobody')]).toHaveLength(0)
    expect([...world.byGroup('nobody')]).toHaveLength(0)
    expect(world.byId('ghost')).toBeUndefined()
  })

  it('assigns unique ids when none provided', async () => {
    const world = await makeWorld()
    const a = world.spawn({ visual: { kind: 'container' } })
    const b = world.spawn({ visual: { kind: 'container' } })
    expect(a.id).not.toBe(b.id)
  })
})

describe('world: destroy + flush', () => {
  it('marks dead on destroy but keeps the entity until flush', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'container' }, id: 'x' })
    world.destroy(e)
    expect(e.dead).toBe(true)
    expect(world.entities).toContain(e) // not yet swept
    world.flush()
    expect(world.entities).not.toContain(e)
  })

  it('deindexes and destroys the display of a dead entity on flush', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'container' }, id: 'x', tag: 'foe', group: 'bad' })
    const display = e.display as { destroyed: boolean }
    e.destroy()
    world.flush()
    expect(world.byId('x')).toBeUndefined()
    expect([...world.byTag('foe')]).toHaveLength(0)
    expect(display.destroyed).toBe(true)
    expect(world.app.stage.children).not.toContain(e.display)
  })

  it('destroy() is idempotent', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'container' } })
    e.destroy()
    e.destroy()
    expect(e.dead).toBe(true)
  })

  it('syncs live entity transform onto its display during flush', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'container' } })
    e.x = 12
    e.y = 34
    e.rotation = 1.5
    e.scale = 3
    e.alpha = 0.25
    e.z = 7
    const d = e.display as { x: number; y: number; rotation: number; scaleX: number; alpha: number; zIndex: number }
    world.flush()
    expect([d.x, d.y, d.rotation, d.scaleX, d.alpha, d.zIndex]).toEqual([12, 34, 1.5, 3, 0.25, 7])
  })

  it('compacts the entity array, keeping survivors in order', async () => {
    const world = await makeWorld()
    const a = world.spawn({ visual: { kind: 'container' }, id: 'a' })
    const b = world.spawn({ visual: { kind: 'container' }, id: 'b' })
    const c = world.spawn({ visual: { kind: 'container' }, id: 'c' })
    b.destroy()
    world.flush()
    expect(world.entities.map((e) => e.id)).toEqual(['a', 'c'])
    void a; void c
  })
})

describe('world: texture loading', () => {
  it('loads a texture once and caches it', async () => {
    const world = await makeWorld()
    const load = vi.fn((src: string) => Promise.resolve({ src }))
    pixiMockState.assetsLoad = load
    const t1 = await world.loadTexture('hero.png')
    const t2 = await world.loadTexture('hero.png')
    expect(t1).toBe(t2)
    expect(load).toHaveBeenCalledOnce()
  })

  it('dedupes concurrent loads of the same src', async () => {
    const world = await makeWorld()
    const load = vi.fn((src: string) => Promise.resolve({ src }))
    pixiMockState.assetsLoad = load
    await Promise.all([world.loadTexture('a.png'), world.loadTexture('a.png')])
    expect(load).toHaveBeenCalledOnce()
  })

  it('assigns the loaded texture to a sprite entity on spawn', async () => {
    const world = await makeWorld()
    pixiMockState.assetsLoad = (src: string) => Promise.resolve({ src, width: 32, height: 32, source: {} })
    const e = world.spawn({ visual: { kind: 'sprite', src: 'ship.png' } })
    await new Promise((r) => setTimeout(r, 0)) // drain loadTexture().then
    expect((e.display as { texture: unknown }).texture).toBeDefined()
  })
})

describe('world: repaint', () => {
  it('repaintComposite rebuilds child graphics', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'composite', parts: [{ shape: 'rect', width: 4, height: 4 }] } })
    const container = e.display as { children: unknown[] }
    expect(container.children).toHaveLength(1)
    world.repaintComposite(e, [
      { shape: 'circle', radius: 2 },
      { shape: 'rect', width: 6, height: 6 },
    ])
    expect(container.children).toHaveLength(2)
  })

  it('repaintShape re-issues paint ops on a single-shape entity', async () => {
    const world = await makeWorld()
    const e = world.spawn({ visual: { kind: 'shape', shape: 'rect', width: 4, height: 4 } })
    const g = e.display as { ops: string[] }
    const before = g.ops.length
    world.repaintShape(e, { kind: 'shape', shape: 'circle', radius: 3 } as ShapeInit)
    expect(g.ops.length).toBeGreaterThan(before)
    expect(g.ops).toContain('clear')
    expect(g.ops).toContain('circle')
  })
})
