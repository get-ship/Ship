// Headless solvability checker for Magnetar levels.
//
// Reuses the SAME integrator, collision tests, and body-motion code the game
// runs, so a "win" here is a real win in-game. For each level it sweeps the
// launch space (angle × thrust × fire-time) and reports:
//   • whether ANY launch wins (impossible levels are bugs)
//   • the size of the solution basin (too few = needle-thread = unfair)
//   • a representative central solution to seed initialAim / initialThrust
//
// Run:  bun run scripts/solve-magnetar.ts
// This is a dev tool — it is not bundled into the site.

import { LEVELS, type LevelSpec } from '../src/games/magnetar/levels.ts'
import { earthAt, planetsAt } from '../src/games/magnetar/bodies.ts'
import { hitsPlanet, hitsTarget, hitsWall, integrate, MAX_SPEED, MIN_SPEED, PROJECTILE_R } from '../src/games/magnetar/physics.ts'

const STAGE_W = 480
const STAGE_H = 540
const DT = 1 / 60
const MAX_FLIGHT_STEPS = 720

interface Attempt { angle: number; speed: number; fireTime: number }
interface Result extends Attempt { steps: number }

function maxPeriod(s: LevelSpec): number {
  let m = 0
  for (const b of s.planets) {
    if (b.orbit) m = Math.max(m, b.orbit.period)
    if (b.osc) m = Math.max(m, b.osc.period)
    if (b.pulse) m = Math.max(m, b.pulse.period)
  }
  if (s.target.orbit) m = Math.max(m, s.target.orbit.period)
  if (s.target.osc) m = Math.max(m, s.target.osc.period)
  return m
}

/** Simulate, recording the full path. For trace mode. */
function trace(s: LevelSpec, a: Attempt, ignoreRescues: boolean): { win: boolean; path: Array<{ x: number; y: number }> } {
  let pos = { x: s.launch.x, y: s.launch.y }
  let vel = { x: Math.cos(a.angle) * a.speed, y: Math.sin(a.angle) * a.speed }
  const path: Array<{ x: number; y: number }> = []
  const total = ignoreRescues ? 0 : (s.rescues?.length ?? 0)
  for (let i = 0; i < MAX_FLIGHT_STEPS; i++) {
    const t = a.fireTime + i * DT
    const planets = planetsAt(s.planets, t)
    const step = integrate(pos, vel, planets, DT)
    pos = step.pos; vel = step.vel
    path.push({ x: Math.round(pos.x), y: Math.round(pos.y) })
    if (hitsTarget(pos, earthAt(s.target, t))) return { win: total === 0, path }
    if (hitsPlanet(pos, planets) || hitsWall(pos, s.walls)) return { win: false, path }
    if (pos.x < -PROJECTILE_R || pos.x > STAGE_W + PROJECTILE_R || pos.y < -PROJECTILE_R || pos.y > STAGE_H + PROJECTILE_R) return { win: false, path }
  }
  return { win: false, path }
}

/** Simulate one launch. Returns step count on a win, or null on miss/crash. */
function simulate(s: LevelSpec, a: Attempt): number | null {
  let pos = { x: s.launch.x, y: s.launch.y }
  let vel = { x: Math.cos(a.angle) * a.speed, y: Math.sin(a.angle) * a.speed }
  const rescued = new Set<number>()
  const total = s.rescues?.length ?? 0
  for (let i = 0; i < MAX_FLIGHT_STEPS; i++) {
    const t = a.fireTime + i * DT
    const planets = planetsAt(s.planets, t)
    const step = integrate(pos, vel, planets, DT)
    pos = step.pos
    vel = step.vel
    if (s.rescues) {
      s.rescues.forEach((r, idx) => {
        if (rescued.has(idx)) return
        const dx = r.x - pos.x
        const dy = r.y - pos.y
        if (dx * dx + dy * dy < (r.radius + PROJECTILE_R) ** 2) rescued.add(idx)
      })
    }
    if (hitsTarget(pos, earthAt(s.target, t))) return rescued.size >= total ? i : null
    if (hitsPlanet(pos, planets) || hitsWall(pos, s.walls)) return null
    if (pos.x < -PROJECTILE_R || pos.x > STAGE_W + PROJECTILE_R || pos.y < -PROJECTILE_R || pos.y > STAGE_H + PROJECTILE_R) return null
  }
  return null
}

const ANGLE_STEPS = 220
const SPEED_STEPS = 26
const TIME_STEPS = 28

function sweep(s: LevelSpec): { wins: Result[]; total: number } {
  const wins: Result[] = []
  const period = maxPeriod(s)
  const times = period > 0 ? Array.from({ length: TIME_STEPS }, (_, k) => (k / TIME_STEPS) * period) : [0]
  let total = 0
  for (let ai = 0; ai < ANGLE_STEPS; ai++) {
    const angle = -Math.PI + (ai / ANGLE_STEPS) * Math.PI * 2
    for (let si = 0; si <= SPEED_STEPS; si++) {
      const speed = MIN_SPEED + (si / SPEED_STEPS) * (MAX_SPEED - MIN_SPEED)
      for (const fireTime of times) {
        total++
        const steps = simulate(s, { angle, speed, fireTime })
        if (steps !== null) wins.push({ angle, speed, fireTime, steps })
      }
    }
  }
  return { wins, total }
}

function median<T>(arr: T[], pick: (v: T) => number): number {
  const xs = arr.map(pick).sort((a, b) => a - b)
  return xs[Math.floor(xs.length / 2)] ?? 0
}

const deg = (r: number) => `${Math.round((r * 180) / Math.PI)}°`

// Trace mode: `bun run scripts/solve-magnetar.ts trace <Level> [ignore]`
// Finds a winning launch (optionally ignoring rescues so you can read the base
// path and place pods on it) and prints sampled points along it.
if (process.argv[2] === 'trace') {
  const name = process.argv[3]
  const ignore = process.argv[4] === 'ignore'
  const s = LEVELS.find((l) => l.name.toLowerCase() === name?.toLowerCase())
  if (!s) { console.log('no such level:', name); process.exit(1) }
  const { wins } = sweep(s)
  const base = wins.length ? wins.sort((a, b) => a.steps - b.steps) : []
  // If the level (with rescues) has no win, fall back to a sweep ignoring them.
  let attempt: Attempt | null = base[Math.floor(base.length / 2)] ?? null
  if (!attempt && ignore) {
    outer: for (let ai = 0; ai < 220; ai++) {
      const angle = -Math.PI + (ai / 220) * Math.PI * 2
      for (let si = 0; si <= 26; si++) {
        const speed = MIN_SPEED + (si / 26) * (MAX_SPEED - MIN_SPEED)
        if (trace(s, { angle, speed, fireTime: 0 }, true).win) { attempt = { angle, speed, fireTime: 0 }; break outer }
      }
    }
  }
  if (!attempt) { console.log('no winning launch found for', name); process.exit(0) }
  const tr = trace(s, attempt, ignore)
  console.log(`${s.name}: aim ${deg(attempt.angle)} thrust ${Math.round(attempt.speed)} fire@${attempt.fireTime.toFixed(2)} win=${tr.win}`)
  const n = tr.path.length
  for (let k = 0; k <= 10; k++) {
    const pt = tr.path[Math.min(n - 1, Math.round((k / 10) * (n - 1)))]!
    console.log(`  ${String(Math.round((k / 10) * 100)).padStart(3)}%  (${pt.x}, ${pt.y})`)
  }
  process.exit(0)
}

// Pod-spot finder: `bun run scripts/solve-magnetar.ts podspot <Level>`
// Sweeps the level IGNORING its rescues, overlays every winning path on a grid,
// and reports the cells the most winning paths pass through (away from launch /
// Earth). Drop a pod there and the largest share of winning launches collect it
// for free — a fat basin instead of a needle.
if (process.argv[2] === 'podspot') {
  const s = LEVELS.find((l) => l.name.toLowerCase() === process.argv[3]?.toLowerCase())
  if (!s) { console.log('no such level'); process.exit(1) }
  const period = maxPeriod(s)
  const times = period > 0 ? Array.from({ length: 10 }, (_, k) => (k / 10) * period) : [0]
  const CELL = 22
  const counts = new Map<string, number>()
  const near = (x: number, y: number, p: { x: number; y: number }, d: number) => (x - p.x) ** 2 + (y - p.y) ** 2 < d * d
  for (let ai = 0; ai < 220; ai++) {
    const angle = -Math.PI + (ai / 220) * Math.PI * 2
    for (let si = 0; si <= 26; si++) {
      const speed = MIN_SPEED + (si / 26) * (MAX_SPEED - MIN_SPEED)
      for (const fireTime of times) {
        const tr = trace(s, { angle, speed, fireTime }, true)
        if (!tr.win) continue
        const seen = new Set<string>()
        for (const pt of tr.path) {
          if (near(pt.x, pt.y, s.launch, 70) || near(pt.x, pt.y, s.target, s.target.radius + 40)) continue
          const key = `${Math.round(pt.x / CELL)},${Math.round(pt.y / CELL)}`
          if (seen.has(key)) continue
          seen.add(key)
          counts.set(key, (counts.get(key) ?? 0) + 1)
        }
      }
    }
  }
  const top = [...counts.entries()].sort((a, b) => b[1] - a[1]).slice(0, 8)
  console.log(`${s.name}: pod-spot candidates (cell≈${CELL}px, count = winning launches passing through)`)
  for (const [key, n] of top) {
    const [cx, cy] = key.split(',').map(Number)
    console.log(`  (${cx! * CELL}, ${cy! * CELL})  ${n} launches`)
  }
  process.exit(0)
}

console.log('Magnetar solvability sweep\n' + '='.repeat(64))
let anyImpossible = false
let anyNeedle = false
for (const s of LEVELS) {
  const { wins, total } = sweep(s)
  const pct = ((wins.length / total) * 100).toFixed(1)
  const tag = wins.length === 0 ? '  ✖ IMPOSSIBLE' : wins.length / total < 0.01 ? '  ⚠ needle' : '  ✓'
  if (wins.length === 0) anyImpossible = true
  if (wins.length > 0 && wins.length / total < 0.01) anyNeedle = true
  let detail = ''
  if (wins.length) {
    const mAng = median(wins, (w) => w.angle)
    const mSpd = median(wins, (w) => w.speed)
    const mTime = median(wins, (w) => w.fireTime)
    detail = ` · basin ${pct}% · suggest aim ${deg(mAng)} thrust ${Math.round(mSpd)}` + (mTime ? ` fire@${mTime.toFixed(2)}s` : '')
  }
  console.log(`${s.name.padEnd(13)} ${String(wins.length).padStart(6)}/${total}${tag}${detail}`)
}
console.log('='.repeat(64))
console.log(anyImpossible ? 'FAIL: at least one level is impossible.' : anyNeedle ? 'WARN: some levels are needle-thin.' : 'OK: every level has a comfortable solution basin.')
