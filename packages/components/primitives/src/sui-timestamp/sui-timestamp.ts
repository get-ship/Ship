// Pure helpers for <sui-timestamp>. Kept in a sibling .ts so tests can
// import without pulling in the controller's runtime registration.

const MINUTE = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

export function formatRelative(iso: string, now = Date.now()): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return iso
  const diff = now - t
  const abs = Math.abs(diff)
  const suffix = diff >= 0 ? 'ago' : 'from now'
  if (abs < MINUTE) return 'just now'
  if (abs < HOUR) return `${Math.floor(abs / MINUTE)}m ${suffix}`
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ${suffix}`
  const days = Math.floor(abs / DAY)
  if (days < 30) return `${days}d ${suffix}`
  if (days < 365) return `${Math.floor(days / 30)}mo ${suffix}`
  return `${Math.floor(days / 365)}y ${suffix}`
}

export function absoluteIso(value: string): string {
  const parsed = Date.parse(value)
  return Number.isNaN(parsed) ? value : new Date(parsed).toISOString()
}
