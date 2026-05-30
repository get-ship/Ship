// Shared chart helpers. Kept tiny — no d3, no deps.
// Tone → CSS var, data parsing, path math.

export type ChartTone =
  | 'default'
  | 'primary'
  | 'info'
  | 'success'
  | 'warning'
  | 'destructive'
  | 'accent'
  | 'muted'

const DEFAULT_TONE = 'var(--foreground)'

const TONE_VARS: Record<string, string> = {
  default: DEFAULT_TONE,
  primary: 'var(--primary)',
  info: 'var(--info)',
  success: 'var(--success)',
  warning: 'var(--warning)',
  destructive: 'var(--destructive)',
  accent: 'var(--accent)',
  muted: 'var(--muted-foreground)',
}

export function toneColor(tone: string | undefined | null): string {
  if (!tone) return DEFAULT_TONE
  return TONE_VARS[tone] ?? DEFAULT_TONE
}

export function parseNums(s: string | null | undefined): number[] {
  if (!s) return []
  return s
    .split(',')
    .map((p) => Number.parseFloat(p.trim()))
    .filter((n) => Number.isFinite(n))
}

export function parseLabels(s: string | null | undefined): string[] {
  if (!s) return []
  return s.split(',').map((p) => p.trim())
}

export function parseJson<T>(s: string | null | undefined, fallback: T): T {
  if (!s) return fallback
  try {
    return JSON.parse(s) as T
  } catch {
    return fallback
  }
}

export function linePath(
  data: number[],
  w: number,
  h: number,
  pad = 2,
): { d: string; points: Array<[number, number]> } {
  if (data.length === 0) return { d: '', points: [] }
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1
  const innerH = h - pad * 2
  const step = data.length === 1 ? 0 : w / (data.length - 1)
  const points: Array<[number, number]> = data.map((v, i) => {
    const x = data.length === 1 ? w / 2 : i * step
    const y = pad + innerH - ((v - min) / range) * innerH
    return [x, y]
  })
  const d = points
    .map(([x, y], i) => `${i === 0 ? 'M' : 'L'}${x.toFixed(2)},${y.toFixed(2)}`)
    .join(' ')
  return { d, points }
}

export function polar(cx: number, cy: number, r: number, angleRad: number): [number, number] {
  return [cx + r * Math.cos(angleRad), cy + r * Math.sin(angleRad)]
}

export function arcPath(
  cx: number,
  cy: number,
  r: number,
  startAngle: number,
  endAngle: number,
): string {
  const [x1, y1] = polar(cx, cy, r, startAngle)
  const [x2, y2] = polar(cx, cy, r, endAngle)
  const large = endAngle - startAngle > Math.PI ? 1 : 0
  const sweep = endAngle > startAngle ? 1 : 0
  return `M${x1.toFixed(2)},${y1.toFixed(2)} A${r},${r} 0 ${large} ${sweep} ${x2.toFixed(2)},${y2.toFixed(2)}`
}
