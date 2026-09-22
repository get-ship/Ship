// Galaga-style minigame director.
//
// Owns the player, bullets, wave progression, collision handlers, and
// scoreboard signals bound to the HUD. Reads the Stage handle exposed by
// <sui-stage>'s `sui-stage:ready` event and never touches Pixi directly.

import { effect, signal, type Signal } from '@ship/sail-runtime'
import type { CollideEvent, Entity, ShapePart, Stage } from '@ship/arcade-engine'
import { damageEnemy, enemyParts, spawnWave, stepEnemies, type FormationState } from './enemies.ts'
import { spawnStarfield, stepStarfield, type Star } from './starfield.ts'
import { WAVES } from './waves.ts'

const STAGE_W = 480
const STAGE_H = 540
const PLAYER_SPEED = 240
const PLAYER_FIRE_COOLDOWN = 0.22
const PLAYER_BULLET_SPEED = 460
const ENEMY_BULLET_SPEED = 220
const PLAYER_RADIUS = 14
const BULLET_LEN = 14
const MAX_PLAYER_BULLETS = 4
const MAX_ENEMY_BULLETS = 20
const STARTING_SHIELDS = 100
const ENEMY_HIT_DAMAGE = 25
const ENEMY_BULLET_DAMAGE = 12

interface Bullet {
  entity: Entity
  vy: number
  alive: boolean
}

export interface GalagaHandle {
  destroy(): void
  start(): void
  pause(): void
  resume(): void
  scoreSignal: Signal<number>
  shieldsSignal: Signal<number>
  waveSignal: Signal<string>
  stateSignal: Signal<'idle' | 'playing' | 'won' | 'over'>
  highSignal: Signal<number>
}

interface StageHostApi extends HTMLElement {
  bind?(name: string, sig: Signal<unknown>): void
  set?(name: string, value: unknown): void
}

interface GalagaOptions {
  stage: Stage
  stageHost: StageHostApi
}

// Player silhouette is a port of the rocket from <sui-loading>'s SVG —
// the Ship loading indicator. Same shape language: bullet nose, fuselage,
// porthole, two swept-back fins. Coordinates re-centred from the original
// 32×40 viewBox (origin at (16, 20)) so the entity's transform pivots
// around the visual center.
function playerVisualParts(stage: Stage): ShapePart[] {
  const body = stage.palette.resolve('primary')
  const port = stage.palette.resolve('background')
  return [
    // Left fin (behind body — draw first)
    { shape: 'polygon', points: [-6, 4, -12, 12, -6, 10], fill: body, alpha: 0.78 },
    // Right fin
    { shape: 'polygon', points: [6, 4, 12, 12, 6, 10], fill: body, alpha: 0.78 },
    // Bullet nose — approximates the rocket SVG's cubic with a tight polygon
    { shape: 'polygon', points: [0, -18, 4, -14, 6, -10, 6, -4, -6, -4, -6, -10, -4, -14], fill: body },
    // Fuselage
    { shape: 'rect', y: 3, width: 12, height: 14, fill: body, alpha: 0.95 },
    // Porthole
    { shape: 'circle', x: 0, y: 0, radius: 2.5, fill: port, alpha: 0.92 },
    // Engine cap (small detail at the base)
    { shape: 'rect', y: 9, width: 6, height: 2, fill: port, alpha: 0.55 },
  ]
}

const HIGH_SCORE_KEY = 'galaga.high'

function readHighScore(): number {
  try {
    const raw = globalThis.localStorage?.getItem(HIGH_SCORE_KEY)
    const n = Number(raw)
    return Number.isFinite(n) && n > 0 ? n : 0
  } catch {
    return 0
  }
}

function writeHighScore(v: number): void {
  try { globalThis.localStorage?.setItem(HIGH_SCORE_KEY, String(v)) } catch { /* private mode */ }
}

export function createGalagaDirector(opts: GalagaOptions): GalagaHandle {
  const { stage, stageHost } = opts
  const score = signal(0)
  const shields = signal(STARTING_SHIELDS)
  const waveLabel = signal('Wave 1')
  const gameState = signal<'idle' | 'playing' | 'won' | 'over'>('idle')
  const high = signal(readHighScore())

  stageHost.bind?.('score', score as unknown as Signal<unknown>)
  stageHost.bind?.('shields', shields as unknown as Signal<unknown>)
  stageHost.bind?.('wave', waveLabel as unknown as Signal<unknown>)
  stageHost.bind?.('high', high as unknown as Signal<unknown>)

  let accentColor = stage.palette.resolve('primary')
  let enemyBulletColor = stage.palette.resolve('destructive')

  // Starfield: a slow parallax layer that runs even while the game is on the
  // start overlay. Lives directly on the world; doesn't participate in collide.
  const stars: Star[] = spawnStarfield(stage, STAGE_W, STAGE_H)

  // Player engine glow trail — a continuous low-rate emitter that follows the
  // player's tail. Pre-spawned at origin; we move it each tick.
  const playerTrail = stage.particles.add(stage.world, {
    kind: 'cone',
    rate: 60,
    burstCount: 0,
    lifetime: 360,
    spread: 28,
    direction: 90,
    speed: 80,
    speedJitter: 0.4,
    gravity: 0,
    color: stage.palette.resolve('warning'),
    sizeStart: 2.2,
    sizeEnd: 0,
    max: 80,
  }, STAGE_W / 2, STAGE_H - 40)

  const player = stage.world.spawn({
    tag: 'player',
    group: 'player',
    x: STAGE_W / 2,
    y: STAGE_H - 56,
    z: 5,
    visual: { kind: 'composite', parts: playerVisualParts(stage) },
  })
  // Player always points up; we don't rotate it.
  player.rotation = 0
  stage.collide.add(player, { kind: 'circle', r: PLAYER_RADIUS, group: 'player', collidesWith: ['enemy', 'enemy-bullet'] })

  const playerBullets: Bullet[] = []
  const enemyBullets: Bullet[] = []

  let formation: FormationState | null = null
  let enemies: Entity[] = []
  let waveIndex = 0
  let fireCooldown = 0
  let leftDown = false
  let rightDown = false
  let nextWaveTimer = 0
  let waveCleared = false
  const unsubs: Array<() => void> = []

  function spawnPlayerBullet() {
    if (gameState() !== 'playing') return
    if (playerBullets.filter((b) => b.alive).length >= MAX_PLAYER_BULLETS) return
    const slot = playerBullets.find((b) => !b.alive)
    const entity = slot?.entity ?? stage.world.spawn({
      tag: 'player-bullet',
      group: 'player-bullet',
      visual: { kind: 'shape', shape: 'rect', width: 3, height: BULLET_LEN, fill: accentColor },
      z: 4,
    })
    stage.collide.add(entity, { kind: 'aabb', w: 4, h: BULLET_LEN, group: 'player-bullet', collidesWith: ['enemy'] })
    entity.x = player.x
    entity.y = player.y - 18
    if (slot) { slot.alive = true; slot.vy = -PLAYER_BULLET_SPEED }
    else playerBullets.push({ entity, vy: -PLAYER_BULLET_SPEED, alive: true })
    stage.input.fire('player-fire', { x: entity.x, y: entity.y })
  }

  function spawnEnemyBullet(x: number, y: number) {
    if (enemyBullets.filter((b) => b.alive).length >= MAX_ENEMY_BULLETS) return
    const slot = enemyBullets.find((b) => !b.alive)
    const entity = slot?.entity ?? stage.world.spawn({
      tag: 'enemy-bullet',
      group: 'enemy-bullet',
      visual: { kind: 'shape', shape: 'rect', width: 3, height: 9, fill: enemyBulletColor },
      z: 3,
    })
    stage.collide.add(entity, { kind: 'aabb', w: 4, h: 9, group: 'enemy-bullet', collidesWith: ['player'] })
    entity.x = x
    entity.y = y
    if (slot) { slot.alive = true; slot.vy = ENEMY_BULLET_SPEED }
    else enemyBullets.push({ entity, vy: ENEMY_BULLET_SPEED, alive: true })
  }

  function killBullet(b: Bullet) {
    b.alive = false
    b.entity.x = -1000
    b.entity.y = -1000
    stage.collide.remove(b.entity)
  }

  function loadWave(idx: number) {
    waveIndex = idx
    const wave = WAVES[idx % WAVES.length]!
    // HUD chip already says "Wave"; the value is just the number.
    waveLabel.set(String(idx + 1))
    const spawned = spawnWave(stage, wave, STAGE_W)
    formation = spawned.formation
    enemies = spawned.enemies
    waveCleared = false
  }

  function endGame(won: boolean) {
    gameState.set(won ? 'won' : 'over')
    const finalScore = score()
    if (finalScore > high()) { high.set(finalScore); writeHighScore(finalScore) }
    stage.input.fire(won ? 'game-won' : 'game-over')
  }

  // Authoritative end-of-game watcher: when shields hit zero we end the game,
  // independent of which collide path drove the damage. Cheap belt-and-braces.
  unsubs.push(effect(() => {
    if (shields() <= 0 && gameState() === 'playing') endGame(false)
  }))

  unsubs.push(stage.input.on('move-left', (phase) => { leftDown = phase === 'start' }))
  unsubs.push(stage.input.on('move-right', (phase) => { rightDown = phase === 'start' }))
  unsubs.push(stage.input.on('fire', (phase) => {
    if (phase === 'start' && fireCooldown <= 0) { spawnPlayerBullet(); fireCooldown = PLAYER_FIRE_COOLDOWN }
  }))

  // Theme-reactive recolor: when the theme changes, re-resolve tokens and
  // repaint every composite-shape entity the director owns. Stars + enemies
  // + player all theme-shift live.
  unsubs.push(stage.palette.onChange(() => {
    accentColor = stage.palette.resolve('primary')
    enemyBulletColor = stage.palette.resolve('destructive')
    stage.world.repaintComposite(player, playerVisualParts(stage))
    const starColor = stage.palette.resolve('foreground')
    for (const s of stars) stage.world.repaintShape(s.entity, { kind: 'shape', shape: 'circle', radius: s.radius, fill: starColor })
    for (const e of enemies) {
      const kind = (e.data.get('enemy') as { spec?: { kind?: 'bee' | 'butterfly' | 'boss' } } | undefined)?.spec?.kind
      if (kind) stage.world.repaintComposite(e, enemyParts(stage, kind))
    }
    for (const b of playerBullets) stage.world.repaintShape(b.entity, { kind: 'shape', shape: 'rect', width: 3, height: BULLET_LEN, fill: accentColor })
    for (const b of enemyBullets) stage.world.repaintShape(b.entity, { kind: 'shape', shape: 'rect', width: 3, height: 9, fill: enemyBulletColor })
    playerTrail.setColor(stage.palette.resolve('warning'))
  }))

  unsubs.push(stage.onCollide((ev: CollideEvent) => {
    const a = ev.a
    const b = ev.b
    handleHit(a, b)
    if (!a.dead && !b.dead) handleHit(b, a)
  }))

  function handleHit(self: Entity, other: Entity) {
    if (self.dead || other.dead) return
    if (self.group === 'player-bullet' && other.group === 'enemy') {
      const slot = playerBullets.find((bb) => bb.entity === self)
      if (slot) killBullet(slot)
      const { killed, points } = damageEnemy(stage, other)
      if (killed) {
        score.set(score() + points)
        stage.input.fire('enemy-die', { x: other.x, y: other.y })
      }
    } else if (self.group === 'enemy-bullet' && other.group === 'player') {
      const slot = enemyBullets.find((bb) => bb.entity === self)
      if (slot) killBullet(slot)
      shields.set(Math.max(0, shields() - ENEMY_BULLET_DAMAGE))
      stage.input.fire('player-hit', { x: other.x, y: other.y })
      if (shields() <= 0) endGame(false)
    } else if (self.group === 'enemy' && other.group === 'player') {
      damageEnemy(stage, self)
      stage.input.fire('enemy-die', { x: self.x, y: self.y })
      shields.set(Math.max(0, shields() - ENEMY_HIT_DAMAGE))
      stage.input.fire('player-hit', { x: other.x, y: other.y })
      if (shields() <= 0) endGame(false)
    }
  }

  unsubs.push(stage.onTick((dt) => {
    // Starfield ticks even on the start overlay so the scene feels alive.
    stepStarfield(stars, dt, STAGE_H, STAGE_W)
    // Engine glow follows the player tail.
    playerTrail.x = player.x
    playerTrail.y = player.y + 18
    if (gameState() !== 'playing') return
    fireCooldown = Math.max(0, fireCooldown - dt)
    const dir = (rightDown ? 1 : 0) - (leftDown ? 1 : 0)
    player.x = clamp(player.x + dir * PLAYER_SPEED * dt, 24, STAGE_W - 24)
    for (const b of playerBullets) {
      if (!b.alive) continue
      b.entity.y += b.vy * dt
      if (b.entity.y < -BULLET_LEN) killBullet(b)
    }
    for (const b of enemyBullets) {
      if (!b.alive) continue
      b.entity.y += b.vy * dt
      if (b.entity.y > STAGE_H + 12) killBullet(b)
    }
    if (formation) stepEnemies(dt, formation, enemies, player, STAGE_H, spawnEnemyBullet)
    if (!waveCleared && enemies.every((e) => e.dead)) {
      waveCleared = true
      const wave = WAVES[waveIndex % WAVES.length]!
      score.set(score() + wave.clearBonus)
      nextWaveTimer = 1.5
    }
    if (waveCleared) {
      nextWaveTimer -= dt
      if (nextWaveTimer <= 0) {
        if (waveIndex + 1 >= WAVES.length) endGame(true)
        else loadWave(waveIndex + 1)
      }
    }
  }))

  return {
    destroy() {
      for (const off of unsubs) off()
      unsubs.length = 0
      // Tear down every entity the director spawned so a restart doesn't
      // leave the previous player drifting in the scene collecting hits.
      stage.collide.remove(player)
      player.destroy()
      for (const b of playerBullets) { stage.collide.remove(b.entity); b.entity.destroy() }
      for (const b of enemyBullets) { stage.collide.remove(b.entity); b.entity.destroy() }
      for (const e of enemies) { stage.collide.remove(e); e.destroy() }
      for (const s of stars) s.entity.destroy()
      playerTrail.destroy()
    },
    start() {
      gameState.set('playing')
      score.set(0)
      shields.set(STARTING_SHIELDS)
      loadWave(0)
      stage.start()
    },
    pause() { stage.pause() },
    resume() { stage.resume() },
    scoreSignal: score,
    shieldsSignal: shields,
    waveSignal: waveLabel,
    stateSignal: gameState,
    highSignal: high,
  }
}

function clamp(v: number, lo: number, hi: number): number { return Math.max(lo, Math.min(hi, v)) }
