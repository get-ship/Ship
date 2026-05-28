// Declarative level data for Magnetar.
//
// Six puzzles, gentle ramp. Every game element lives inside the safe play
// zone (x: 50-430, y: 150-440) so nothing renders under the HUD chips or
// the bottom hint pill. The director loads one at a time and rebuilds
// entities; nothing here ticks.

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

// Safe-zone constants — anything outside these bounds risks getting hidden
// behind the corner HUD chips (top, ~130 tall) or the bottom hint pill.
const TOP = 160
const BOT = 440
const LEFT = 60
const RIGHT = 420

export const LEVELS: LevelSpec[] = [
  {
    name: 'First Light',
    hint: 'Drag from the launch pad to aim. Release to fire.',
    launch: { x: 240, y: BOT - 10 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: TOP + 10, radius: 22 },
    planets: [],
    tokens: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Single Pull',
    hint: 'A planet bends the path. Aim wide of it.',
    launch: { x: LEFT + 30, y: BOT - 10 },
    initialAim: -Math.PI / 3,
    target: { x: RIGHT - 30, y: TOP + 20, radius: 22 },
    planets: [{ x: 240, y: 290, mass: 2.0, radius: 22 }],
    tokens: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Swing-By',
    hint: 'Loop around the heavy mass to reach the portal.',
    launch: { x: LEFT + 30, y: BOT - 20 },
    initialAim: -Math.PI / 2.4,
    target: { x: LEFT + 30, y: TOP + 10, radius: 22 },
    planets: [{ x: 320, y: 290, mass: 3.0, radius: 28 }],
    tokens: [{ x: 380, y: 220 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Token Sweep',
    hint: 'Collect tokens en route — bonus per pickup.',
    launch: { x: 240, y: BOT - 10 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: TOP + 10, radius: 22 },
    planets: [
      { x: 130, y: 250, mass: 1.5, radius: 20 },
      { x: 350, y: 350, mass: 1.5, radius: 20 },
    ],
    tokens: [
      { x: 220, y: 220 },
      { x: 260, y: 300 },
      { x: 230, y: 380 },
    ],
    walls: [],
    shots: 3,
  },
  {
    name: 'Repulsor',
    hint: 'The bright planet pushes — let it kick you in.',
    launch: { x: LEFT + 30, y: BOT - 20 },
    initialAim: -Math.PI / 2.5,
    target: { x: RIGHT - 30, y: TOP + 30, radius: 22 },
    planets: [
      { x: 220, y: 330, mass: -2.4, radius: 24 },
      { x: 340, y: 240, mass: 1.6, radius: 20 },
    ],
    tokens: [{ x: 150, y: 250 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Slalom',
    hint: 'Walls bite. Curve through the gap.',
    launch: { x: 240, y: BOT - 10 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: TOP + 10, radius: 22 },
    planets: [
      { x: LEFT + 50, y: 300, mass: 2.0, radius: 22 },
      { x: RIGHT - 50, y: 300, mass: 2.0, radius: 22 },
    ],
    tokens: [
      { x: 240, y: 250 },
      { x: 240, y: 360 },
    ],
    walls: [
      { x: 0, y: 300, width: 70, height: 16 },
      { x: 410, y: 300, width: 70, height: 16 },
    ],
    shots: 3,
  },
]
