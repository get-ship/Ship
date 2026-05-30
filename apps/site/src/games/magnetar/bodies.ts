// Time-evolving body positions for Magnetar.
//
// Bodies (planets, Earth) can orbit, oscillate, or pulse. Everything is a pure
// function of time-since-launch `t` (seconds), so the live simulation and the
// trajectory predictor stay in lockstep — both ask "where is this body at t?"
// and get the same answer.

import type { LevelBody, LevelTarget } from './levels.ts'
import type { Planet, Target } from './physics.ts'

const TAU = Math.PI * 2

/** Resolved position of one body at time t. */
export function bodyAt(b: LevelBody, t: number): { x: number; y: number } {
  if (b.orbit) {
    const a = (b.orbit.phase ?? 0) + (TAU * t) / b.orbit.period
    return { x: b.orbit.cx + Math.cos(a) * b.orbit.radius, y: b.orbit.cy + Math.sin(a) * b.orbit.radius }
  }
  if (b.osc) {
    const d = b.osc.amp * Math.sin((b.osc.phase ?? 0) + (TAU * t) / b.osc.period)
    return { x: b.x + (b.osc.axis === 'x' ? d : 0), y: b.y + (b.osc.axis === 'y' ? d : 0) }
  }
  return { x: b.x, y: b.y }
}

/** Effective mass at time t — pulsars flip sign on their clock. */
export function massAt(b: LevelBody, t: number): number {
  if (!b.pulse) return b.mass
  const s = Math.sin((b.pulse.phase ?? 0) + (TAU * t) / b.pulse.period)
  return Math.abs(b.mass) * (s >= 0 ? 1 : -1)
}

/** Full planet snapshot for the integrator/predictor at time t. */
export function planetsAt(bodies: LevelBody[], t: number): Planet[] {
  return bodies.map((b) => {
    const pos = bodyAt(b, t)
    return { x: pos.x, y: pos.y, mass: massAt(b, t), radius: b.radius }
  })
}

/** Earth's snapshot at time t. */
export function earthAt(tgt: LevelTarget, t: number): Target {
  if (tgt.orbit) {
    const a = (tgt.orbit.phase ?? 0) + (TAU * t) / tgt.orbit.period
    return { x: tgt.orbit.cx + Math.cos(a) * tgt.orbit.radius, y: tgt.orbit.cy + Math.sin(a) * tgt.orbit.radius, radius: tgt.radius }
  }
  if (tgt.osc) {
    const d = tgt.osc.amp * Math.sin((tgt.osc.phase ?? 0) + (TAU * t) / tgt.osc.period)
    return { x: tgt.x + (tgt.osc.axis === 'x' ? d : 0), y: tgt.y + (tgt.osc.axis === 'y' ? d : 0), radius: tgt.radius }
  }
  return { x: tgt.x, y: tgt.y, radius: tgt.radius }
}

/** Does this level have anything that moves or pulses? Drives the live ghost. */
export function isDynamic(planets: LevelBody[], tgt: LevelTarget): boolean {
  return Boolean(tgt.orbit || tgt.osc) || planets.some((b) => b.orbit || b.osc || b.pulse)
}
