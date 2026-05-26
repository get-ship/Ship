// <sui-pipeline-dag> — graph form. Lays out a step DAG into dependency columns
// (level = longest path from a root) and renders type-coloured nodes + state
// fills + bezier edges (active edge animated) as SVG. SVG is rendered via
// bindHtml (runtime string), so it composes tokens freely — no compile-time
// token lint on the generated markup.
import { controller, bindHtml } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-dag.sui'

interface Step { id: string; type?: string; state?: string; after?: string[] }

const COL = 132, ROW = 54, NW = 104, NH = 40, PAD = 16

const STATE_COLOR: Record<string, string> = {
  ok: 'var(--success)',
  running: 'var(--info)',
  awaiting: 'var(--warning)',
  err: 'var(--destructive)',
  never: 'var(--muted-foreground)',
}

function esc(s: string): string {
  return String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c)
}

const EMPTY_HTML = '<div class="dag-empty">no steps</div>'

function render(steps: Step[]): string {
  if (!Array.isArray(steps) || steps.length === 0) return EMPTY_HTML
  const byId = new Map(steps.map((s) => [s.id, s]))

  // level = longest dependency path from a root (cycle-safe).
  const level = new Map<string, number>()
  const compute = (id: string, stack: Set<string>): number => {
    if (level.has(id)) return level.get(id)!
    if (stack.has(id)) return 0
    stack.add(id)
    const deps = (byId.get(id)?.after ?? []).filter((d) => byId.has(d))
    const l = deps.length ? Math.max(...deps.map((d) => compute(d, stack))) + 1 : 0
    stack.delete(id)
    level.set(id, l)
    return l
  }
  steps.forEach((s) => compute(s.id, new Set()))

  const cols = new Map<number, Step[]>()
  steps.forEach((s) => {
    const l = level.get(s.id) ?? 0
    if (!cols.has(l)) cols.set(l, [])
    cols.get(l)!.push(s)
  })
  const maxLevel = Math.max(...level.values())
  const maxCount = Math.max(...[...cols.values()].map((c) => c.length))
  const H = maxCount * ROW + PAD * 2
  const W = maxLevel * COL + NW + PAD * 2

  const pos = new Map<string, { x: number; y: number; s: Step }>()
  for (const [l, items] of cols) {
    const colH = items.length * ROW
    items.forEach((s, i) => {
      pos.set(s.id, { x: PAD + l * COL, y: (H - colH) / 2 + i * ROW + (ROW - NH) / 2, s })
    })
  }

  const edges = steps
    .flatMap((s) => (s.after ?? []).filter((d) => pos.has(d) && pos.has(s.id)).map((d) => {
      const a = pos.get(d)!, b = pos.get(s.id)!
      const x1 = a.x + NW, y1 = a.y + NH / 2, x2 = b.x, y2 = b.y + NH / 2, mx = (x1 + x2) / 2
      const active = b.s.state === 'running' || b.s.state === 'awaiting'
      const path = `M ${x1} ${y1} C ${mx} ${y1}, ${mx} ${y2}, ${x2} ${y2}`
      return active
        ? `<path d="${path}" stroke="${STATE_COLOR[b.s.state!]}" stroke-width="1.6" stroke-dasharray="3 3" fill="none" marker-end="url(#pl-active)"><animate attributeName="stroke-dashoffset" from="0" to="-18" dur="1.2s" repeatCount="indefinite"/></path>`
        : `<path d="${path}" stroke="var(--success)" stroke-width="1.3" fill="none" opacity="0.5" marker-end="url(#pl-arrow)"/>`
    }))
    .join('')

  const nodes = steps
    .map((s) => {
      const p = pos.get(s.id)!
      const type = s.type ?? 'agent'
      const st = s.state ?? 'ok'
      const typeC = `var(--type-${type})`
      const stC = STATE_COLOR[st] ?? 'var(--muted-foreground)'
      const dot = st === 'running' || st === 'awaiting'
        ? `<circle cx="${NW - 13}" cy="11" r="3.5" fill="${stC}"><animate attributeName="opacity" values="1;0.3;1" dur="1.6s" repeatCount="indefinite"/></circle>`
        : `<circle cx="${NW - 13}" cy="11" r="3" fill="${stC}"/>`
      return `<g transform="translate(${p.x},${p.y})">`
        + `<rect width="${NW}" height="${NH}" rx="6" fill="color-mix(in oklch, ${typeC} 8%, var(--card))" stroke="color-mix(in oklch, ${typeC} 45%, var(--border))"/>`
        + `<rect width="2" height="${NH}" fill="${typeC}"/>`
        + `<text x="9" y="16" font-family="var(--font-mono)" font-size="7" fill="${typeC}" font-weight="700">${esc(type.toUpperCase())}</text>`
        + `<text x="9" y="29" font-family="var(--font-mono)" font-size="9.5" fill="var(--foreground)" font-weight="600">${esc(s.id)}</text>`
        + dot + `</g>`
    })
    .join('')

  return `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid meet" role="img" aria-label="pipeline graph">`
    + `<defs>`
    + `<marker id="pl-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0,0 L10,5 L0,10 z" fill="var(--success)" opacity="0.6"/></marker>`
    + `<marker id="pl-active" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0,0 L10,5 L0,10 z" fill="var(--warning)"/></marker>`
    + `</defs>${edges}${nodes}</svg>`
}

export default controller<Scope>({
  mount(state, _host) {
    const el = state.$signals.contentEl() as HTMLElement | null
    if (!el) return () => {}
    return bindHtml(el, () => {
      try {
        return render(JSON.parse(state.$signals.steps() || '[]') as Step[])
      } catch {
        return EMPTY_HTML
      }
    })
  },
})
