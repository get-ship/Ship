// <ship-pipeline-graph> — renders a JSON DAG as a hand-rolled horizontal flow.
// Nodes + connectors are generated as a runtime HTML string via bindHtml, so
// tone tokens compose freely (no compile-time token lint on generated markup).
// A delegated listener emits `ship-select { node }` on click / Enter / Space.
import { controller, ref, bindHtml } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-pipeline-graph.sui'

interface Step {
  id: string
  label?: string
  state?: string
  sub?: string
  after?: string[]
  progress?: number
  blast?: string
}

const STATES = new Set(['done', 'running', 'failed', 'queued', 'awaiting'])
const BLAST = new Set(['C0', 'C1', 'C2', 'C3'])
const EMPTY = '<div class="empty">no steps</div>'

function esc(s: string): string {
  return String(s).replace(/[&<>"]/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c)
}

function node(s: Step): string {
  const state = STATES.has(s.state ?? '') ? s.state! : 'queued'
  const label = esc(s.label ?? s.id)
  const blast = BLAST.has(s.blast ?? '')
    ? `<span class="blast bl-${s.blast}">${s.blast}</span>` : ''
  const pct = typeof s.progress === 'number'
    ? ` style="width:${Math.max(0, Math.min(100, s.progress))}%"` : ''
  return `<div class="node st-${state}" data-node-id="${esc(s.id)}" tabindex="0" `
    + `role="button" aria-label="${label} — ${state}">`
    + `<div class="row"><span class="dot"></span><span class="lbl">${label}</span>${blast}</div>`
    + `<div class="bar"><i${pct}></i></div>`
    + `<span class="sub">${esc(s.sub ?? '')}</span></div>`
}

function render(steps: Step[]): string {
  if (!Array.isArray(steps) || steps.length === 0) return EMPTY
  return steps
    .map((s, i) => {
      if (i === 0) return node(s)
      const prevDone = steps[i - 1]?.state === 'done'
      return `<div class="conn${prevDone ? ' done' : ''}"></div>${node(s)}`
    })
    .join('')
}

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => ({ flowEl: ref<HTMLElement>(null) }),

  mount(state, host) {
    const el = state.flowEl as HTMLElement | null
    if (!el) return () => {}

    const select = (target: EventTarget | null) => {
      const card = (target as HTMLElement | null)?.closest?.('[data-node-id]')
      const id = card?.getAttribute('data-node-id')
      if (!id) return
      host.dispatchEvent(new CustomEvent('ship-select', {
        detail: { node: id }, bubbles: true, composed: true,
      }))
    }
    const onClick = (e: Event) => select(e.target)
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const card = (e.target as HTMLElement | null)?.closest?.('[data-node-id]')
      if (!card) return
      e.preventDefault()
      select(e.target)
    }
    el.addEventListener('click', onClick)
    el.addEventListener('keydown', onKey)

    const stop = bindHtml(el, () => {
      try {
        return render(JSON.parse((state.steps as string) || '[]') as Step[])
      } catch {
        return EMPTY
      }
    })

    return () => {
      stop()
      el.removeEventListener('click', onClick)
      el.removeEventListener('keydown', onKey)
    }
  },
})
