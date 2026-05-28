// Declarative level data for Magnetar.
//
// Six puzzles, gentle ramp. Each defines the launch pad, target portal, and
// the optional planets/tokens/walls populating the field. The director loads
// one at a time and rebuilds entities; nothing here ticks.

export interface LevelSpec {
  name: string
  hint: string
  launch: { x: number; y: number }
  /** Launch aim's starting angle in radians, 0 = right, -PI/2 = up. */
  initialAim: number
  target: { x: number; y: number; radius: number }
  planets: Array<{ x: number; y: number; mass: number; radius: number }>
  tokens: Array<{ x: number; y: number }>
  walls: Array<{ x: number; y: number; width: number; height: number }>
  shots: number
}

const W = 480
const H = 540

export const LEVELS: LevelSpec[] = [
  {
    name: 'First Light',
    hint: 'Drag from the launch pad to aim. Release to fire.',
    launch: { x: W / 2, y: H - 70 },
    initialAim: -Math.PI / 2,
    target: { x: W / 2, y: 80, radius: 22 },
    planets: [],
    tokens: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Single Pull',
    hint: 'A planet bends the path. Aim wide of it.',
    launch: { x: 80, y: H - 70 },
    initialAim: -Math.PI / 3,
    target: { x: W - 80, y: 100, radius: 22 },
    planets: [{ x: W / 2, y: H / 2, mass: 1.0, radius: 22 }],
    tokens: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Swing-By',
    hint: 'Loop around the heavy mass to reach the portal.',
    launch: { x: 80, y: H - 80 },
    initialAim: -Math.PI / 2.4,
    target: { x: 80, y: 90, radius: 22 },
    planets: [{ x: W / 2 + 80, y: H / 2, mass: 2.2, radius: 28 }],
    tokens: [{ x: W - 100, y: H / 2 - 80 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Token Sweep',
    hint: 'Collect all three tokens en route.',
    launch: { x: W / 2, y: H - 70 },
    initialAim: -Math.PI / 2,
    target: { x: W / 2, y: 80, radius: 22 },
    planets: [
      { x: 130, y: 220, mass: 1.0, radius: 20 },
      { x: W - 130, y: 320, mass: 1.0, radius: 20 },
    ],
    tokens: [
      { x: 230, y: 150 },
      { x: W - 230, y: 240 },
      { x: 240, y: 360 },
    ],
    walls: [],
    shots: 3,
  },
  {
    name: 'Repulsor',
    hint: 'The bright planet pushes — let it kick you in.',
    launch: { x: 80, y: H - 80 },
    initialAim: -Math.PI / 2.5,
    target: { x: W - 80, y: 110, radius: 22 },
    planets: [
      { x: 240, y: 320, mass: -1.4, radius: 24 },
      { x: 360, y: 200, mass: 1.0, radius: 20 },
    ],
    tokens: [{ x: 140, y: 200 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Slalom',
    hint: 'Walls bite. Curve through the gap.',
    launch: { x: W / 2, y: H - 70 },
    initialAim: -Math.PI / 2,
    target: { x: W / 2, y: 80, radius: 22 },
    planets: [
      { x: 110, y: H / 2, mass: 1.4, radius: 22 },
      { x: W - 110, y: H / 2, mass: 1.4, radius: 22 },
    ],
    tokens: [
      { x: W / 2, y: 240 },
      { x: W / 2, y: 360 },
    ],
    walls: [
      { x: 0, y: 280, width: 80, height: 16 },
      { x: W - 80, y: 280, width: 80, height: 16 },
    ],
    shots: 3,
  },
]
