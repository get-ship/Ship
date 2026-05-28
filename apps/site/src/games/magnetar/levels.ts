// Level data for Magnetar — gravity-slingshot puzzles.
//
// Every level placed so gravity is part of the solution: planets sit
// where they BEND the path toward the target, not where they block it.
// The hint nudges the player toward using the curve.
//
// All elements live inside the safe play zone (x: 60-420, y: 160-440)
// so nothing renders under the HUD chips or the bottom hint pill.

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
  {
    name: 'First Light',
    hint: 'Drag from the launch pad to aim. Release to fire.',
    launch: { x: 240, y: 450 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 180, radius: 22 },
    planets: [],
    tokens: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Gravity Assist',
    hint: "The planet pulls. Aim wide of the target — let gravity bring you home.",
    launch: { x: 90, y: 440 },
    initialAim: -Math.PI / 6,
    target: { x: 380, y: 220, radius: 22 },
    planets: [{ x: 260, y: 250, mass: 2.0, radius: 22 }],
    tokens: [],
    walls: [],
    shots: 3,
  },
  {
    name: 'Around the Bend',
    hint: 'Target is above you. Use the planet to slingshot up.',
    launch: { x: 90, y: 430 },
    initialAim: -Math.PI / 12,
    target: { x: 380, y: 200, radius: 22 },
    planets: [{ x: 240, y: 330, mass: 2.6, radius: 28 }],
    tokens: [{ x: 90, y: 220 }, { x: 380, y: 330 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Slingshot',
    hint: 'Two pulls. Find the corridor between them.',
    launch: { x: 240, y: 440 },
    initialAim: -Math.PI / 2,
    target: { x: 240, y: 180, radius: 22 },
    planets: [
      { x: 130, y: 300, mass: 1.6, radius: 22 },
      { x: 350, y: 300, mass: 1.6, radius: 22 },
    ],
    tokens: [{ x: 240, y: 300 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Repulsor',
    hint: 'The bright planet shoves you away. Use it as a kick.',
    launch: { x: 90, y: 380 },
    initialAim: -Math.PI / 4,
    target: { x: 400, y: 220, radius: 22 },
    planets: [
      { x: 200, y: 320, mass: -2.4, radius: 24 },
      { x: 330, y: 240, mass: 1.4, radius: 20 },
    ],
    tokens: [{ x: 280, y: 380 }],
    walls: [],
    shots: 3,
  },
  {
    name: 'Cradle',
    hint: 'A heavy mass below the target. Drop into orbit; let it carry you.',
    launch: { x: 90, y: 230 },
    initialAim: -Math.PI / 8,
    target: { x: 400, y: 230, radius: 22 },
    planets: [
      { x: 240, y: 360, mass: 3.0, radius: 28 },
    ],
    tokens: [
      { x: 240, y: 200 },
      { x: 340, y: 320 },
    ],
    walls: [],
    shots: 3,
  },
]
