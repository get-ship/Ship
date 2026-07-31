// Shared visual part-builders for Magnetar entities.
//
// One source of truth for how the gravity bodies, Earth, the ship, and rescue
// pods are drawn — both the initial spawn and the palette-change repaint call
// these, so a colour tweak never drifts between the two paths.

// Palette colours resolve to PIXI colour numbers, not CSS strings.
export interface Pal {
  primary: number
  warning: number
  destructive: number
  accent: number
  info: number
  foreground: number
  background: number
  success: number
}

/** Visual family for a gravity body. Physics is still driven by mass/radius;
 *  `kind` only decides how the body reads on screen. */
export type PlanetKind = 'moon' | 'planet' | 'giant' | 'repulsor' | 'blackhole'

export interface VisualPart {
  shape: 'circle' | 'polygon'
  radius?: number
  points?: number[]
  fill: number
  alpha?: number
}

/** Fall back to a sensible kind when a level omits one. */
export function inferKind(mass: number, radius: number): PlanetKind {
  if (mass < 0) return 'repulsor'
  if (radius >= 28 || mass >= 3) return 'giant'
  if (radius <= 16 || mass <= 1.2) return 'moon'
  return 'planet'
}

export function planetParts(kind: PlanetKind, radius: number, p: Pal): VisualPart[] {
  switch (kind) {
    case 'moon':
      return [
        { shape: 'circle', radius: radius + 7, fill: p.foreground, alpha: 0.1 },
        { shape: 'circle', radius, fill: p.foreground, alpha: 0.85 },
        { shape: 'circle', radius: radius - 5, fill: p.background, alpha: 0.12 },
      ]
    case 'giant':
      return [
        { shape: 'circle', radius: radius + 16, fill: p.warning, alpha: 0.12 },
        { shape: 'circle', radius: radius + 8, fill: p.warning, alpha: 0.2 },
        { shape: 'circle', radius, fill: p.warning },
        { shape: 'circle', radius: radius - 9, fill: p.background, alpha: 0.22 },
      ]
    case 'repulsor':
      return [
        { shape: 'circle', radius: radius + 12, fill: p.destructive, alpha: 0.16 },
        { shape: 'circle', radius: radius + 5, fill: p.destructive, alpha: 0.22 },
        { shape: 'circle', radius, fill: p.destructive },
        { shape: 'circle', radius: radius - 6, fill: p.background, alpha: 0.18 },
      ]
    case 'blackhole':
      return [
        { shape: 'circle', radius: radius + 20, fill: p.destructive, alpha: 0.1 },
        { shape: 'circle', radius: radius + 10, fill: p.warning, alpha: 0.32 },
        { shape: 'circle', radius: radius + 4, fill: p.destructive, alpha: 0.55 },
        { shape: 'circle', radius, fill: p.background },
        { shape: 'circle', radius: radius - 3, fill: p.foreground, alpha: 0.06 },
      ]
    case 'planet':
    default:
      return [
        { shape: 'circle', radius: radius + 12, fill: p.accent, alpha: 0.12 },
        { shape: 'circle', radius: radius + 6, fill: p.accent, alpha: 0.18 },
        { shape: 'circle', radius, fill: p.accent },
      ]
  }
}

/** Earth — the home you're landing on. Atmosphere halo, ocean, two land masses. */
export function earthParts(radius: number, p: Pal): VisualPart[] {
  return [
    { shape: 'circle', radius: radius + 14, fill: p.info, alpha: 0.12 },
    { shape: 'circle', radius: radius + 7, fill: p.info, alpha: 0.2 },
    { shape: 'circle', radius, fill: p.info },
    { shape: 'polygon', points: [2, -10, 9, -5, 6, 3, -1, -1], fill: p.success, alpha: 0.9 },
    { shape: 'polygon', points: [-9, 1, -2, 4, -4, 11, -10, 7], fill: p.success, alpha: 0.9 },
    // Centre beacon — Earth is a landing target; nail the middle for 3 stars.
    { shape: 'circle', radius: 3, fill: p.foreground, alpha: 0.4 },
  ]
}

/** The ship — a single prominent rocket. Nose points up at rotation 0; the
 *  director rotates it to its heading. The exhaust flame IS the thrust gauge:
 *  `flameLen` (0 when idle) grows the plume out the tail. */
export function shipParts(p: Pal, flameLen = 0): VisualPart[] {
  const parts: VisualPart[] = [
    { shape: 'circle', radius: 14, fill: p.warning, alpha: 0.2 },
  ]
  if (flameLen > 0.5) {
    parts.push(
      { shape: 'polygon', points: [-4.5, 9, 4.5, 9, 0, 9 + flameLen], fill: p.warning, alpha: 0.85 },
      { shape: 'polygon', points: [-2.5, 9, 2.5, 9, 0, 9 + flameLen * 0.62], fill: p.accent },
    )
  }
  parts.push(
    { shape: 'polygon', points: [-5, 3, -11, 12, -5, 10], fill: p.accent },
    { shape: 'polygon', points: [5, 3, 11, 12, 5, 10], fill: p.accent },
    { shape: 'polygon', points: [-5, -6, 5, -6, 5, 10, -5, 10], fill: p.foreground },
    { shape: 'polygon', points: [0, -17, 6, -6, -6, -6], fill: p.warning },
    { shape: 'polygon', points: [0, -5, 3, -1, 0, 3, -3, -1], fill: p.info },
  )
  return parts
}

/** A stranded crew pod — collect it before landing. */
export function rescueParts(radius: number, p: Pal): VisualPart[] {
  return [
    { shape: 'circle', radius: radius + 5, fill: p.success, alpha: 0.22 },
    { shape: 'circle', radius, fill: p.foreground, alpha: 0.92 },
    { shape: 'circle', radius: radius - 4, fill: p.info, alpha: 0.9 },
  ]
}

/** An optional score token — a small star. Pure points, never gates a level. */
export function tokenParts(p: Pal): VisualPart[] {
  return [
    { shape: 'circle', radius: 9, fill: p.accent, alpha: 0.18 },
    { shape: 'polygon', points: [0, -6, 1.7, -1.7, 6, 0, 1.7, 1.7, 0, 6, -1.7, 1.7, -6, 0, -1.7, -1.7], fill: p.accent },
  ]
}
