// Magnetar scene — the persistent visual entities and how they repaint on a
// palette change. The director owns game logic; the scene owns pixels. Level
// bodies (planets, Earth, rescues, walls) are spawned per-level by the
// director; this is everything that lives for the whole session: starfield,
// the ship (which sits on the launch point while aiming), the ghost-trajectory
// pool, and the particle emitters.

import type { Entity, Stage } from '@ship/arcade-engine'
import { type Pal, shipParts } from './visuals.ts'

const STARS_PER_LAYER = 32
export const GHOST_POOL = 90

type Emitter = ReturnType<Stage['particles']['add']>

export interface Scene {
  stars: Array<{ entity: Entity; speed: number; radius: number }>
  ghost: Entity[]
  ship: Entity
  impactEmitter: Emitter
  trailEmitter: Emitter
  repaintStatic(p: Pal): void
  destroy(): void
}

export function createScene(stage: Stage, p: Pal, w: number, h: number): Scene {
  // Ambient starfield.
  const stars: Scene['stars'] = []
  for (let layer = 0; layer < 3; layer++) {
    const speed = 8 + layer * 6
    const r = 0.6 + layer * 0.4
    const a = 0.35 + layer * 0.22
    for (let i = 0; i < STARS_PER_LAYER; i++) {
      const entity = stage.world.spawn({
        tag: 'star', z: -10,
        x: Math.random() * w, y: Math.random() * h,
        visual: { kind: 'shape', shape: 'circle', radius: r, fill: p.foreground },
        alpha: a,
      })
      stars.push({ entity, speed, radius: r })
    }
  }

  // Ghost trajectory pool — tiny circles the director repositions each aim update.
  const ghost: Entity[] = []
  for (let i = 0; i < GHOST_POOL; i++) {
    ghost.push(stage.world.spawn({
      tag: 'ghost', z: 2, x: -1000, y: -1000,
      visual: { kind: 'shape', shape: 'circle', radius: 1.6, fill: p.warning },
      alpha: 0,
    }))
  }

  // The ship — the one prominent object. Sits on the launch point while aiming;
  // rotated to its heading in flight. Its exhaust flame is the thrust gauge.
  const ship = stage.world.spawn({
    tag: 'ship', z: 6, x: -1000, y: -1000,
    visual: { kind: 'composite', parts: shipParts(p, 0) },
    alpha: 0,
  })

  const impactEmitter = stage.particles.add(stage.world, {
    kind: 'radial', rate: 0, burstCount: 22, lifetime: 540, spread: 360,
    direction: 0, speed: 180, speedJitter: 0.6, gravity: 0,
    color: p.warning, sizeStart: 3, sizeEnd: 0, max: 160,
  }, 0, 0)

  const trailEmitter = stage.particles.add(stage.world, {
    kind: 'cone', rate: 90, burstCount: 0, lifetime: 360,
    spread: 30, direction: 0, speed: 60, speedJitter: 0.5, gravity: 0,
    color: p.warning, sizeStart: 1.8, sizeEnd: 0, max: 80,
  }, -1000, -1000)

  return {
    stars, ghost, ship, impactEmitter, trailEmitter,
    repaintStatic(np: Pal) {
      for (const g of ghost) stage.world.repaintShape(g, { kind: 'shape', shape: 'circle', radius: 1.6, fill: np.warning })
      stage.world.repaintComposite(ship, shipParts(np, 0))
      for (const s of stars) stage.world.repaintShape(s.entity, { kind: 'shape', shape: 'circle', radius: s.radius, fill: np.foreground })
      trailEmitter.setColor(np.warning)
      impactEmitter.setColor(np.warning)
    },
    destroy() {
      for (const g of ghost) g.destroy()
      ship.destroy()
      for (const s of stars) s.entity.destroy()
      impactEmitter.destroy()
      trailEmitter.destroy()
    },
  }
}
