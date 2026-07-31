// Magnetar director — gravity-puzzle orchestration.
//
// Owns the game clock, aim + thrust, the live trajectory predictor, the flight
// integration, rescue collection, scoring, the landing celebration, and the
// level list. Pixels live in scene.ts (persistent) and level-view.ts (per
// level); body motion lives in bodies.ts; this file is logic.

import { signal, type Signal } from '@ship/sail-runtime'
import type { Stage } from '@ship/arcade-engine'
import { LEVELS, type LevelSpec } from './levels.ts'
import { earthAt, isDynamic, planetsAt } from './bodies.ts'
import { hitsPlanet, hitsTarget, hitsWall, integrate, MAX_SPEED, MIN_SPEED, predictPath, PROJECTILE_R } from './physics.ts'
import { createScene, GHOST_POOL } from './scene.ts'
import { createLevelView, type LevelView } from './level-view.ts'
import { shipParts, type Pal } from './visuals.ts'
import { loadProgress, recordStars, totalStars, type Progress } from './progress.ts'

const STAGE_W = 480
const STAGE_H = 540
const PREDICT_STEPS = 240
const DT = 1 / 60
const MAX_FLIGHT_STEPS = 720 // ~12s — an orbit that never lands is a miss.
const LAND_TIME = 0.75 // seconds of touchdown animation
const TOKEN_SCORE = 250
const MAX_STARS = LEVELS.length * 3
// Stars come from landing ACCURACY — Earth is a target, and how close to its
// centre you touch down is the score. accuracy ∈ [0,1]: 1 = dead centre, 0 =
// grazing the edge. 3★ = inner third (bullseye), 2★ = inner two-thirds, 1★ =
// landed anywhere on Earth. (Not attempts — replay to nail a cleaner landing.)
function starsForAccuracy(acc: number): number { return acc >= 0.66 ? 3 : acc >= 0.33 ? 2 : 1 }

type State = 'idle' | 'aiming' | 'flying' | 'landing' | 'cleared' | 'failed' | 'won'

interface StageHostApi extends HTMLElement {
  bind?(name: string, sig: Signal<unknown>): void
  stage?: Stage
  setAim?(angleRad: number, speed?: number): void
  fire?(): void
  restart?(): void
  next?(): void
}

export interface MagnetarHandle {
  destroy(): void
  start(): void
  pause(): void
  resume(): void
  score: Signal<number>
  levelIndex: Signal<number>
  levelName: Signal<string>
  shotsLeft: Signal<number>
  lastShotsUsed: Signal<number>
  lastStars: Signal<number>
  lastAccuracy: Signal<number>
  lastPoints: Signal<number>
  thrust: Signal<number>
  state: Signal<State>
  hint: Signal<string>
  /** Best stars earned per level (0 = not cleared). */
  levelStars(): number[]
  /** Levels the player may enter (1-based count). */
  unlockedCount(): number
  levelCount: number
  loadLevel(i: number): void
  setAimAngle(rad: number, speed?: number): void
  fire(): void
  restartLevel(): void
  nextLevel(): void
}

function clampSpeed(v: number): number { return Math.max(MIN_SPEED, Math.min(MAX_SPEED, v)) }
function power(speed: number): number { return (clampSpeed(speed) - MIN_SPEED) / (MAX_SPEED - MIN_SPEED) }
function flameLen(speed: number): number { return 6 + power(speed) * 24 }

export function createMagnetarDirector(opts: { stage: Stage; stageHost: StageHostApi }): MagnetarHandle {
  const { stage, stageHost } = opts
  const score = signal(0)
  const levelIndex = signal(0)
  const levelName = signal(LEVELS[0]!.name)
  const shotsLeft = signal(LEVELS[0]!.shots)
  const gameState = signal<State>('idle')
  const hint = signal(LEVELS[0]!.hint)
  const thrust = signal(Math.round(power(LEVELS[0]!.initialThrust) * 100))
  const crew = signal('')
  let shotsUsedThisLevel = 0
  const lastShotsUsed = signal(0)
  const lastStars = signal(0)
  const lastAccuracy = signal(0)
  const lastPoints = signal(0)
  let progress: Progress = loadProgress(LEVELS.length)
  const starsText = signal(`${totalStars(progress)}/${MAX_STARS}`)

  for (const [name, sig] of [['score', score], ['level', levelName], ['shots', shotsLeft], ['hint', hint], ['stars', starsText], ['thrust', thrust], ['crew', crew]] as const) {
    stageHost.bind?.(name, sig as unknown as Signal<unknown>)
  }

  function pal(): Pal {
    return {
      primary: stage.palette.resolve('primary'), warning: stage.palette.resolve('warning'),
      destructive: stage.palette.resolve('destructive'), accent: stage.palette.resolve('accent'),
      info: stage.palette.resolve('info'), foreground: stage.palette.resolve('foreground'),
      background: stage.palette.resolve('background'), success: stage.palette.resolve('success'),
    }
  }
  let p = pal()
  const scene = createScene(stage, p, STAGE_W, STAGE_H)
  const { ship, ghost, impactEmitter, trailEmitter } = scene

  let view: LevelView | null = null
  let dynamic = false
  let clock = 0
  let aimAngle = LEVELS[0]!.initialAim
  let aimSpeed = LEVELS[0]!.initialThrust
  let flightPos = { x: 0, y: 0 }
  let flightVel = { x: 0, y: 0 }
  let flightSteps = 0
  let landT = 0
  let landFrom = { x: 0, y: 0 }
  let landTo = { x: 0, y: 0 }
  let landAccuracy = 0

  function spec(): LevelSpec { return LEVELS[levelIndex()]! }

  function spawnLevel(s: LevelSpec) {
    p = pal()
    view?.destroy()
    view = createLevelView(stage, s, p)
    dynamic = isDynamic(s.planets, s.target)
    clock = 0
    aimAngle = s.initialAim
    aimSpeed = clampSpeed(s.initialThrust)
    shotsUsedThisLevel = 0
    flightSteps = 0
    levelName.set(s.name)
    hint.set(s.hint)
    shotsLeft.set(s.shots)
    remaining = s.rescues?.length ?? 0
    updateCrew()
    gameState.set('aiming')
    view.sync(0)
    refreshAim()
  }

  // Crew HUD: "collected/total", blank when a level has no rescues.
  let remaining = 0
  function updateCrew() {
    const total = spec().rescues?.length ?? 0
    crew.set(total === 0 ? '' : `${total - remaining}/${total}`)
  }

  function placeShip() {
    const s = spec()
    ship.x = s.launch.x
    ship.y = s.launch.y
    ship.rotation = aimAngle + Math.PI / 2
    ship.alpha = 1
    stage.world.repaintComposite(ship, shipParts(p, flameLen(aimSpeed)))
  }

  function refreshAim() {
    // Aim/thrust only mean something while aiming — ignore stray input during
    // flight or the landing animation so the ship isn't yanked back to the pad.
    if (gameState() !== 'aiming') return
    const s = spec()
    const start = { x: s.launch.x, y: s.launch.y }
    const vx = Math.cos(aimAngle) * aimSpeed
    const vy = Math.sin(aimAngle) * aimSpeed
    const path = predictPath(start, { x: vx, y: vy }, (t) => planetsAt(s.planets, clock + t),
      s.walls, { minX: 0, minY: 0, maxX: STAGE_W, maxY: STAGE_H }, PREDICT_STEPS, DT)
    for (let i = 0; i < GHOST_POOL; i++) {
      const g = ghost[i]!
      const pt = path[i * 2]
      if (!pt) { g.alpha = 0; g.x = -1000; continue }
      g.x = pt.x; g.y = pt.y
      g.alpha = (1 - i / GHOST_POOL) * 0.7
    }
    placeShip()
    thrust.set(Math.round(power(aimSpeed) * 100))
  }

  function clearGhost() { for (const g of ghost) { g.alpha = 0; g.x = -1000 } }

  function fire() {
    if (gameState() !== 'aiming' || !view) return
    const s = spec()
    flightPos = { x: s.launch.x, y: s.launch.y }
    flightVel = { x: Math.cos(aimAngle) * aimSpeed, y: Math.sin(aimAngle) * aimSpeed }
    flightSteps = 0
    view.resetRescues()
    view.resetTokens()
    remaining = s.rescues?.length ?? 0
    updateCrew()
    ship.x = flightPos.x; ship.y = flightPos.y
    ship.rotation = Math.atan2(flightVel.y, flightVel.x) + Math.PI / 2
    stage.world.repaintComposite(ship, shipParts(p, 16))
    clearGhost()
    shotsUsedThisLevel += 1
    gameState.set('flying')
    trailEmitter.x = flightPos.x; trailEmitter.y = flightPos.y
  }

  function endShot(reason: 'crash' | 'out-of-bounds' | 'incomplete') {
    ship.alpha = 0
    trailEmitter.x = -1000; trailEmitter.y = -1000
    impactEmitter.x = ship.x; impactEmitter.y = ship.y
    impactEmitter.setColor(reason === 'crash' ? p.destructive : p.foreground)
    impactEmitter.burst(reason === 'crash' ? 18 : 8)
    const left = shotsLeft() - 1
    shotsLeft.set(left)
    if (left <= 0) { gameState.set('failed') }
    else { gameState.set('aiming'); ship.alpha = 1; refreshAim() }
  }

  function beginLanding() {
    landT = 0
    landFrom = { x: flightPos.x, y: flightPos.y }
    landTo = view!.earthPos()
    stage.world.repaintComposite(ship, shipParts(p, 0))
    trailEmitter.x = -1000; trailEmitter.y = -1000
    impactEmitter.x = landTo.x; impactEmitter.y = landTo.y
    impactEmitter.setColor(p.success)
    impactEmitter.burst(28)
    gameState.set('landing')
  }

  function finalizeWin() {
    ship.alpha = 0
    const stars = starsForAccuracy(landAccuracy)
    // Points scale with accuracy and with the level — later levels pay more.
    const points = Math.round(landAccuracy * (700 + levelIndex() * 130))
    score.set(score() + points)
    lastShotsUsed.set(shotsUsedThisLevel)
    lastStars.set(stars)
    lastAccuracy.set(landAccuracy)
    lastPoints.set(points)
    progress = recordStars(progress, levelIndex(), stars)
    starsText.set(`${totalStars(progress)}/${MAX_STARS}`)
    impactEmitter.x = landTo.x; impactEmitter.y = landTo.y
    impactEmitter.setColor(p.primary)
    impactEmitter.burst(20)
    gameState.set('cleared')
  }

  function advanceFlight() {
    const s = spec()
    const planets = planetsAt(s.planets, clock)
    const step = integrate(flightPos, flightVel, planets, DT)
    flightPos = step.pos; flightVel = step.vel
    flightSteps += 1
    ship.x = flightPos.x; ship.y = flightPos.y
    ship.rotation = Math.atan2(flightVel.y, flightVel.x) + Math.PI / 2
    trailEmitter.x = flightPos.x; trailEmitter.y = flightPos.y
    if (view) {
      const grabbed = view.collect(flightPos.x, flightPos.y)
      if (grabbed.length) {
        remaining = Math.max(0, remaining - grabbed.length)
        updateCrew()
        for (const g of grabbed) { impactEmitter.x = g.x; impactEmitter.y = g.y; impactEmitter.setColor(p.success); impactEmitter.burst(10) }
      }
      for (const tk of view.collectTokens(flightPos.x, flightPos.y)) {
        score.set(score() + TOKEN_SCORE)
        impactEmitter.x = tk.x; impactEmitter.y = tk.y; impactEmitter.setColor(p.accent); impactEmitter.burst(8)
      }
    }
    if (hitsTarget(flightPos, earthAt(s.target, clock))) {
      if (view?.allRescued()) {
        const e = earthAt(s.target, clock)
        const d = Math.hypot(flightPos.x - e.x, flightPos.y - e.y)
        landAccuracy = Math.max(0, 1 - d / (s.target.radius + PROJECTILE_R))
        beginLanding()
      } else endShot('incomplete')
      return
    }
    if (hitsPlanet(flightPos, planets) || hitsWall(flightPos, s.walls)) { endShot('crash'); return }
    const out = flightPos.x < -PROJECTILE_R || flightPos.x > STAGE_W + PROJECTILE_R || flightPos.y < -PROJECTILE_R || flightPos.y > STAGE_H + PROJECTILE_R
    if (out || flightSteps > MAX_FLIGHT_STEPS) endShot('out-of-bounds')
  }

  const unsubs: Array<() => void> = []
  unsubs.push(stage.input.on('aim-left', (ph) => { if (ph === 'start') { aimAngle -= 0.06; refreshAim() } }))
  unsubs.push(stage.input.on('aim-right', (ph) => { if (ph === 'start') { aimAngle += 0.06; refreshAim() } }))
  unsubs.push(stage.input.on('thrust-up', (ph) => { if (ph === 'start') { aimSpeed = clampSpeed(aimSpeed + 4); refreshAim() } }))
  unsubs.push(stage.input.on('thrust-down', (ph) => { if (ph === 'start') { aimSpeed = clampSpeed(aimSpeed - 4); refreshAim() } }))
  unsubs.push(stage.input.on('fire', (ph) => { if (ph === 'start') fire() }))

  unsubs.push(stage.palette.onChange(() => {
    p = pal()
    scene.repaintStatic(p)
    view?.repaint(p)
    if (gameState() === 'aiming') refreshAim()
  }))

  unsubs.push(stage.onTick((dt) => {
    for (const s of scene.stars) {
      s.entity.y += s.speed * dt
      if (s.entity.y > STAGE_H + 2) { s.entity.y = -2; s.entity.x = Math.random() * STAGE_W }
    }
    const st = gameState()
    if (st === 'landing') {
      landT += dt
      const k = Math.min(1, landT / LAND_TIME)
      const e = k * (2 - k)
      ship.x = landFrom.x + (landTo.x - landFrom.x) * e
      ship.y = landFrom.y + (landTo.y - landFrom.y) * e
      ship.scale = 1 - 0.72 * e
      ship.rotation += dt * 6
      if (k >= 1) finalizeWin()
      return
    }
    if (st === 'idle' || st === 'aiming' || st === 'flying') {
      clock += dt
      view?.sync(clock)
      if (st === 'aiming' && dynamic) refreshAim()
      if (st === 'flying') advanceFlight()
    }
  }))

  function loadLevel(i: number) {
    if (i < 0 || i >= LEVELS.length || i >= progress.unlocked) return
    ship.scale = 1
    levelIndex.set(i)
    spawnLevel(LEVELS[i]!)
  }

  const api = stageHost
  api.setAim = (rad, speed) => { aimAngle = rad; if (speed !== undefined) aimSpeed = clampSpeed(speed); refreshAim() }
  api.fire = fire
  api.restart = () => { ship.scale = 1; spawnLevel(spec()) }
  api.next = () => {
    const nxt = levelIndex() + 1
    // Completing a level unlocks the next; if there's none left, back to select.
    if (nxt < LEVELS.length && nxt < progress.unlocked) loadLevel(nxt)
    else { ship.scale = 1; gameState.set('idle') }
  }

  spawnLevel(LEVELS[0]!)
  gameState.set('idle')
  globalThis.requestAnimationFrame?.(() => { stage.world.flush(); stage.world.app.render() })

  return {
    destroy() {
      for (const off of unsubs) off()
      unsubs.length = 0
      view?.destroy()
      scene.destroy()
      delete (api as StageHostApi).setAim
      delete (api as StageHostApi).fire
      delete (api as StageHostApi).restart
      delete (api as StageHostApi).next
    },
    start() { ship.scale = 1; gameState.set('idle'); stage.start() },
    pause() { stage.pause() },
    resume() { stage.resume() },
    score, levelIndex, levelName, shotsLeft, lastShotsUsed, lastStars, lastAccuracy, lastPoints, thrust, state: gameState, hint,
    levelStars: () => progress.stars.slice(),
    unlockedCount: () => progress.unlocked,
    levelCount: LEVELS.length,
    loadLevel,
    setAimAngle(rad, speed) { aimAngle = rad; if (speed !== undefined) aimSpeed = clampSpeed(speed); refreshAim() },
    fire,
    restartLevel() { ship.scale = 1; spawnLevel(spec()) },
    nextLevel() { api.next?.() },
  }
}
