// Magnetar director — gravity-slingshot puzzle orchestration.
//
// Loads a level (planets, target, tokens, walls), exposes aim + fire over the
// stage host API, runs the projectile flight through the same integrator the
// trajectory ghost uses, scores token + target hits, and steps through the
// level list.

import { effect, signal, type Signal } from '@ship/sui-runtime'
import type { Entity, Stage } from '@ship/arcade-engine'
import { LEVELS, type LevelSpec } from './levels.ts'
import { hitsPlanet, hitsTarget, hitsToken, hitsWall, integrate, LAUNCH_SPEED, predictPath, PROJECTILE_R, TOKEN_R } from './physics.ts'

const STAGE_W = 480
const STAGE_H = 540
const PREDICT_STEPS = 90
const PREDICT_DT = 1 / 60
const FLIGHT_DT = 1 / 60
const GHOST_POOL = 60
const STARS_PER_LAYER = 32
const PER_TARGET = 1000
const PER_TOKEN = 200

interface StageHostApi extends HTMLElement {
  bind?(name: string, sig: Signal<unknown>): void
  set?(name: string, value: unknown): void
  stage?: Stage
  setAim?(angleRad: number): void
  fire?(): void
  restart?(): void
  next?(): void
}

interface MagnetarOptions {
  stage: Stage
  stageHost: StageHostApi
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
  tokensLeft: Signal<number>
  state: Signal<'idle' | 'aiming' | 'flying' | 'cleared' | 'failed' | 'won'>
  hint: Signal<string>
  setAimAngle(rad: number): void
  fire(): void
  restartLevel(): void
  nextLevel(): void
}

interface LevelEntities {
  planets: Entity[]
  target: Entity
  targetGlow: Entity
  tokens: Array<{ entity: Entity; x: number; y: number; taken: boolean }>
  walls: Entity[]
}

const HIGH_KEY = 'magnetar.high'

function readHigh(): number {
  try {
    const v = Number(globalThis.localStorage?.getItem(HIGH_KEY))
    return Number.isFinite(v) && v >= 0 ? v : 0
  } catch { return 0 }
}
function writeHigh(v: number): void {
  try { globalThis.localStorage?.setItem(HIGH_KEY, String(v)) } catch { /* private */ }
}

export function createMagnetarDirector(opts: MagnetarOptions): MagnetarHandle {
  const { stage, stageHost } = opts
  const score = signal(0)
  const levelIndex = signal(0)
  const levelName = signal(LEVELS[0]!.name)
  const shotsLeft = signal(LEVELS[0]!.shots)
  const tokensLeft = signal(LEVELS[0]!.tokens.length)
  const gameState = signal<'idle' | 'aiming' | 'flying' | 'cleared' | 'failed' | 'won'>('idle')
  const hint = signal(LEVELS[0]!.hint)
  const high = signal(readHigh())

  stageHost.bind?.('score', score as unknown as Signal<unknown>)
  stageHost.bind?.('level', levelName as unknown as Signal<unknown>)
  stageHost.bind?.('shots', shotsLeft as unknown as Signal<unknown>)
  stageHost.bind?.('tokens', tokensLeft as unknown as Signal<unknown>)
  stageHost.bind?.('hint', hint as unknown as Signal<unknown>)
  stageHost.bind?.('high', high as unknown as Signal<unknown>)

  function pal() {
    return {
      primary: stage.palette.resolve('primary'),
      warning: stage.palette.resolve('warning'),
      destructive: stage.palette.resolve('destructive'),
      accent: stage.palette.resolve('accent'),
      info: stage.palette.resolve('info'),
      foreground: stage.palette.resolve('foreground'),
      background: stage.palette.resolve('background'),
    }
  }
  let p = pal()

  // Ambient starfield.
  const stars: Array<{ entity: Entity; speed: number; radius: number }> = []
  for (let layer = 0; layer < 3; layer++) {
    const speed = 8 + layer * 6
    const r = 0.6 + layer * 0.4
    const a = 0.35 + layer * 0.22
    for (let i = 0; i < STARS_PER_LAYER; i++) {
      const entity = stage.world.spawn({
        tag: 'star', z: -10,
        x: Math.random() * STAGE_W,
        y: Math.random() * STAGE_H,
        visual: { kind: 'shape', shape: 'circle', radius: r, fill: p.foreground },
        alpha: a,
      })
      stars.push({ entity, speed, radius: r })
    }
  }

  // Launch pad entity — a triangular chevron at the launch site.
  const launchPad = stage.world.spawn({
    tag: 'launch-pad', z: 4,
    x: LEVELS[0]!.launch.x, y: LEVELS[0]!.launch.y,
    visual: { kind: 'composite', parts: [
      { shape: 'circle', radius: 14, fill: p.primary, alpha: 0.18 },
      { shape: 'polygon', points: [0, -12, 10, 8, -10, 8], fill: p.primary },
    ] },
  })

  // Aim indicator — a short line projecting from the pad.
  const aimIndicator = stage.world.spawn({
    tag: 'aim', z: 4,
    x: LEVELS[0]!.launch.x, y: LEVELS[0]!.launch.y,
    visual: { kind: 'shape', shape: 'rect', width: 4, height: 22, fill: p.warning },
    alpha: 0.85,
  })

  // Ghost trajectory pool — pre-spawned tiny circles we reposition each aim update.
  const ghost: Entity[] = []
  for (let i = 0; i < GHOST_POOL; i++) {
    const entity = stage.world.spawn({
      tag: 'ghost', z: 2,
      x: -1000, y: -1000,
      visual: { kind: 'shape', shape: 'circle', radius: 1.6, fill: p.warning },
      alpha: 0,
    })
    ghost.push(entity)
  }

  // Projectile entity (one, reused). Sits off-stage when not in flight.
  const projectile = stage.world.spawn({
    tag: 'projectile', z: 5,
    x: -1000, y: -1000,
    visual: { kind: 'composite', parts: [
      { shape: 'circle', radius: 5, fill: p.warning, alpha: 0.35 },
      { shape: 'circle', radius: 3, fill: p.warning },
    ] },
    alpha: 0,
  })

  // Particle emitter for impact bursts.
  const impactEmitter = stage.particles.add(stage.world, {
    kind: 'radial',
    rate: 0, burstCount: 22, lifetime: 540, spread: 360,
    direction: 0, speed: 180, speedJitter: 0.6, gravity: 0,
    color: p.warning, sizeStart: 3, sizeEnd: 0, max: 120,
  }, 0, 0)

  // Engine trail emitter — follows the projectile.
  const trailEmitter = stage.particles.add(stage.world, {
    kind: 'cone', rate: 90, burstCount: 0, lifetime: 360,
    spread: 30, direction: 0, speed: 60, speedJitter: 0.5, gravity: 0,
    color: p.warning, sizeStart: 1.8, sizeEnd: 0, max: 80,
  }, -1000, -1000)

  let entities: LevelEntities | null = null
  let aimAngle = LEVELS[0]!.initialAim
  let flightPos = { x: 0, y: 0 }
  let flightVel = { x: 0, y: 0 }
  let levelTokensTaken = 0

  function spawnLevel(spec: LevelSpec) {
    p = pal()
    teardownLevel()
    launchPad.x = spec.launch.x
    launchPad.y = spec.launch.y
    aimAngle = spec.initialAim
    levelTokensTaken = 0
    const planets = spec.planets.map((pl) => {
      const isPush = pl.mass < 0
      return stage.world.spawn({
        tag: 'planet', z: 1,
        x: pl.x, y: pl.y,
        visual: { kind: 'composite', parts: [
          { shape: 'circle', radius: pl.radius + 12, fill: isPush ? p.destructive : p.info, alpha: 0.12 },
          { shape: 'circle', radius: pl.radius + 6, fill: isPush ? p.destructive : p.info, alpha: 0.18 },
          { shape: 'circle', radius: pl.radius, fill: isPush ? p.destructive : p.info },
        ] },
      })
    })
    const targetGlow = stage.world.spawn({
      tag: 'target-glow', z: 1,
      x: spec.target.x, y: spec.target.y,
      visual: { kind: 'shape', shape: 'circle', radius: spec.target.radius + 16, fill: p.primary, stroke: p.primary, strokeWidth: 1 },
      alpha: 0.14,
    })
    const target = stage.world.spawn({
      tag: 'target', z: 2,
      x: spec.target.x, y: spec.target.y,
      visual: { kind: 'composite', parts: [
        { shape: 'circle', radius: spec.target.radius, fill: p.primary, alpha: 0.22 },
        { shape: 'circle', radius: spec.target.radius - 6, fill: p.primary },
      ] },
    })
    const tokens = spec.tokens.map((t) => ({
      x: t.x, y: t.y, taken: false,
      entity: stage.world.spawn({
        tag: 'token', z: 2, x: t.x, y: t.y,
        visual: { kind: 'composite', parts: [
          { shape: 'circle', radius: TOKEN_R + 4, fill: p.warning, alpha: 0.18 },
          { shape: 'circle', radius: TOKEN_R, fill: p.warning },
        ] },
      }),
    }))
    const walls = spec.walls.map((w) => stage.world.spawn({
      tag: 'wall', z: 1,
      x: w.x + w.width / 2, y: w.y + w.height / 2,
      visual: { kind: 'shape', shape: 'rect', width: w.width, height: w.height, fill: p.destructive },
      alpha: 0.55,
    }))
    entities = { planets, target, targetGlow, tokens, walls }
    levelName.set(spec.name)
    hint.set(spec.hint)
    shotsLeft.set(spec.shots)
    tokensLeft.set(spec.tokens.length)
    gameState.set('aiming')
    refreshAim()
  }

  function teardownLevel() {
    if (!entities) return
    for (const e of entities.planets) e.destroy()
    entities.target.destroy()
    entities.targetGlow.destroy()
    for (const t of entities.tokens) t.entity.destroy()
    for (const e of entities.walls) e.destroy()
    entities = null
  }

  function refreshAim() {
    const spec = LEVELS[levelIndex()]!
    const start = { x: spec.launch.x, y: spec.launch.y }
    const vx = Math.cos(aimAngle) * LAUNCH_SPEED
    const vy = Math.sin(aimAngle) * LAUNCH_SPEED
    const planets = spec.planets
    const walls = spec.walls
    const path = predictPath(start, { x: vx, y: vy },
      planets,
      walls,
      { minX: 0, minY: 0, maxX: STAGE_W, maxY: STAGE_H },
      PREDICT_STEPS, PREDICT_DT)
    for (let i = 0; i < GHOST_POOL; i++) {
      const g = ghost[i]!
      const sampleEvery = 2
      const idx = i * sampleEvery
      const pt = path[idx]
      if (!pt) {
        g.alpha = 0
        g.x = -1000
        continue
      }
      g.x = pt.x
      g.y = pt.y
      g.alpha = (1 - i / GHOST_POOL) * 0.75
    }
    aimIndicator.x = spec.launch.x + Math.cos(aimAngle) * 22
    aimIndicator.y = spec.launch.y + Math.sin(aimAngle) * 22
    aimIndicator.rotation = aimAngle + Math.PI / 2
  }

  function clearGhost() {
    for (const g of ghost) { g.alpha = 0; g.x = -1000 }
  }

  function fire() {
    if (gameState() !== 'aiming' || !entities) return
    const spec = LEVELS[levelIndex()]!
    flightPos = { x: spec.launch.x, y: spec.launch.y }
    flightVel = { x: Math.cos(aimAngle) * LAUNCH_SPEED, y: Math.sin(aimAngle) * LAUNCH_SPEED }
    projectile.x = flightPos.x
    projectile.y = flightPos.y
    projectile.alpha = 1
    clearGhost()
    aimIndicator.alpha = 0
    gameState.set('flying')
    trailEmitter.x = flightPos.x
    trailEmitter.y = flightPos.y
  }

  function endShot(reason: 'hit-target' | 'crash' | 'out-of-bounds') {
    projectile.alpha = 0
    trailEmitter.x = -1000
    trailEmitter.y = -1000
    aimIndicator.alpha = 0.85
    if (reason === 'hit-target') {
      const spec = LEVELS[levelIndex()]!
      const bonus = PER_TARGET + levelTokensTaken * PER_TOKEN
      score.set(score() + bonus)
      gameState.set('cleared')
      impactEmitter.x = entities!.target.x
      impactEmitter.y = entities!.target.y
      impactEmitter.setColor(p.primary)
      impactEmitter.burst()
      return
    }
    impactEmitter.x = projectile.x
    impactEmitter.y = projectile.y
    impactEmitter.setColor(reason === 'crash' ? p.destructive : p.foreground)
    impactEmitter.burst(reason === 'crash' ? 18 : 8)
    const left = shotsLeft() - 1
    shotsLeft.set(left)
    // Reset tokens that were collected this attempt so the next shot starts fresh.
    if (entities) {
      for (const t of entities.tokens) { t.taken = false; t.entity.alpha = 1 }
      tokensLeft.set(entities.tokens.length)
    }
    levelTokensTaken = 0
    if (left <= 0) {
      gameState.set('failed')
    } else {
      gameState.set('aiming')
      refreshAim()
    }
  }

  function advanceFlight(dt: number) {
    if (!entities) return
    const spec = LEVELS[levelIndex()]!
    const step = integrate(flightPos, flightVel, spec.planets, dt)
    flightPos = step.pos
    flightVel = step.vel
    projectile.x = flightPos.x
    projectile.y = flightPos.y
    trailEmitter.x = flightPos.x
    trailEmitter.y = flightPos.y
    // Token collection.
    for (const t of entities.tokens) {
      if (!t.taken && hitsToken(flightPos, t)) {
        t.taken = true
        t.entity.alpha = 0
        levelTokensTaken++
        tokensLeft.set(tokensLeft() - 1)
        impactEmitter.x = t.x
        impactEmitter.y = t.y
        impactEmitter.setColor(p.warning)
        impactEmitter.burst(10)
      }
    }
    if (hitsTarget(flightPos, spec.target)) { endShot('hit-target'); return }
    if (hitsPlanet(flightPos, spec.planets) || hitsWall(flightPos, spec.walls)) { endShot('crash'); return }
    if (flightPos.x < -PROJECTILE_R || flightPos.x > STAGE_W + PROJECTILE_R || flightPos.y < -PROJECTILE_R || flightPos.y > STAGE_H + PROJECTILE_R) {
      endShot('out-of-bounds')
    }
  }

  // Cursor-rotation actions for keyboard aim.
  const unsubs: Array<() => void> = []
  unsubs.push(stage.input.on('aim-left', (phase) => { if (phase === 'start') { aimAngle -= 0.08; refreshAim() } }))
  unsubs.push(stage.input.on('aim-right', (phase) => { if (phase === 'start') { aimAngle += 0.08; refreshAim() } }))
  unsubs.push(stage.input.on('fire', (phase) => { if (phase === 'start') fire() }))

  unsubs.push(stage.palette.onChange(() => {
    p = pal()
    stage.world.repaintComposite(launchPad, [
      { shape: 'circle', radius: 14, fill: p.primary, alpha: 0.18 },
      { shape: 'polygon', points: [0, -12, 10, 8, -10, 8], fill: p.primary },
    ])
    stage.world.repaintShape(aimIndicator, { kind: 'shape', shape: 'rect', width: 4, height: 22, fill: p.warning })
    for (const g of ghost) stage.world.repaintShape(g, { kind: 'shape', shape: 'circle', radius: 1.6, fill: p.warning })
    stage.world.repaintComposite(projectile, [
      { shape: 'circle', radius: 5, fill: p.warning, alpha: 0.35 },
      { shape: 'circle', radius: 3, fill: p.warning },
    ])
    for (const s of stars) stage.world.repaintShape(s.entity, { kind: 'shape', shape: 'circle', radius: s.radius, fill: p.foreground })
    trailEmitter.setColor(p.warning)
    impactEmitter.setColor(p.warning)
    if (entities) {
      const spec = LEVELS[levelIndex()]!
      for (let i = 0; i < entities.planets.length; i++) {
        const pl = spec.planets[i]!
        const isPush = pl.mass < 0
        stage.world.repaintComposite(entities.planets[i]!, [
          { shape: 'circle', radius: pl.radius + 12, fill: isPush ? p.destructive : p.info, alpha: 0.12 },
          { shape: 'circle', radius: pl.radius + 6, fill: isPush ? p.destructive : p.info, alpha: 0.18 },
          { shape: 'circle', radius: pl.radius, fill: isPush ? p.destructive : p.info },
        ])
      }
      stage.world.repaintShape(entities.targetGlow, { kind: 'shape', shape: 'circle', radius: spec.target.radius + 16, fill: p.primary, stroke: p.primary, strokeWidth: 1 })
      stage.world.repaintComposite(entities.target, [
        { shape: 'circle', radius: spec.target.radius, fill: p.primary, alpha: 0.22 },
        { shape: 'circle', radius: spec.target.radius - 6, fill: p.primary },
      ])
      for (const t of entities.tokens) {
        stage.world.repaintComposite(t.entity, [
          { shape: 'circle', radius: TOKEN_R + 4, fill: p.warning, alpha: 0.18 },
          { shape: 'circle', radius: TOKEN_R, fill: p.warning },
        ])
      }
      for (const w of entities.walls) stage.world.repaintShape(w, { kind: 'shape', shape: 'rect', width: w.display.width || 1, height: w.display.height || 1, fill: p.destructive })
    }
  }))

  unsubs.push(stage.onTick((dt) => {
    // Starfield ticks always (looks alive on idle).
    for (const s of stars) {
      s.entity.y += s.speed * dt
      if (s.entity.y > STAGE_H + 2) { s.entity.y = -2; s.entity.x = Math.random() * STAGE_W }
    }
    // Subtle target pulse.
    if (entities) {
      const t = (performance.now() / 460) % (Math.PI * 2)
      entities.targetGlow.scale = 1 + Math.sin(t) * 0.06
    }
    if (gameState() === 'flying') advanceFlight(dt)
  }))

  const api = stageHost
  api.setAim = (rad: number) => { aimAngle = rad; refreshAim() }
  api.fire = fire
  api.restart = () => { spawnLevel(LEVELS[levelIndex()]!) }
  api.next = () => {
    const next = levelIndex() + 1
    if (next >= LEVELS.length) {
      const final = score()
      if (final > high()) { high.set(final); writeHigh(final) }
      gameState.set('won')
      return
    }
    levelIndex.set(next)
    spawnLevel(LEVELS[next]!)
  }

  // Initial scene paint so the idle overlay sits over a populated field.
  spawnLevel(LEVELS[0]!)
  gameState.set('idle')
  globalThis.requestAnimationFrame?.(() => {
    stage.world.flush()
    stage.world.app.render()
  })

  // Track loss/win transitions through an effect; the page binds it.
  unsubs.push(effect(() => {
    const s = gameState()
    if (s === 'won') {
      const final = score()
      if (final > high()) { high.set(final); writeHigh(final) }
    }
  }))

  return {
    destroy() {
      for (const off of unsubs) off()
      unsubs.length = 0
      teardownLevel()
      launchPad.destroy()
      aimIndicator.destroy()
      for (const g of ghost) g.destroy()
      projectile.destroy()
      for (const s of stars) s.entity.destroy()
      impactEmitter.destroy()
      trailEmitter.destroy()
      delete (api as StageHostApi).setAim
      delete (api as StageHostApi).fire
      delete (api as StageHostApi).restart
      delete (api as StageHostApi).next
    },
    start() {
      score.set(0)
      levelIndex.set(0)
      gameState.set('aiming')
      spawnLevel(LEVELS[0]!)
      stage.start()
    },
    pause() { stage.pause() },
    resume() { stage.resume() },
    score, levelIndex, levelName, shotsLeft, tokensLeft, state: gameState, hint,
    setAimAngle(rad: number) { aimAngle = rad; refreshAim() },
    fire,
    restartLevel() { spawnLevel(LEVELS[levelIndex()]!) },
    nextLevel() { api.next?.() },
  }
}
