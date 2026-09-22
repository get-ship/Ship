// Wave specifications for the Galaga-style minigame.
//
// Each wave declares its formation: a grid of enemy slots positioned at
// formation positions (relative to the formation center). Enemies enter
// from above and settle into these positions, then begin oscillating;
// periodically one peels off and dives at the player.

export interface EnemySpec {
  /** Logical identifier for art/behavior variant. */
  kind: 'bee' | 'butterfly' | 'boss'
  /** Slot index in the formation grid (left-to-right, top-to-bottom). */
  slot: number
  /** Points awarded when destroyed. */
  points: number
  /** Hit points. */
  hp: number
  /** How aggressive the dive timer is (lower = dives more often). */
  divePeriodMs: number
}

export interface WaveSpec {
  /** Display label (e.g. "Wave 1"). */
  label: string
  /** Columns in the formation grid. */
  cols: number
  /** Rows in the formation grid. */
  rows: number
  /** Horizontal spacing between formation slots, in stage pixels. */
  spacingX: number
  /** Vertical spacing between formation slots. */
  spacingY: number
  /** Pixels/sec the whole formation oscillates side-to-side. */
  formationSpeed: number
  /** Half-amplitude of formation oscillation. */
  formationAmp: number
  /** Enemy slots in this wave. */
  enemies: EnemySpec[]
  /** Bonus when this wave is cleared. */
  clearBonus: number
}

function formationGrid(cols: number, rows: number, kinds: EnemySpec['kind'][]): EnemySpec[] {
  const out: EnemySpec[] = []
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const slot = r * cols + c
      const kind = kinds[r % kinds.length]!
      const points = kind === 'boss' ? 400 : kind === 'butterfly' ? 200 : 100
      const hp = kind === 'boss' ? 2 : 1
      const divePeriodMs = kind === 'boss' ? 4500 : 6000
      out.push({ kind, slot, points, hp, divePeriodMs })
    }
  }
  return out
}

export const WAVES: WaveSpec[] = [
  {
    label: 'Wave 1',
    cols: 6, rows: 2,
    spacingX: 44, spacingY: 40,
    formationSpeed: 28, formationAmp: 36,
    enemies: formationGrid(6, 2, ['bee']),
    clearBonus: 500,
  },
  {
    label: 'Wave 2',
    cols: 7, rows: 3,
    spacingX: 42, spacingY: 38,
    formationSpeed: 36, formationAmp: 48,
    enemies: formationGrid(7, 3, ['butterfly', 'bee', 'bee']),
    clearBonus: 800,
  },
  {
    label: 'Wave 3',
    cols: 8, rows: 4,
    spacingX: 40, spacingY: 38,
    formationSpeed: 48, formationAmp: 60,
    enemies: formationGrid(8, 4, ['boss', 'butterfly', 'bee', 'bee']),
    clearBonus: 1200,
  },
]
