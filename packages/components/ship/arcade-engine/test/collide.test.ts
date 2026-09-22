import { describe, expect, it } from 'vitest'
import { createCollide, type ColliderSpec } from '../src/collide.ts'
import type { Entity } from '../src/world.ts'

// A minimal Entity stand-in: collide only reads id/x/y/dead.
let idSeq = 0
function ent(x: number, y: number, id?: string): Entity {
  return {
    id: id ?? `c${idSeq++}`,
    x,
    y,
    z: 0,
    vx: 0,
    vy: 0,
    rotation: 0,
    scale: 1,
    alpha: 1,
    tint: null,
    data: new Map(),
    display: null,
    dead: false,
    destroy() { this.dead = true },
  } as unknown as Entity
}

const box = (group: string, collidesWith: string[], w = 10, h = 10): ColliderSpec =>
  ({ kind: 'aabb', w, h, group, collidesWith })
const circ = (group: string, collidesWith: string[], r = 5): ColliderSpec =>
  ({ kind: 'circle', r, group, collidesWith })

describe('collide: AABB vs AABB', () => {
  it('reports a hit with the shallow-axis MTV when boxes overlap', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(6, 0) // half-widths 5+5=10, dx=6 → px=4 (x is shallow axis)
    sys.add(a, box('player', ['enemy']))
    sys.add(b, box('enemy', ['player']))
    const events = sys.step()
    expect(events).toHaveLength(1)
    const e = events[0]!
    expect(e.depth).toBeCloseTo(4)
    // a is left of b (dx>0) so a must be pushed -x to separate.
    expect(e.nx).toBe(-1)
    expect(e.ny).toBe(0)
  })

  it('reports a miss when boxes are fully separated', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(20, 0) // gap of 10 between edges
    sys.add(a, box('player', ['enemy']))
    sys.add(b, box('enemy', ['player']))
    expect(sys.step()).toHaveLength(0)
  })

  it('treats edge-touching (zero penetration) as a miss', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(10, 0) // edges exactly meet: px = 5+5-10 = 0 → not < 0, the code uses <=0
    sys.add(a, box('player', ['enemy']))
    sys.add(b, box('enemy', ['player']))
    expect(sys.step()).toHaveLength(0)
  })

  it('picks the vertical axis as MTV when it is the shallower one', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(1, 8) // px = 9, py = 2 → py shallower, b above-ish (dy>0) → push a -y
    sys.add(a, box('p', ['e']))
    sys.add(b, box('e', ['p']))
    const e = sys.step()[0]!
    expect(e.ny).toBe(-1)
    expect(e.nx).toBe(0)
    expect(e.depth).toBeCloseTo(2)
  })
})

describe('collide: circle vs circle', () => {
  it('hits when centers are closer than the radii sum', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(8, 0) // r 5+5=10, d=8 → depth 2
    sys.add(a, circ('p', ['e']))
    sys.add(b, circ('e', ['p']))
    const e = sys.step()[0]!
    expect(e.depth).toBeCloseTo(2)
    expect(e.nx).toBeCloseTo(-1) // push a away from b along -x
    expect(e.ny).toBeCloseTo(0)
  })

  it('misses when exactly at the radii sum (touching)', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(10, 0) // d == r → d2 >= r*r → null
    sys.add(a, circ('p', ['e']))
    sys.add(b, circ('e', ['p']))
    expect(sys.step()).toHaveLength(0)
  })

  it('uses fallback normal when centers coincide (avoids divide-by-zero)', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(0, 0)
    sys.add(a, circ('p', ['e']))
    sys.add(b, circ('e', ['p']))
    const e = sys.step()[0]!
    // d2 = 0 → the `|| 1` guard sets d = 1 (not 0), so depth = r - 1 = 9 and
    // the normal collapses to (0/1, 0/1) = (0, 0). No NaN/Infinity leaks out.
    expect(e.depth).toBeCloseTo(9)
    expect(e.nx).toBe(-0) // -dx/d = -0/1
    expect(e.ny).toBe(-0)
    expect(Number.isFinite(e.nx)).toBe(true)
    expect(Number.isFinite(e.ny)).toBe(true)
  })
})

describe('collide: circle vs AABB', () => {
  it('hits when the circle overlaps the nearest box edge', () => {
    const sys = createCollide()
    const c = ent(9, 0) // circle r=5 to the right of a 10x10 box at origin
    const b = ent(0, 0) // box edge at x=5, closest point (5,0), dx=4 < r → hit, depth 1
    sys.add(c, circ('bullet', ['wall']))
    sys.add(b, box('wall', ['bullet']))
    const e = sys.step()[0]!
    expect(e.depth).toBeCloseTo(1)
    expect(Number.isFinite(e.nx)).toBe(true)
  })

  it('misses when the circle clears the box', () => {
    const sys = createCollide()
    const c = ent(20, 0)
    const b = ent(0, 0)
    sys.add(c, circ('bullet', ['wall']))
    sys.add(b, box('wall', ['bullet']))
    expect(sys.step()).toHaveLength(0)
  })
})

describe('collide: filtering + lifecycle', () => {
  it('emits nothing when neither group lists the other', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(2, 0)
    sys.add(a, box('a', ['x']))
    sys.add(b, box('b', ['y']))
    expect(sys.step()).toHaveLength(0)
  })

  it('emits when only one side opts in (one-way trigger)', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(2, 0)
    sys.add(a, box('a', ['b']))
    sys.add(b, box('b', [])) // b ignores a, but a wants b
    expect(sys.step()).toHaveLength(1)
  })

  it('skips dead entities', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(2, 0)
    sys.add(a, box('p', ['e']))
    sys.add(b, box('e', ['p']))
    expect(sys.step()).toHaveLength(1)
    b.destroy()
    expect(sys.step()).toHaveLength(0)
  })

  it('stops reporting after remove()', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(2, 0)
    sys.add(a, box('p', ['e']))
    sys.add(b, box('e', ['p']))
    expect(sys.step()).toHaveLength(1)
    sys.remove(b)
    expect(sys.step()).toHaveLength(0)
  })

  it('reports each overlapping pair exactly once (no duplicate from multi-cell spanning)', () => {
    const sys = createCollide()
    // Large overlapping AABBs that span multiple 64px broad-phase cells.
    const a = ent(0, 0, 'aaa')
    const b = ent(40, 0, 'bbb')
    sys.add(a, box('p', ['e'], 200, 200))
    sys.add(b, box('e', ['p'], 200, 200))
    expect(sys.step()).toHaveLength(1)
  })

  it('applies offset (ox/oy) when positioning the collider', () => {
    const sys = createCollide()
    const a = ent(0, 0)
    const b = ent(0, 0)
    // Without offset these fully overlap. Offset b far away → miss.
    sys.add(a, box('p', ['e']))
    sys.add(b, { kind: 'aabb', w: 10, h: 10, ox: 100, oy: 0, group: 'e', collidesWith: ['p'] })
    expect(sys.step()).toHaveLength(0)
  })
})
