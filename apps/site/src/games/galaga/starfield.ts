// Three-layer parallax starfield. Stars are tiny shape entities at z=-10
// behind everything; each layer scrolls downward at a different speed and
// wraps when it leaves the bottom. No collision, no interaction — pure
// ambient.

import type { Entity, Stage } from '@ship/arcade-engine'

export interface Star {
  entity: Entity
  speed: number
  radius: number
}

const LAYERS = [
  { count: 36, speed: 14, radius: 0.6, alpha: 0.45 },
  { count: 22, speed: 28, radius: 0.9, alpha: 0.65 },
  { count: 12, speed: 56, radius: 1.4, alpha: 0.95 },
]

export function spawnStarfield(stage: Stage, width: number, height: number): Star[] {
  const color = stage.palette.resolve('foreground')
  const stars: Star[] = []
  for (const layer of LAYERS) {
    for (let i = 0; i < layer.count; i++) {
      const entity = stage.world.spawn({
        tag: 'star',
        z: -10,
        x: Math.random() * width,
        y: Math.random() * height,
        visual: { kind: 'shape', shape: 'circle', radius: layer.radius, fill: color },
        alpha: layer.alpha,
      })
      stars.push({ entity, speed: layer.speed, radius: layer.radius })
    }
  }
  return stars
}

export function stepStarfield(stars: Star[], dt: number, height: number, width = 480): void {
  for (const s of stars) {
    s.entity.y += s.speed * dt
    if (s.entity.y > height + 2) {
      s.entity.y = -2
      s.entity.x = Math.random() * width
    }
  }
}
