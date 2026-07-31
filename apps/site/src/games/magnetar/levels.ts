// Level data for Magnetar — gravity puzzles.
//
// You launch a ship and have to land it back home on Earth. You set the launch
// ANGLE and THRUST; gravity does the rest. Every level is a real puzzle: a
// straight shot never reaches home — you have to use the field. Mechanics are
// introduced one at a time and combined at the end:
//   • attractors / repulsors (pull / push)        • walls that block the lane
//   • orbiting + oscillating bodies (timing)       • pulsars (push⇄pull on a clock)
//   • a moving Earth (lead the target)             • rescues (collect before landing)
//
// Bodies move as a function of time-since-launch, and the trajectory predictor
// advances them too, so the ghost path always tells the truth. Play zone is
// roughly x: 50-430, y: 120-470 inside the 480×540 stage.

import { DEFAULT_SPEED } from './physics.ts'
import type { PlanetKind } from './visuals.ts'

/** Circular orbit around (cx,cy); angle = phase + 2π·t/period. */
export interface Orbit { cx: number; cy: number; radius: number; period: number; phase?: number }
/** Linear oscillation along one axis; offset = amp·sin(phase + 2π·t/period). */
export interface Osc { axis: 'x' | 'y'; amp: number; period: number; phase?: number }
/** Mass sign flips on a clock: sign = sin(phase + 2π·t/period) ≥ 0 ? pull : push. */
export interface Pulse { period: number; phase?: number }

export interface LevelBody {
  /** Base position (orbit ignores it; osc swings around it). */
  x: number
  y: number
  /** Positive = attractor, negative = repulsor. Pulsars use |mass|. */
  mass: number
  radius: number
  kind?: PlanetKind
  orbit?: Orbit
  osc?: Osc
  pulse?: Pulse
}

export interface LevelTarget {
  x: number
  y: number
  radius: number
  orbit?: Orbit
  osc?: Osc
}

/** A crew/cargo pod that must be collected before landing counts as a win. */
export interface Rescue { x: number; y: number; radius: number }

/** An optional score pickup. Collecting it adds points; it never gates a win. */
export interface Token { x: number; y: number }

export interface LevelSpec {
  name: string
  hint: string
  launch: { x: number; y: number }
  initialAim: number
  initialThrust: number
  /** Earth — land the ship here (once any rescues are aboard). */
  target: LevelTarget
  planets: LevelBody[]
  rescues?: Rescue[]
  tokens?: Token[]
  walls: Array<{ x: number; y: number; width: number; height: number }>
  shots: number
}

const UP = -Math.PI / 2

export const LEVELS: LevelSpec[] = [
  {
    name: 'Liftoff',
    hint: 'Drag to aim, pull farther for thrust. The planet bends your path — let it carry you to Earth.',
    launch: { x: 110, y: 450 },
    initialAim: -0.8,
    initialThrust: DEFAULT_SPEED,
    target: { x: 380, y: 250, radius: 28 },
    planets: [{ x: 250, y: 350, mass: 2.2, radius: 24, kind: 'planet' }],
    tokens: [{ x: 235, y: 320 }, { x: 320, y: 275 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Sidestep',
    hint: 'The planet sits dead on the line. Curve around either side — gravity does the steering.',
    launch: { x: 240, y: 460 },
    initialAim: UP,
    initialThrust: 105,
    target: { x: 240, y: 150, radius: 30 },
    planets: [{ x: 240, y: 305, mass: 2.4, radius: 26, kind: 'planet' }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Swing-by',
    hint: 'Earth sits behind the giant. Swing close around it and let gravity sling you the rest of the way.',
    launch: { x: 90, y: 410 },
    initialAim: -0.95,
    initialThrust: 92,
    target: { x: 360, y: 150, radius: 28 },
    planets: [{ x: 250, y: 300, mass: 3.6, radius: 32, kind: 'giant' }],
    walls: [],
    shots: 3,
  },
  {
    name: 'The Gate',
    hint: 'The red planet pushes. Skim its edge to get kicked sideways through the gap to Earth.',
    launch: { x: 240, y: 460 },
    initialAim: -1.1,
    initialThrust: 120,
    target: { x: 410, y: 200, radius: 26 },
    planets: [{ x: 250, y: 280, mass: -2.0, radius: 24, kind: 'repulsor' }],
    walls: [
      { x: 150, y: 170, width: 200, height: 12 },
      { x: 150, y: 170, width: 12, height: 90 },
    ],
    shots: 3,
  },
  {
    name: 'Rescue',
    hint: 'Scoop up the stranded pod on the way, then let the planet bend you down into Earth.',
    launch: { x: 90, y: 450 },
    initialAim: -0.2,
    initialThrust: 105,
    target: { x: 400, y: 440, radius: 30 },
    planets: [{ x: 300, y: 330, mass: 2.4, radius: 22, kind: 'planet' }],
    rescues: [{ x: 240, y: 456, radius: 20 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Window',
    hint: 'A moon orbits in front of Earth. Time your launch to thread the gap when it swings clear.',
    launch: { x: 100, y: 450 },
    initialAim: -0.8,
    initialThrust: 100,
    target: { x: 380, y: 180, radius: 26 },
    planets: [{
      x: 300, y: 280, mass: 2.4, radius: 18, kind: 'moon',
      orbit: { cx: 300, cy: 280, radius: 64, period: 4.2 },
    }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Pulsar',
    hint: 'The pulsar flips between pull and push. Launch on the beat that bends you home.',
    launch: { x: 90, y: 430 },
    initialAim: -0.45,
    initialThrust: 115,
    target: { x: 390, y: 300, radius: 24 },
    planets: [{
      x: 240, y: 300, mass: 2.6, radius: 24, kind: 'planet',
      pulse: { period: 2.6 },
    }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Drift',
    hint: 'Earth drifts side to side. Lead it — aim where it will be, not where it is.',
    launch: { x: 240, y: 460 },
    initialAim: -1.35,
    initialThrust: 95,
    target: { x: 240, y: 160, radius: 26, osc: { axis: 'x', amp: 110, period: 5 } },
    planets: [{ x: 240, y: 320, mass: 2.2, radius: 22, kind: 'planet' }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Twin',
    hint: 'Earth rides high behind two planets. Chain the pulls — curve off the first into the second, then home.',
    launch: { x: 80, y: 450 },
    initialAim: -1.0,
    initialThrust: 95,
    target: { x: 400, y: 250, radius: 28 },
    planets: [
      { x: 175, y: 300, mass: 2.6, radius: 24, kind: 'planet' },
      { x: 320, y: 235, mass: 2.4, radius: 22, kind: 'planet' },
    ],
    tokens: [{ x: 175, y: 264 }, { x: 320, y: 199 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Event Horizon',
    hint: 'A black hole. Cross the edge and you are gone — whip around it and let it fling you home.',
    launch: { x: 90, y: 450 },
    initialAim: -0.6,
    initialThrust: 120,
    target: { x: 400, y: 220, radius: 28 },
    planets: [{ x: 240, y: 320, mass: 6.5, radius: 15, kind: 'blackhole' }],
    tokens: [{ x: 300, y: 250 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Homecoming',
    hint: 'Everything at once: grab the pod, ride the pulsar, and land the drifting Earth.',
    launch: { x: 90, y: 450 },
    initialAim: -0.5,
    initialThrust: 110,
    target: { x: 390, y: 240, radius: 30, osc: { axis: 'x', amp: 45, period: 6 } },
    planets: [
      { x: 250, y: 330, mass: 2.4, radius: 22, kind: 'planet', pulse: { period: 3.2 } },
    ],
    rescues: [{ x: 264, y: 374, radius: 22 }],
    walls: [],
    shots: 3,
  },
]
