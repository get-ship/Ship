// Per-level visual entities for Magnetar: the gravity bodies, Earth, walls, and
// rescue pods. Owns their spawn, per-tick sync (moving/pulsing bodies, drifting
// Earth), collection state for rescues, palette repaint, and teardown — so the
// director only deals with game logic, not entity bookkeeping.

import type { Entity, Stage } from '@ship/arcade-engine'
import type { LevelSpec, Rescue, Token } from './levels.ts'
import { bodyAt, earthAt, massAt } from './bodies.ts'
import { PROJECTILE_R } from './physics.ts'
import { earthParts, inferKind, planetParts, rescueParts, tokenParts, type Pal } from './visuals.ts'

const TOKEN_R = 9

interface PlanetView {
  entity: Entity
  body: LevelSpec['planets'][number]
  /** Last rendered mass sign; -1 repulsing, 1 attracting. Repaint on flip. */
  sign: number
}
interface RescueView { entity: Entity; spec: Rescue; collected: boolean }
interface TokenView { entity: Entity; spec: Token; collected: boolean }

export interface LevelView {
  sync(t: number): void
  /** Collect any pods the ship is touching; returns world points of new grabs. */
  collect(x: number, y: number): Array<{ x: number; y: number }>
  /** Collect any score tokens touched this shot; returns world points. */
  collectTokens(x: number, y: number): Array<{ x: number; y: number }>
  allRescued(): boolean
  resetRescues(): void
  resetTokens(): void
  earthPos(): { x: number; y: number }
  repaint(p: Pal): void
  destroy(): void
}

function planetKind(view: PlanetView, mass: number): 'repulsor' | ReturnType<typeof inferKind> {
  if (mass < 0) return 'repulsor'
  return view.body.kind ?? inferKind(mass, view.body.radius)
}

export function createLevelView(stage: Stage, spec: LevelSpec, p: Pal): LevelView {
  const planets: PlanetView[] = spec.planets.map((body) => {
    const m0 = massAt(body, 0)
    const view: PlanetView = { entity: null as unknown as Entity, body, sign: m0 < 0 ? -1 : 1 }
    view.entity = stage.world.spawn({
      tag: 'planet', z: 1, x: body.x, y: body.y,
      visual: { kind: 'composite', parts: planetParts(planetKind(view, m0), body.radius, p) },
    })
    return view
  })

  const earthGlow = stage.world.spawn({
    tag: 'earth-glow', z: 1, x: spec.target.x, y: spec.target.y,
    visual: { kind: 'shape', shape: 'circle', radius: spec.target.radius + 18, fill: p.info, stroke: p.info, strokeWidth: 1 },
    alpha: 0.12,
  })
  const earth = stage.world.spawn({
    tag: 'earth', z: 2, x: spec.target.x, y: spec.target.y,
    visual: { kind: 'composite', parts: earthParts(spec.target.radius, p) },
  })

  const walls = spec.walls.map((w) => stage.world.spawn({
    tag: 'wall', z: 1, x: w.x + w.width / 2, y: w.y + w.height / 2,
    visual: { kind: 'shape', shape: 'rect', width: w.width, height: w.height, fill: p.destructive },
    alpha: 0.55,
  }))

  const rescues: RescueView[] = (spec.rescues ?? []).map((r) => ({
    spec: r, collected: false,
    entity: stage.world.spawn({
      tag: 'rescue', z: 3, x: r.x, y: r.y,
      visual: { kind: 'composite', parts: rescueParts(r.radius, p) },
    }),
  }))

  const tokens: TokenView[] = (spec.tokens ?? []).map((t) => ({
    spec: t, collected: false,
    entity: stage.world.spawn({
      tag: 'token', z: 3, x: t.x, y: t.y,
      visual: { kind: 'composite', parts: tokenParts(p) },
    }),
  }))

  return {
    sync(t: number) {
      for (const v of planets) {
        const pos = bodyAt(v.body, t)
        v.entity.x = pos.x
        v.entity.y = pos.y
        if (v.body.pulse) {
          const m = massAt(v.body, t)
          const s = m < 0 ? -1 : 1
          if (s !== v.sign) {
            v.sign = s
            stage.world.repaintComposite(v.entity, planetParts(planetKind(v, m), v.body.radius, p))
          }
        }
      }
      const e = earthAt(spec.target, t)
      earth.x = e.x; earth.y = e.y
      earthGlow.x = e.x; earthGlow.y = e.y
      earthGlow.scale = 1 + Math.sin(t * 3.4) * 0.06
      for (const r of rescues) {
        if (!r.collected) r.entity.scale = 1 + Math.sin(t * 4 + r.spec.x) * 0.12
      }
      for (const tk of tokens) {
        if (!tk.collected) tk.entity.rotation = t * 1.6 + tk.spec.x
      }
    },
    collect(x: number, y: number) {
      const grabbed: Array<{ x: number; y: number }> = []
      for (const r of rescues) {
        if (r.collected) continue
        const dx = r.spec.x - x
        const dy = r.spec.y - y
        if (dx * dx + dy * dy < (r.spec.radius + PROJECTILE_R) * (r.spec.radius + PROJECTILE_R)) {
          r.collected = true
          r.entity.alpha = 0.18
          grabbed.push({ x: r.spec.x, y: r.spec.y })
        }
      }
      return grabbed
    },
    collectTokens(x: number, y: number) {
      const grabbed: Array<{ x: number; y: number }> = []
      for (const tk of tokens) {
        if (tk.collected) continue
        const dx = tk.spec.x - x
        const dy = tk.spec.y - y
        if (dx * dx + dy * dy < (TOKEN_R + PROJECTILE_R) * (TOKEN_R + PROJECTILE_R)) {
          tk.collected = true
          tk.entity.alpha = 0
          grabbed.push({ x: tk.spec.x, y: tk.spec.y })
        }
      }
      return grabbed
    },
    allRescued() { return rescues.every((r) => r.collected) },
    resetRescues() { for (const r of rescues) { r.collected = false; r.entity.alpha = 1 } },
    resetTokens() { for (const tk of tokens) { tk.collected = false; tk.entity.alpha = 1 } },
    earthPos() { return { x: earth.x, y: earth.y } },
    repaint(np: Pal) {
      for (const v of planets) stage.world.repaintComposite(v.entity, planetParts(planetKind(v, massAt(v.body, 0)), v.body.radius, np))
      stage.world.repaintShape(earthGlow, { kind: 'shape', shape: 'circle', radius: spec.target.radius + 18, fill: np.info, stroke: np.info, strokeWidth: 1 })
      stage.world.repaintComposite(earth, earthParts(spec.target.radius, np))
      for (const w of walls) stage.world.repaintShape(w, { kind: 'shape', shape: 'rect', width: w.display.width || 1, height: w.display.height || 1, fill: np.destructive })
      for (const r of rescues) stage.world.repaintComposite(r.entity, rescueParts(r.spec.radius, np))
      for (const tk of tokens) stage.world.repaintComposite(tk.entity, tokenParts(np))
    },
    destroy() {
      for (const v of planets) v.entity.destroy()
      earth.destroy(); earthGlow.destroy()
      for (const w of walls) w.destroy()
      for (const r of rescues) r.entity.destroy()
      for (const tk of tokens) tk.entity.destroy()
    },
  }
}
