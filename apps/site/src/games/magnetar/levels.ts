// Level data for Magnetar — gravity-slingshot puzzles.
//
// Ten levels with a learning arc: each introduces one idea, the next
// layers on it. Tokens removed — scoring is now per-shot accuracy
// (clear-on-first-shot pays max, third-shot still passes but cheap),
// so replay value comes from "can I do this on one shot?" rather than
// "did I miss an impossible pickup?"
//
// All elements inside the safe play zone (x: 60-420, y: 160-440).

export interface LevelSpec {
  name: string
  hint: string
  launch: { x: number; y: number }
  /** Launch aim's starting angle in radians, 0 = right, -PI/2 = up. */
  initialAim: number
  target: { x: number; y: number; radius: number }
  planets: Array<{ x: number; y: number; mass: number; radius: number }>
  walls: Array<{ x: number; y: number; width: number; height: number }>
  shots: number
}

export const LEVELS: LevelSpec[] = [
  {
    name: 'First Light',
    hint: 'Drag from the launch pad to aim. Release to fire.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 200, radius: 24 },
    planets: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Gravity Assist',
    hint: 'The planet pulls. Aim through it — let gravity bring you home.',
    launch: { x: 90, y: 430 },
    initialAim: -Math.PI / 5,
    target: { x: 380, y: 220, radius: 22 },
    planets: [{ x: 260, y: 260, mass: 2.0, radius: 24 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Sidestep',
    hint: 'The planet blocks the path. Curve around it.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2.6,
    target: { x: 240, y: 180, radius: 22 },
    planets: [{ x: 240, y: 310, mass: 2.0, radius: 26 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Push',
    hint: 'The bright planet shoves. Skim past — get kicked across the gap.',
    launch: { x: 90, y: 280 },
    initialAim: 0,
    target: { x: 400, y: 280, radius: 24 },
    planets: [{ x: 240, y: 380, mass: -1.6, radius: 22 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Corridor',
    hint: 'Two pulls. Find the seam between them.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 180, radius: 22 },
    planets: [
      { x: 130, y: 300, mass: 1.6, radius: 22 },
      { x: 350, y: 300, mass: 1.6, radius: 22 },
    ],
    walls: [],
    shots: 3,
  },
  {
    name: 'Cradle',
    hint: 'Heavy mass below. Drop into orbit — let it carry you across.',
    launch: { x: 90, y: 230 },
    initialAim: -Math.PI / 12,
    target: { x: 400, y: 230, radius: 22 },
    planets: [{ x: 240, y: 360, mass: 3.0, radius: 28 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Eclipse',
    hint: 'Target is behind the planet. Go all the way around.',
    launch: { x: 90, y: 400 },
    initialAim: -Math.PI / 4,
    target: { x: 400, y: 280, radius: 22 },
    planets: [{ x: 250, y: 280, mass: 3.2, radius: 32 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Pinball',
    hint: 'Walls bite. Find the bounce through them.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 180, radius: 22 },
    planets: [{ x: 240, y: 310, mass: 1.8, radius: 22 }],
    walls: [
      { x: 0, y: 290, width: 80, height: 14 },
      { x: 400, y: 290, width: 80, height: 14 },
      { x: 60, y: 220, width: 14, height: 60 },
      { x: 406, y: 340, width: 14, height: 60 },
    ],
    shots: 3,
  },
  {
    name: 'Singularity',
    hint: 'Heavy mass. Steep gravity. One narrow path.',
    launch: { x: 90, y: 430 },
    initialAim: -Math.PI / 8,
    target: { x: 400, y: 430, radius: 22 },
    planets: [{ x: 240, y: 280, mass: 4.5, radius: 32 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Cosmos',
    hint: 'All the rules. Find the perfect curve.',
    launch: { x: 80, y: 430 },
    initialAim: -Math.PI / 6,
    target: { x: 400, y: 220, radius: 22 },
    planets: [
      { x: 170, y: 320, mass: 2.0, radius: 22 },
      { x: 290, y: 240, mass: -1.6, radius: 22 },
      { x: 360, y: 360, mass: 1.8, radius: 22 },
    ],
    walls: [
      { x: 100, y: 220, width: 50, height: 12 },
    ],
    shots: 3,
  },
]
