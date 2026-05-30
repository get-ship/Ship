// Opponent strategy for Particle Garden.
//
// Single difficulty for v1: a balanced strategy that mixes three behaviours
// with weights tuned for "competitive but beatable in 60 seconds":
//
//   - Expand: pick an empty cell adjacent to our own — grows our blob.
//   - Contest: pick an empty cell adjacent to the player's blob — denies
//     them more growth and grabs disputed border land.
//   - Pioneer: pick an empty cell with no neighbours of either color —
//     starts a new front in unclaimed territory.
//
// The weighting is intentionally simple so the visible game feel is "the
// other side plays smart but never reads your mind." Strategy depth is
// fertile ground for a v2 difficulty selector.

import type { Cell, Grid } from './grid.ts'

const NEIGHBORS: Array<[number, number]> = [[0, -1], [1, 0], [0, 1], [-1, 0]]

export type Move = { col: number; row: number }

interface Scored {
  cell: Cell
  expand: number
  contest: number
  pioneer: number
}

export function pickAiMove(grid: Grid, rng: () => number): Move | null {
  const empties = grid.cells.filter((c) => c.owner === 'empty')
  if (empties.length === 0) return null

  const scored: Scored[] = []
  for (const cell of empties) {
    let selfN = 0
    let oppN = 0
    for (const [dc, dr] of NEIGHBORS) {
      const n = grid.at(cell.col + dc, cell.row + dr)
      if (!n) continue
      if (n.owner === 'self') selfN++
      else if (n.owner === 'opp') oppN++
    }
    const expand = oppN > 0 ? oppN * 1.2 : 0
    const contest = selfN > 0 ? selfN * 1.4 : 0
    const pioneer = selfN === 0 && oppN === 0 ? 0.3 : 0
    if (expand + contest + pioneer === 0) continue
    scored.push({ cell, expand, contest, pioneer })
  }

  // If nothing scored (e.g. opp has no cells yet), fall back to a random pioneer.
  if (scored.length === 0) {
    const pick = empties[Math.floor(rng() * empties.length)]!
    return { col: pick.col, row: pick.row }
  }

  // Sample weighted by total score.
  const totals = scored.map((s) => s.expand + s.contest + s.pioneer)
  const sum = totals.reduce((a, b) => a + b, 0)
  let pick = rng() * sum
  for (let i = 0; i < scored.length; i++) {
    pick -= totals[i]!
    if (pick <= 0) {
      const c = scored[i]!.cell
      return { col: c.col, row: c.row }
    }
  }
  const last = scored[scored.length - 1]!.cell
  return { col: last.col, row: last.row }
}
