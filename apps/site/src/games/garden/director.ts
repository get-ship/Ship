// Particle Garden director — orchestrates grid + sim + AI + input + HUD.
//
// Reads the Stage handle from <sui-stage>'s sui-stage:ready event, owns all
// game state, exposes signals the HUD binds to via data-bind. Mirrors the
// shape established by the Galaga director so the surface stays consistent.

import { effect, signal, type Signal } from '@ship/sui-runtime'
import type { Stage } from '@ship/arcade-engine'
import { pickAiMove } from './ai.ts'
import { COLS, ROWS, PULSE_MS, paintCell, readGridPalette, spawnGrid, type Cell, type Grid, type GridPalette } from './grid.ts'
import { decayPulses, stepSim } from './sim.ts'

const STAGE_W = 480
const STAGE_H = 540
const GAME_DURATION_S = 75
const SEED_COOLDOWN_S = 1.55
const AI_INTERVAL_S = 1.45
const TICK_INTERVAL_S = 0.12
const TOTAL_CELLS = COLS * ROWS
const FULL_THRESHOLD = 0.96

interface StageHostApi extends HTMLElement {
  bind?(name: string, sig: Signal<unknown>): void
  set?(name: string, value: unknown): void
  stage?: Stage
}

interface GardenOptions {
  stage: Stage
  stageHost: StageHostApi
}

export interface GardenHandle {
  destroy(): void
  start(): void
  pause(): void
  resume(): void
  scoreSelf: Signal<number>
  scoreOpp: Signal<number>
  timeLeft: Signal<number>
  cooldown: Signal<number>
  cursor: Signal<{ col: number; row: number } | null>
  state: Signal<'idle' | 'playing' | 'won' | 'lost' | 'draw'>
}

const HIGH_KEY = 'garden.high'

function readHigh(): number {
  try {
    const v = Number(globalThis.localStorage?.getItem(HIGH_KEY))
    return Number.isFinite(v) && v >= 0 ? v : 0
  } catch { return 0 }
}
function writeHigh(v: number): void {
  try { globalThis.localStorage?.setItem(HIGH_KEY, String(v)) } catch { /* private mode */ }
}

export function createGardenDirector(opts: GardenOptions): GardenHandle {
  const { stage, stageHost } = opts
  const scoreSelf = signal(0)
  const scoreOpp = signal(0)
  // Wall-clock float for the sim; integer signal for the HUD ceil-display so
  // "75" → "74" happens cleanly without "75.0", "74.97" jitter.
  let timeLeftFloat = GAME_DURATION_S
  const timeLeft = signal(GAME_DURATION_S)
  const cooldown = signal(0)
  const cursor = signal<{ col: number; row: number } | null>({ col: COLS / 2 | 0, row: ROWS / 2 | 0 })
  const gameState = signal<'idle' | 'playing' | 'won' | 'lost' | 'draw'>('idle')
  const high = signal(readHigh())

  stageHost.bind?.('self', scoreSelf as unknown as Signal<unknown>)
  stageHost.bind?.('opp', scoreOpp as unknown as Signal<unknown>)
  stageHost.bind?.('time', timeLeft as unknown as Signal<unknown>)
  stageHost.bind?.('high', high as unknown as Signal<unknown>)
  stageHost.bind?.('cooldown', cooldown as unknown as Signal<unknown>)

  let pal: GridPalette = readGridPalette(stage)
  const grid: Grid = spawnGrid(stage, pal)
  // Paint a fresh-spawn pass to set alphas correctly.
  for (const c of grid.cells) paintCell(stage, c, pal)
  // Render the empty grid onto the idle overlay. Pixi v8's render queue runs
  // inside its own scheduler; calling render() directly without an rAF cycle
  // sometimes draws to the backbuffer but the browser hasn't done a paint
  // pass. Schedule the flush+render on the next frame so the canvas is
  // visible on the idle overlay.
  globalThis.requestAnimationFrame?.(() => {
    stage.world.flush()
    stage.world.app.render()
  })

  // Cursor highlight is a single shape entity that rides over the grid.
  const cursorEntity = stage.world.spawn({
    tag: 'cursor',
    x: 0, y: 0, z: 5,
    visual: { kind: 'shape', shape: 'rect', width: 19, height: 19, fill: pal.empty, stroke: pal.selfHot, strokeWidth: 2 },
    alpha: 0,
  })

  // Contest emitter — bursts when tendrils collide.
  const contestEmitter = stage.particles.add(stage.world, {
    kind: 'radial',
    rate: 0,
    burstCount: 6,
    lifetime: 380,
    spread: 360,
    direction: 0,
    speed: 90,
    speedJitter: 0.6,
    gravity: 0,
    color: pal.selfHot,
    sizeStart: 2,
    sizeEnd: 0,
    max: 240,
  }, 0, 0)

  let tickAccum = 0
  let aiAccum = 0
  let pointerX: number | null = null
  let pointerY: number | null = null

  let rng = mulberry32(0xC0FFEE)

  const unsubs: Array<() => void> = []

  function syncCursorVisual() {
    const c = cursor()
    if (!c) { cursorEntity.alpha = 0; return }
    const target = grid.at(c.col, c.row)
    if (!target) { cursorEntity.alpha = 0; return }
    cursorEntity.x = target.entity.x
    cursorEntity.y = target.entity.y
    cursorEntity.alpha = gameState() === 'playing' ? 0.65 : 0
  }

  function plantSeed(col: number, row: number, owner: 'self' | 'opp'): boolean {
    const cell = grid.at(col, row)
    if (!cell || cell.owner !== 'empty') return false
    cell.owner = owner
    cell.pulseT = 1
    paintCell(stage, cell, pal)
    if (owner === 'self') scoreSelf.set(scoreSelf() + 1)
    else scoreOpp.set(scoreOpp() + 1)
    return true
  }

  function tryPlayerSeed(col: number, row: number) {
    if (gameState() !== 'playing') return
    if (cooldown() > 0) return
    if (plantSeed(col, row, 'self')) {
      cooldown.set(SEED_COOLDOWN_S)
    }
  }

  function endGame() {
    const s = scoreSelf()
    const o = scoreOpp()
    let outcome: 'won' | 'lost' | 'draw' = 'draw'
    if (s > o) outcome = 'won'
    else if (o > s) outcome = 'lost'
    gameState.set(outcome)
    cursorEntity.alpha = 0
    if (s > high()) { high.set(s); writeHigh(s) }
  }

  // Move-cursor actions.
  unsubs.push(stage.input.on('cursor-left', (phase) => { if (phase === 'start') moveCursor(-1, 0) }))
  unsubs.push(stage.input.on('cursor-right', (phase) => { if (phase === 'start') moveCursor(1, 0) }))
  unsubs.push(stage.input.on('cursor-up', (phase) => { if (phase === 'start') moveCursor(0, -1) }))
  unsubs.push(stage.input.on('cursor-down', (phase) => { if (phase === 'start') moveCursor(0, 1) }))
  unsubs.push(stage.input.on('plant', (phase) => {
    if (phase !== 'start') return
    const c = cursor()
    if (c) tryPlayerSeed(c.col, c.row)
  }))

  function moveCursor(dc: number, dr: number) {
    const c = cursor() ?? { col: 0, row: 0 }
    const next = {
      col: Math.max(0, Math.min(COLS - 1, c.col + dc)),
      row: Math.max(0, Math.min(ROWS - 1, c.row + dr)),
    }
    cursor.set(next)
    syncCursorVisual()
  }

  // Pointer input — set by the page script via the host.set('pointer', ...).
  // We poll cursor() reactively to keep the highlight in sync.
  unsubs.push(effect(syncCursorVisual))

  // End-of-game watcher: grid-fill or timer-zero. Use the float so the
  // transition is precise, not gated on the ceil(...) display.
  unsubs.push(effect(() => {
    if (gameState() !== 'playing') return
    const used = scoreSelf() + scoreOpp()
    if (used / TOTAL_CELLS >= FULL_THRESHOLD) endGame()
  }))
  unsubs.push(stage.onTick(() => {
    if (gameState() === 'playing' && timeLeftFloat <= 0) endGame()
  }))

  unsubs.push(stage.palette.onChange(() => {
    pal = readGridPalette(stage)
    for (const cell of grid.cells) paintCell(stage, cell, pal)
    contestEmitter.setColor(pal.selfHot)
    stage.world.repaintShape(cursorEntity, { kind: 'shape', shape: 'rect', width: 19, height: 19, fill: pal.empty, stroke: pal.selfHot, strokeWidth: 2 })
  }))

  unsubs.push(stage.onTick((dt) => {
    if (gameState() !== 'playing') return
    if (cooldown() > 0) cooldown.set(Math.max(0, cooldown() - dt))
    timeLeftFloat = Math.max(0, timeLeftFloat - dt)
    const ceiled = Math.ceil(timeLeftFloat)
    if (ceiled !== timeLeft()) timeLeft.set(ceiled)
    tickAccum += dt
    aiAccum += dt
    while (tickAccum >= TICK_INTERVAL_S) {
      tickAccum -= TICK_INTERVAL_S
      const step = stepSim({ grid, rng })
      if (step.colonizedSelf > 0) scoreSelf.set(scoreSelf() + step.colonizedSelf)
      if (step.colonizedOpp > 0) scoreOpp.set(scoreOpp() + step.colonizedOpp)
      for (const c of step.contests) {
        contestEmitter.x = c.x
        contestEmitter.y = c.y
        contestEmitter.setColor(c.by === 'self' ? pal.selfHot : pal.oppHot)
        contestEmitter.burst()
      }
    }
    while (aiAccum >= AI_INTERVAL_S) {
      aiAccum -= AI_INTERVAL_S
      const move = pickAiMove(grid, rng)
      if (move) plantSeed(move.col, move.row, 'opp')
    }
    // Decay pulses + repaint touched cells once per render frame.
    const touched = decayPulses(grid.cells, dt, PULSE_MS)
    if (touched.length > 0) for (const c of touched) paintCell(stage, c, pal)
  }))

  const api = stageHost as StageHostApi & {
    plantAt?(x: number, y: number): void
    moveCursorTo?(x: number, y: number): void
  }
  api.plantAt = (x: number, y: number) => {
    const cell = grid.cellAtPixel(x, y)
    if (!cell) return
    cursor.set({ col: cell.col, row: cell.row })
    tryPlayerSeed(cell.col, cell.row)
  }
  api.moveCursorTo = (x: number, y: number) => {
    pointerX = x; pointerY = y
    const cell = grid.cellAtPixel(x, y)
    if (cell) cursor.set({ col: cell.col, row: cell.row })
  }

  return {
    destroy() {
      for (const off of unsubs) off()
      unsubs.length = 0
      for (const c of grid.cells) c.entity.destroy()
      cursorEntity.destroy()
      contestEmitter.destroy()
      delete api.plantAt
      delete api.moveCursorTo
    },
    start() {
      // Reset grid state without destroying entities (cheap restart).
      for (const c of grid.cells) {
        c.owner = 'empty'
        c.pulseT = 0
        paintCell(stage, c, pal)
      }
      scoreSelf.set(0)
      scoreOpp.set(0)
      timeLeftFloat = GAME_DURATION_S
      timeLeft.set(GAME_DURATION_S)
      cooldown.set(0)
      tickAccum = 0
      aiAccum = 0
      rng = mulberry32(Date.now() | 0)
      cursor.set({ col: (COLS / 2) | 0, row: (ROWS / 2) | 0 })
      gameState.set('playing')
      syncCursorVisual()
      stage.start()
    },
    pause() { stage.pause() },
    resume() { stage.resume() },
    scoreSelf, scoreOpp, timeLeft, cooldown, cursor,
    state: gameState,
  }
}

function mulberry32(seed: number): () => number {
  let s = seed >>> 0
  return () => {
    s = (s + 0x6D2B79F5) | 0
    let t = s
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}
