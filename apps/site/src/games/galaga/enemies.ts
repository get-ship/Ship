// Enemy spawning, formation motion, dive AI for the Galaga-style minigame.
//
// Director calls spawnWave() to seed entities and stepEnemies() each tick to
// advance their behavior. Enemies live in three states:
//   - entering: scripted curve from off-screen to formation slot
//   - formation: oscillates with the formation centre + slot offset
//   - diving: peels toward the player along a noisy curve
//
// Returns helpers for collision-handler use (resolve a bullet hit, etc.)

import type { Entity, ShapePart, Stage } from '@ship/arcade-engine'
import type { EnemySpec, WaveSpec } from './waves.ts'

export interface EnemyState {
  spec: EnemySpec
  phase: 'entering' | 'formation' | 'diving'
  enterT: number
  enterDuration: number
  enterFromX: number
  enterFromY: number
  formationX: number
  formationY: number
  diveT: number
  diveDuration: number
  diveStartX: number
  diveStartY: number
  diveTargetX: number
  diveTargetY: number
  divePhase: number
  diveTimer: number
}

export interface FormationState {
  wave: WaveSpec
  centreX: number
  centreY: number
  phase: number
}

const ENEMY_RADIUS = 12

export function enemyParts(stage: Stage, kind: EnemySpec['kind']): ShapePart[] {
  if (kind === 'bee') {
    const body = stage.palette.resolve('warning')
    const wing = stage.palette.resolve('warning')
    return [
      // wings (drawn behind body so the body silhouette stays crisp)
      { shape: 'polygon', points: [-10, 0, -3, -4, -3, 4], fill: wing, alpha: 0.55 },
      { shape: 'polygon', points: [10, 0, 3, -4, 3, 4], fill: wing, alpha: 0.55 },
      // body (rounded by stacking a circle + a small lower bulb)
      { shape: 'circle', radius: 6, fill: body },
      { shape: 'circle', y: 4, radius: 4, fill: body, alpha: 0.85 },
      // eye pair (white dots — use the background token so they invert)
      { shape: 'circle', x: -2, y: -1.5, radius: 1.2, fill: stage.palette.resolve('background') },
      { shape: 'circle', x: 2, y: -1.5, radius: 1.2, fill: stage.palette.resolve('background') },
    ]
  }
  if (kind === 'butterfly') {
    const body = stage.palette.resolve('destructive')
    const wing = stage.palette.resolve('destructive')
    const accent = stage.palette.resolve('warning')
    return [
      // upper wings
      { shape: 'polygon', points: [-3, -3, -13, -10, -10, 2, -3, 1], fill: wing },
      { shape: 'polygon', points: [3, -3, 13, -10, 10, 2, 3, 1], fill: wing },
      // lower wings
      { shape: 'polygon', points: [-3, 2, -9, 11, -3, 8], fill: wing, alpha: 0.78 },
      { shape: 'polygon', points: [3, 2, 9, 11, 3, 8], fill: wing, alpha: 0.78 },
      // body
      { shape: 'polygon', points: [0, -10, 3, 9, -3, 9], fill: body },
      // accent dots on wings
      { shape: 'circle', x: -8, y: -3, radius: 1.4, fill: accent },
      { shape: 'circle', x: 8, y: -3, radius: 1.4, fill: accent },
    ]
  }
  // boss
  const body = stage.palette.resolve('accent')
  const eye = stage.palette.resolve('foreground')
  const back = stage.palette.resolve('background')
  return [
    // hex chassis
    { shape: 'polygon', points: [-13, -5, -7, -12, 7, -12, 13, -5, 13, 5, 7, 12, -7, 12, -13, 5], fill: body },
    // side arms
    { shape: 'rect', x: -14, y: 0, width: 5, height: 14, fill: body },
    { shape: 'rect', x: 14, y: 0, width: 5, height: 14, fill: body },
    // central eye
    { shape: 'circle', radius: 4.5, fill: eye },
    { shape: 'circle', radius: 2, fill: back },
  ]
}

function slotPosition(wave: WaveSpec, slot: number, centreX: number, centreY: number) {
  const col = slot % wave.cols
  const row = Math.floor(slot / wave.cols)
  const x = centreX + (col - (wave.cols - 1) / 2) * wave.spacingX
  const y = centreY + row * wave.spacingY
  return { x, y }
}

export function spawnWave(
  stage: Stage,
  wave: WaveSpec,
  stageWidth: number,
): { formation: FormationState; enemies: Entity[] } {
  const centreX = stageWidth / 2
  const centreY = 90
  const formation: FormationState = { wave, centreX, centreY, phase: 0 }
  const enemies: Entity[] = []
  wave.enemies.forEach((spec, i) => {
    const slot = slotPosition(wave, spec.slot, centreX, centreY)
    const enterFromX = i % 2 === 0 ? -40 : stageWidth + 40
    const enterFromY = -30
    const entity = stage.world.spawn({
      tag: `enemy:${spec.kind}`,
      group: 'enemy',
      x: enterFromX,
      y: enterFromY,
      z: 2,
      visual: { kind: 'composite', parts: enemyParts(stage, spec.kind) },
    })
    const state: EnemyState = {
      spec,
      phase: 'entering',
      enterT: 0,
      enterDuration: 0.8 + (i % wave.cols) * 0.05,
      enterFromX, enterFromY,
      formationX: slot.x, formationY: slot.y,
      diveT: 0, diveDuration: 0,
      diveStartX: 0, diveStartY: 0,
      diveTargetX: 0, diveTargetY: 0,
      divePhase: 0,
      diveTimer: spec.divePeriodMs / 1000 + Math.random() * 2,
    }
    entity.data.set('enemy', state)
    entity.data.set('hp', spec.hp)
    stage.collide.add(entity, { kind: 'circle', r: ENEMY_RADIUS, group: 'enemy', collidesWith: ['player', 'player-bullet'] })
    enemies.push(entity)
  })
  return { formation, enemies }
}

export function stepEnemies(
  dt: number,
  formation: FormationState,
  enemies: Entity[],
  player: Entity,
  stageHeight: number,
  onEnemyFire: (x: number, y: number) => void,
): void {
  formation.phase += dt * (formation.wave.formationSpeed / formation.wave.formationAmp)
  const sway = Math.sin(formation.phase) * formation.wave.formationAmp
  for (const e of enemies) {
    if (e.dead) continue
    const s = e.data.get('enemy') as EnemyState | undefined
    if (!s) continue
    if (s.phase === 'entering') {
      s.enterT += dt
      const t = Math.min(1, s.enterT / s.enterDuration)
      const targetX = s.formationX + sway
      const targetY = s.formationY
      e.x = lerp(s.enterFromX, targetX, easeOutCubic(t))
      e.y = lerp(s.enterFromY, targetY, easeOutCubic(t))
      // Banking lean toward the formation slot — clamped so we never tumble.
      e.rotation = clamp(Math.atan2(targetX - e.x, -(targetY - e.y + 60)), -0.5, 0.5)
      if (t >= 1) { s.phase = 'formation'; e.rotation = 0 }
      continue
    }
    if (s.phase === 'formation') {
      e.x = s.formationX + sway
      e.y = s.formationY
      s.diveTimer -= dt
      if (s.diveTimer <= 0 && Math.random() < 0.6) {
        s.phase = 'diving'
        s.diveT = 0
        s.diveDuration = 2.1 + Math.random() * 0.6
        s.diveStartX = e.x
        s.diveStartY = e.y
        s.diveTargetX = player.x + (Math.random() * 80 - 40)
        s.diveTargetY = stageHeight + 30
        s.divePhase = Math.random() * Math.PI * 2
      } else if (s.diveTimer <= 0) {
        s.diveTimer = s.spec.divePeriodMs / 1000 + Math.random() * 1.5
      }
      continue
    }
    if (s.phase === 'diving') {
      s.diveT += dt
      const t = s.diveT / s.diveDuration
      if (t >= 1) {
        // Missed the player: return to formation (re-enter from above).
        s.phase = 'entering'
        s.enterT = 0
        s.enterDuration = 0.6
        s.enterFromX = s.formationX + (Math.random() * 80 - 40)
        s.enterFromY = -30
        s.diveTimer = s.spec.divePeriodMs / 1000 + 1
        continue
      }
      const baseX = lerp(s.diveStartX, s.diveTargetX, t)
      const baseY = lerp(s.diveStartY, s.diveTargetY, t)
      const wobble = Math.sin(s.divePhase + t * 6) * 30 * (1 - t)
      e.x = baseX + wobble
      e.y = baseY
      // Bank toward dive direction, but cap at ±35° so the silhouette is still
      // recognizable instead of cartwheeling at the player.
      const dx = s.diveTargetX - s.diveStartX
      e.rotation = clamp(Math.atan2(dx, 240), -0.6, 0.6)
      if (Math.random() < dt * 0.6) onEnemyFire(e.x, e.y + 12)
    }
  }
}

export function damageEnemy(stage: Stage, e: Entity): { killed: boolean; points: number } {
  const hp = (e.data.get('hp') as number | undefined) ?? 1
  const next = hp - 1
  if (next <= 0) {
    const s = e.data.get('enemy') as EnemyState | undefined
    stage.collide.remove(e)
    e.destroy()
    return { killed: true, points: s?.spec.points ?? 100 }
  }
  e.data.set('hp', next)
  return { killed: false, points: 0 }
}

function lerp(a: number, b: number, t: number): number { return a + (b - a) * t }
function easeOutCubic(t: number): number { return 1 - (1 - t) ** 3 }
function clamp(v: number, lo: number, hi: number): number { return Math.max(lo, Math.min(hi, v)) }
