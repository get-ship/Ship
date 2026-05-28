// Collision: uniform-grid broad-phase + AABB/circle narrow-phase.
//
// One CollideSystem per world. Entities register a spec describing kind
// (aabb|circle), group, and collidesWith. Each step the system rebuckets
// every collider into a sparse grid, then for each cell checks pairs whose
// groups intersect. Emits a CollideEvent per overlapping pair.

import type { Entity } from './world.ts'

export interface ColliderSpec {
  kind: 'aabb' | 'circle'
  w?: number
  h?: number
  r?: number
  ox?: number
  oy?: number
  group: string
  collidesWith: string[]
  trigger?: boolean
}

export interface CollideEvent {
  a: Entity
  b: Entity
  /** Minimum translation vector — push `a` by (nx, ny) to separate. */
  nx: number
  ny: number
  depth: number
}

export interface CollideSystem {
  add(e: Entity, spec: ColliderSpec): void
  remove(e: Entity): void
  step(): CollideEvent[]
}

interface Registration {
  e: Entity
  spec: ColliderSpec
}

const CELL = 64

function cellsFor(reg: Registration, out: number[]): void {
  out.length = 0
  const cx = reg.e.x + (reg.spec.ox ?? 0)
  const cy = reg.e.y + (reg.spec.oy ?? 0)
  const half = reg.spec.kind === 'circle'
    ? (reg.spec.r ?? 1)
    : Math.max(reg.spec.w ?? 1, reg.spec.h ?? 1) / 2
  const minX = Math.floor((cx - half) / CELL)
  const maxX = Math.floor((cx + half) / CELL)
  const minY = Math.floor((cy - half) / CELL)
  const maxY = Math.floor((cy + half) / CELL)
  for (let gy = minY; gy <= maxY; gy++) {
    for (let gx = minX; gx <= maxX; gx++) {
      out.push(gx * 73856093 ^ gy * 19349663)
    }
  }
}

function intersect(a: Registration, b: Registration): { nx: number; ny: number; depth: number } | null {
  const ax = a.e.x + (a.spec.ox ?? 0)
  const ay = a.e.y + (a.spec.oy ?? 0)
  const bx = b.e.x + (b.spec.ox ?? 0)
  const by = b.e.y + (b.spec.oy ?? 0)
  if (a.spec.kind === 'circle' && b.spec.kind === 'circle') {
    const dx = bx - ax
    const dy = by - ay
    const r = (a.spec.r ?? 1) + (b.spec.r ?? 1)
    const d2 = dx * dx + dy * dy
    if (d2 >= r * r) return null
    const d = Math.sqrt(d2) || 1
    return { nx: -dx / d, ny: -dy / d, depth: r - d }
  }
  if (a.spec.kind === 'aabb' && b.spec.kind === 'aabb') {
    const ahx = (a.spec.w ?? 1) / 2
    const ahy = (a.spec.h ?? 1) / 2
    const bhx = (b.spec.w ?? 1) / 2
    const bhy = (b.spec.h ?? 1) / 2
    const dx = bx - ax
    const dy = by - ay
    const px = ahx + bhx - Math.abs(dx)
    const py = ahy + bhy - Math.abs(dy)
    if (px <= 0 || py <= 0) return null
    if (px < py) return { nx: dx < 0 ? 1 : -1, ny: 0, depth: px }
    return { nx: 0, ny: dy < 0 ? 1 : -1, depth: py }
  }
  // Circle vs AABB (or AABB vs circle, normalized).
  const circ = a.spec.kind === 'circle' ? a : b
  const box = a.spec.kind === 'aabb' ? a : b
  const cx = circ.e.x + (circ.spec.ox ?? 0)
  const cy = circ.e.y + (circ.spec.oy ?? 0)
  const bxx = box.e.x + (box.spec.ox ?? 0)
  const byy = box.e.y + (box.spec.oy ?? 0)
  const hx = (box.spec.w ?? 1) / 2
  const hy = (box.spec.h ?? 1) / 2
  const closestX = Math.max(bxx - hx, Math.min(cx, bxx + hx))
  const closestY = Math.max(byy - hy, Math.min(cy, byy + hy))
  const dx = cx - closestX
  const dy = cy - closestY
  const r = circ.spec.r ?? 1
  const d2 = dx * dx + dy * dy
  if (d2 >= r * r) return null
  const d = Math.sqrt(d2) || 1
  const sign = circ === a ? -1 : 1
  return { nx: sign * dx / d, ny: sign * dy / d, depth: r - d }
}

export function createCollide(): CollideSystem {
  const regs = new Map<Entity, Registration>()
  const cells = new Map<number, Registration[]>()
  const cellBuf: number[] = []
  const seen = new Set<string>()

  function add(e: Entity, spec: ColliderSpec) {
    regs.set(e, { e, spec })
  }
  function remove(e: Entity) {
    regs.delete(e)
  }

  function step(): CollideEvent[] {
    cells.clear()
    seen.clear()
    for (const reg of regs.values()) {
      if (reg.e.dead) continue
      cellsFor(reg, cellBuf)
      for (const h of cellBuf) {
        let list = cells.get(h)
        if (!list) { list = []; cells.set(h, list) }
        list.push(reg)
      }
    }
    const events: CollideEvent[] = []
    for (const bucket of cells.values()) {
      for (let i = 0; i < bucket.length; i++) {
        for (let j = i + 1; j < bucket.length; j++) {
          const a = bucket[i]!
          const b = bucket[j]!
          if (a.e.dead || b.e.dead) continue
          const aWantsB = a.spec.collidesWith.includes(b.spec.group)
          const bWantsA = b.spec.collidesWith.includes(a.spec.group)
          if (!aWantsB && !bWantsA) continue
          const key = a.e.id < b.e.id ? `${a.e.id}:${b.e.id}` : `${b.e.id}:${a.e.id}`
          if (seen.has(key)) continue
          seen.add(key)
          const hit = intersect(a, b)
          if (!hit) continue
          events.push({ a: a.e, b: b.e, nx: hit.nx, ny: hit.ny, depth: hit.depth })
        }
      }
    }
    return events
  }

  return { add, remove, step }
}
