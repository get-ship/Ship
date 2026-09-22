// Grid for Particle Garden — a fixed COLS × ROWS field of cell entities.
//
// Each cell is one shape entity; we index them in a flat array and store
// ownership / pulse state alongside. The grid spawn is one-shot at game
// start; we never spawn/destroy cells during play — only repaint via
// `world.repaintShape` as state changes.

import type { Entity, Stage } from '@ship/arcade-engine'

export type Owner = 'empty' | 'self' | 'opp'

export interface Cell {
  entity: Entity
  col: number
  row: number
  owner: Owner
  /** When non-zero, the cell is freshly colonized; decays to 0 over PULSE_MS. */
  pulseT: number
}

export interface Grid {
  cells: Cell[]
  cols: number
  rows: number
  cellSize: number
  originX: number
  originY: number
  index(col: number, row: number): number | -1
  at(col: number, row: number): Cell | null
  cellAtPixel(px: number, py: number): Cell | null
  countByOwner(owner: Owner): number
}

export const COLS = 22
export const ROWS = 24
export const CELL = 19
export const GUTTER = 2
/* Grid is 22 cols × 19px cells + 21 gutters × 2px = 460px wide.
 * Stage is 480 wide; centred origin = (480 - 460)/2 = 10. The old
 * ORIGIN_X=24 pushed the right column 4px past the stage edge, which
 * read as a wide-left / clipped-right asymmetry. */
export const ORIGIN_X = 10
/* 24 rows × 19 + 23 gutters × 2 = 502px tall. (540 - 502)/2 ≈ 19. */
export const ORIGIN_Y = 19
export const PULSE_MS = 420

export interface GridPalette {
  empty: number
  self: number
  opp: number
  selfHot: number
  oppHot: number
}

export function readGridPalette(stage: Stage): GridPalette {
  return {
    empty: stage.palette.resolve('foreground'),
    self: stage.palette.resolve('primary'),
    opp: stage.palette.resolve('destructive'),
    selfHot: stage.palette.resolve('warning'),
    oppHot: stage.palette.resolve('accent'),
  }
}

export function spawnGrid(stage: Stage, pal: GridPalette): Grid {
  const cells: Cell[] = []
  const pitch = CELL + GUTTER
  for (let row = 0; row < ROWS; row++) {
    for (let col = 0; col < COLS; col++) {
      const x = ORIGIN_X + col * pitch + CELL / 2
      const y = ORIGIN_Y + row * pitch + CELL / 2
      const entity = stage.world.spawn({
        tag: 'cell',
        x, y, z: 1,
        visual: { kind: 'shape', shape: 'rect', width: CELL, height: CELL, fill: pal.empty },
        alpha: 0.18,
      })
      cells.push({ entity, col, row, owner: 'empty', pulseT: 0 })
    }
  }
  function index(col: number, row: number): number | -1 {
    if (col < 0 || col >= COLS || row < 0 || row >= ROWS) return -1
    return row * COLS + col
  }
  function at(col: number, row: number): Cell | null {
    const i = index(col, row)
    return i === -1 ? null : cells[i] ?? null
  }
  function cellAtPixel(px: number, py: number): Cell | null {
    const col = Math.floor((px - ORIGIN_X) / pitch)
    const row = Math.floor((py - ORIGIN_Y) / pitch)
    return at(col, row)
  }
  function countByOwner(owner: Owner): number {
    let n = 0
    for (const c of cells) if (c.owner === owner) n++
    return n
  }
  return { cells, cols: COLS, rows: ROWS, cellSize: CELL, originX: ORIGIN_X, originY: ORIGIN_Y, index, at, cellAtPixel, countByOwner }
}

/** Paint a single cell to match its current owner + pulse state. */
export function paintCell(stage: Stage, cell: Cell, pal: GridPalette): void {
  let fill: number
  let alpha: number
  if (cell.owner === 'empty') {
    fill = pal.empty
    alpha = 0.16
  } else {
    const base = cell.owner === 'self' ? pal.self : pal.opp
    const hot = cell.owner === 'self' ? pal.selfHot : pal.oppHot
    if (cell.pulseT > 0) {
      // Interpolate hot → base as pulse decays.
      const t = 1 - cell.pulseT
      fill = lerpColor(hot, base, t)
      alpha = 0.7 + 0.3 * (1 - t * t)
    } else {
      fill = base
      alpha = 0.88
    }
  }
  stage.world.repaintShape(cell.entity, { kind: 'shape', shape: 'rect', width: CELL, height: CELL, fill })
  cell.entity.alpha = alpha
}

function lerpColor(a: number, b: number, t: number): number {
  const ar = (a >> 16) & 0xff
  const ag = (a >> 8) & 0xff
  const ab = a & 0xff
  const br = (b >> 16) & 0xff
  const bg = (b >> 8) & 0xff
  const bb = b & 0xff
  const r = Math.round(ar + (br - ar) * t)
  const g = Math.round(ag + (bg - ag) * t)
  const bl = Math.round(ab + (bb - ab) * t)
  return (r << 16) | (g << 8) | bl
}
