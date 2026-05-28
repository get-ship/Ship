// Level data for Magnetar — gravity-slingshot puzzles, Angry Birds-shaped.
//
// Ten levels with a learning arc: each introduces one idea, the next
// layers on it. Every level placed so the initial aim shows a CURVED
// ghost (the player learns from the preview what gravity is about to
// do) without auto-solving — you still have to fine-tune to clear.
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
  tokens: Array<{ x: number; y: number }>
  walls: Array<{ x: number; y: number; width: number; height: number }>
  shots: number
}

export const LEVELS: LevelSpec[] = [
  // 1 — Pure tutorial: learn the aim mechanic on a clean field.
  {
    name: 'First Light',
    hint: 'Drag from the launch pad to aim. Release to fire.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 200, radius: 24 },
    planets: [],
    tokens: [],
    walls: [],
    shots: 3,
  },
  // 2 — Single attractor: aim wide of the target, gravity brings you home.
  {
    name: 'Gravity Assist',
    hint: 'The planet pulls. Aim through it — let gravity bring you home.',
    launch: { x: 90, y: 430 },
    initialAim: -Math.PI / 5,
    target: { x: 380, y: 220, radius: 22 },
    planets: [{ x: 260, y: 260, mass: 2.0, radius: 24 }],
    tokens: [{ x: 200, y: 380 }],
    walls: [],
    shots: 3,
  },
  // 3 — Direct path blocked: planet sits between launch and target.
  {
    name: 'Sidestep',
    hint: 'The planet blocks the path. Curve around it.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2.6,
    target: { x: 240, y: 180, radius: 22 },
    planets: [{ x: 240, y: 310, mass: 2.0, radius: 26 }],
    tokens: [{ x: 130, y: 280 }, { x: 350, y: 280 }],
    walls: [],
    shots: 3,
  },
  // 4 — Repulsor: aim INTO the push, get kicked the other way.
  {
    name: 'Push',
    hint: 'The bright planet shoves. Aim into it — get kicked across.',
    launch: { x: 90, y: 380 },
    initialAim: -Math.PI / 6,
    target: { x: 400, y: 240, radius: 22 },
    planets: [{ x: 220, y: 360, mass: -2.4, radius: 26 }],
    tokens: [{ x: 320, y: 380 }],
    walls: [],
    shots: 3,
  },
  // 5 — Corridor: thread between two pulls, token in the middle.
  {
    name: 'Corridor',
    hint: 'Two pulls. Find the seam.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 180, radius: 22 },
    planets: [
      { x: 130, y: 300, mass: 1.6, radius: 22 },
      { x: 350, y: 300, mass: 1.6, radius: 22 },
    ],
    tokens: [
      { x: 240, y: 380 },
      { x: 240, y: 240 },
    ],
    walls: [],
    shots: 3,
  },
  // 6 — Cradle: heavy mass below, drop into orbit, planet carries you.
  {
    name: 'Cradle',
    hint: 'Heavy mass below. Drop into orbit — let it carry you across.',
    launch: { x: 90, y: 230 },
    initialAim: -Math.PI / 12,
    target: { x: 400, y: 230, radius: 22 },
    planets: [{ x: 240, y: 360, mass: 3.0, radius: 28 }],
    tokens: [{ x: 160, y: 320 }, { x: 320, y: 320 }],
    walls: [],
    shots: 3,
  },
  // 7 — Eclipse: target HIDDEN behind a planet. Must orbit around.
  {
    name: 'Eclipse',
    hint: 'Target is behind the planet. Go all the way around.',
    launch: { x: 90, y: 400 },
    initialAim: -Math.PI / 4,
    target: { x: 400, y: 280, radius: 22 },
    planets: [{ x: 250, y: 280, mass: 3.2, radius: 32 }],
    tokens: [{ x: 250, y: 200 }, { x: 250, y: 360 }],
    walls: [],
    shots: 3,
  },
  // 8 — Pinball: walls create a chute; planet inside.
  {
    name: 'Pinball',
    hint: 'Walls bite. Find the bounce through them.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 180, radius: 22 },
    planets: [{ x: 240, y: 310, mass: 1.8, radius: 22 }],
    tokens: [{ x: 140, y: 240 }, { x: 340, y: 380 }],
    walls: [
      { x: 0, y: 290, width: 80, height: 14 },
      { x: 400, y: 290, width: 80, height: 14 },
      { x: 60, y: 220, width: 14, height: 60 },
      { x: 406, y: 340, width: 14, height: 60 },
    ],
    shots: 3,
  },
  // 9 — Singularity: one VERY heavy mass. Narrow path that survives the well.
  {
    name: 'Singularity',
    hint: 'Heavy mass. Steep gravity. One narrow path.',
    launch: { x: 90, y: 430 },
    initialAim: -Math.PI / 8,
    target: { x: 400, y: 430, radius: 22 },
    planets: [{ x: 240, y: 280, mass: 4.5, radius: 32 }],
    tokens: [{ x: 240, y: 200 }],
    walls: [],
    shots: 3,
  },
  // 10 — Cosmos: master level. All the toys.
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
    tokens: [
      { x: 240, y: 380 },
      { x: 240, y: 200 },
      { x: 340, y: 280 },
    ],
    walls: [
      { x: 100, y: 220, width: 50, height: 12 },
    ],
    shots: 3,
  },
]
