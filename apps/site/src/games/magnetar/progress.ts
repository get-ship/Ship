// Magnetar progression — per-level star ratings + unlock state, persisted to
// localStorage. Stars: 3 = landed on the first shot, 2 = second, 1 = third.
// Completing a level unlocks the next. This replaces the old "start at level 0
// every time" flow with a proper level-select.

const KEY = 'magnetar.progress.v2'

export interface Progress {
  /** Best stars earned per level index (0 = not cleared). */
  stars: number[]
  /** Highest level index the player may enter (0-based count, ≥1). */
  unlocked: number
}

export function loadProgress(count: number): Progress {
  const stars = new Array<number>(count).fill(0)
  let unlocked = 1
  try {
    const raw = globalThis.localStorage?.getItem(KEY)
    if (raw) {
      const p = JSON.parse(raw) as Partial<Progress>
      if (Array.isArray(p.stars)) for (let i = 0; i < count; i++) stars[i] = Math.max(0, Math.min(3, Number(p.stars[i]) || 0))
      unlocked = Math.max(1, Math.min(count, Number(p.unlocked) || 1))
    }
  } catch { /* ignore */ }
  return { stars, unlocked }
}

function save(p: Progress): void {
  try { globalThis.localStorage?.setItem(KEY, JSON.stringify(p)) } catch { /* private mode */ }
}

/** Record a clear: keep the best stars for that level, unlock the next. */
export function recordStars(p: Progress, level: number, stars: number): Progress {
  const next: Progress = { stars: p.stars.slice(), unlocked: p.unlocked }
  if (stars > (next.stars[level] ?? 0)) next.stars[level] = stars
  next.unlocked = Math.max(next.unlocked, Math.min(next.stars.length, level + 2))
  save(next)
  return next
}

export function totalStars(p: Progress): number {
  return p.stars.reduce((a, b) => a + b, 0)
}
