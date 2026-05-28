// 2D gravity physics for Magnetar.
//
// Each planet exerts F = G * m / r² on the projectile, capped at close range
// so the force can't blow up to infinity (which would launch the projectile
// off-screen at light speed when it grazes a well). The same step is used by
// the actual flight tick and by the trajectory predictor — predictions stay
// in sync with reality because both run the identical integrator.

export interface Vec2 {
  x: number
  y: number
}

export interface Planet {
  x: number
  y: number
  /** Positive = attractor, negative = repulsor. */
  mass: number
  /** Visual radius; projectile collides if dist < radius + projectileR. */
  radius: number
}

export interface Wall {
  x: number
  y: number
  width: number
  height: number
}

export interface Token {
  x: number
  y: number
  /** Collected this attempt — stays collected across the projectile's flight. */
  taken: boolean
}

export interface Target {
  x: number
  y: number
  radius: number
}

// Gravity tuning. The visible curve of the projectile path is the whole
// game, so this needs to be aggressive enough that a planet 80-120 px
// from the trajectory bends it by tens of pixels — not the 2 px of
// deflection the prior 5800/280 combo produced.
//
// At a perpendicular distance of 100 px from a mass-2 planet, the
// instantaneous acceleration is G*m/r² = 28000*2/10000 = 5.6 px/s².
// Over the ~2.5s flight time (slower launch speed gives gravity time to
// act), perpendicular deflection ≈ 0.5 * 5.6 * 6.25 = ~17 px — a clear
// arc. The trajectory predictor uses the same constants so the ghost
// line matches the actual flight to the pixel.
export const G = 28000
export const MIN_DIST = 18
export const LAUNCH_SPEED = 220
export const PROJECTILE_R = 4
export const TOKEN_R = 8

export function gravityForceAt(p: Vec2, planets: Planet[]): Vec2 {
  let ax = 0
  let ay = 0
  for (const pl of planets) {
    const dx = pl.x - p.x
    const dy = pl.y - p.y
    const d2 = dx * dx + dy * dy
    const d = Math.sqrt(d2)
    const eff = Math.max(d, MIN_DIST)
    const f = (G * pl.mass) / (eff * eff)
    ax += (dx / d) * f
    ay += (dy / d) * f
  }
  return { x: ax, y: ay }
}

export interface IntegrateResult {
  pos: Vec2
  vel: Vec2
}

export function integrate(pos: Vec2, vel: Vec2, planets: Planet[], dt: number): IntegrateResult {
  const a = gravityForceAt(pos, planets)
  const nvx = vel.x + a.x * dt
  const nvy = vel.y + a.y * dt
  // Symplectic-ish: use the new velocity for position update so curves are stable.
  return {
    pos: { x: pos.x + nvx * dt, y: pos.y + nvy * dt },
    vel: { x: nvx, y: nvy },
  }
}

export interface PredictBounds {
  minX: number
  minY: number
  maxX: number
  maxY: number
}

/**
 * Walk forward `steps` integration steps and return the visited positions.
 * Stops early if the path leaves the bounds or hits a planet/wall — gives a
 * predictor that bottoms out where the actual flight would.
 */
export function predictPath(
  start: Vec2,
  vel: Vec2,
  planets: Planet[],
  walls: Wall[],
  bounds: PredictBounds,
  steps: number,
  dt: number,
): Vec2[] {
  const out: Vec2[] = []
  let p = { ...start }
  let v = { ...vel }
  for (let i = 0; i < steps; i++) {
    const next = integrate(p, v, planets, dt)
    p = next.pos
    v = next.vel
    if (p.x < bounds.minX || p.x > bounds.maxX || p.y < bounds.minY || p.y > bounds.maxY) break
    if (hitsPlanet(p, planets)) break
    if (hitsWall(p, walls)) break
    out.push({ x: p.x, y: p.y })
  }
  return out
}

export function hitsPlanet(p: Vec2, planets: Planet[]): Planet | null {
  for (const pl of planets) {
    const dx = pl.x - p.x
    const dy = pl.y - p.y
    if (dx * dx + dy * dy < (pl.radius + PROJECTILE_R) * (pl.radius + PROJECTILE_R)) return pl
  }
  return null
}

export function hitsWall(p: Vec2, walls: Wall[]): Wall | null {
  for (const w of walls) {
    if (p.x >= w.x && p.x <= w.x + w.width && p.y >= w.y && p.y <= w.y + w.height) return w
  }
  return null
}

export function hitsTarget(p: Vec2, target: Target): boolean {
  const dx = target.x - p.x
  const dy = target.y - p.y
  return dx * dx + dy * dy < (target.radius + PROJECTILE_R) * (target.radius + PROJECTILE_R)
}

export function hitsToken(p: Vec2, token: Token): boolean {
  if (token.taken) return false
  const dx = token.x - p.x
  const dy = token.y - p.y
  return dx * dx + dy * dy < (TOKEN_R + PROJECTILE_R) * (TOKEN_R + PROJECTILE_R)
}
