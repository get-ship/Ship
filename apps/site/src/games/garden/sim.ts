// Growth simulation for Particle Garden.
//
// Each owned cell, on every sim tick, has a chance to colonize an empty
// orthogonal neighbour. The chance scales with how many same-owner neighbours
// the source cell has — a "growing edge" with three same-color allies almost
// always spreads, an isolated seed barely does.
//
// Tendrils that meet a foreign neighbour mark a contested boundary; we expose
// those events so the director can fire particle bursts at the meeting point.

import type { Cell, Grid, Owner } from './grid.ts'

const NEIGHBOR_OFFSETS: Array<[number, number]> = [
  [0, -1], [1, 0], [0, 1], [-1, 0],
]

/** Per-tick spread parameters. The naïve formula
 *  `chance = BASE + allies * BONUS` makes growth scale with the *count*
 *  of active cells (each cell rolls every tick), which saturates the
 *  528-cell grid in seconds. We mitigate with a saturation factor that
 *  drops the global chance toward zero as the grid approaches the
 *  threshold: `chance *= max(0.2, 1 - filled / (TOTAL * SAT_END))`.
 *  This gives a clear arc — fast early growth, gradual mid-game,
 *  slow end-game — and the 75-second timer becomes meaningful again. */
const BASE_SPREAD = 0.16
const ALLY_BONUS = 0.04
const SAT_END = 0.85

export interface ContestEvent {
  /** Pixel position of the contesting cell — where to drop particles. */
  x: number
  y: number
  /** The owner that pushed; the loser is opposite. */
  by: Owner
}

export interface SimStep {
  colonizedSelf: number
  colonizedOpp: number
  contests: ContestEvent[]
}

export interface SimContext {
  grid: Grid
  rng: () => number
}

export function stepSim(ctx: SimContext): SimStep {
  const { grid } = ctx
  const out: SimStep = { colonizedSelf: 0, colonizedOpp: 0, contests: [] }
  // Snapshot the current owner state so colonizations in this tick don't
  // cascade into themselves (each cell decides based on previous tick).
  const prev = new Array<Owner>(grid.cells.length)
  let filled = 0
  for (let i = 0; i < grid.cells.length; i++) {
    const o = grid.cells[i]!.owner
    prev[i] = o
    if (o !== 'empty') filled++
  }
  // Saturation factor: spread chance scales down as the grid fills so
  // the endgame is gradual rather than explosive.
  const saturation = Math.max(0.2, 1 - filled / (grid.cells.length * SAT_END))

  for (let i = 0; i < grid.cells.length; i++) {
    const source = grid.cells[i]!
    if (prev[i] === 'empty') continue
    // Count same-owner orthogonal neighbours among the snapshot.
    let allies = 0
    const empties: Array<{ col: number; row: number; idx: number }> = []
    const foes: Array<{ col: number; row: number }> = []
    for (const [dc, dr] of NEIGHBOR_OFFSETS) {
      const nc = source.col + dc
      const nr = source.row + dr
      const ni = grid.index(nc, nr)
      if (ni === -1) continue
      const p = prev[ni]!
      if (p === prev[i]) allies++
      else if (p === 'empty') empties.push({ col: nc, row: nr, idx: ni })
      else foes.push({ col: nc, row: nr })
    }
    // Spread chance: base + ally bonus per same-color neighbor, scaled
    // by global saturation so growth tapers as the grid fills.
    const chance = (BASE_SPREAD + allies * ALLY_BONUS) * saturation
    if (empties.length > 0 && ctx.rng() < chance) {
      const target = empties[Math.floor(ctx.rng() * empties.length)]!
      const targetCell = grid.cells[target.idx]!
      // Re-check the live owner (an earlier loop iteration in this tick
      // may have already taken it — first-write-wins to avoid overgrowth).
      if (targetCell.owner !== 'empty') continue
      targetCell.owner = prev[i]!
      targetCell.pulseT = 1
      if (prev[i] === 'self') out.colonizedSelf++
      else out.colonizedOpp++
      // Boundary contest event if this colonization touches the opposite color.
      if (foes.length > 0) {
        const cx = grid.originX + target.col * (grid.cellSize + 2) + grid.cellSize / 2
        const cy = grid.originY + target.row * (grid.cellSize + 2) + grid.cellSize / 2
        out.contests.push({ x: cx, y: cy, by: prev[i]! })
      }
    }
  }

  // Decay pulses. Caller drives the dt; we expose a helper instead.
  return out
}

/** Decay every cell's pulseT toward zero over PULSE_MS. */
export function decayPulses(cells: Cell[], dt: number, pulseMs: number): Cell[] {
  const touched: Cell[] = []
  const decay = (dt * 1000) / pulseMs
  for (const c of cells) {
    if (c.pulseT > 0) {
      c.pulseT = Math.max(0, c.pulseT - decay)
      touched.push(c)
    }
  }
  return touched
}
