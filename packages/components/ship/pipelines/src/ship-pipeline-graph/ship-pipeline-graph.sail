<scope>
  props: {
    steps = '[]'
    name = ''
    status: 'passing' | 'failing' | 'running' | 'none' = 'none'
    run = ''
  }
</scope>
<controller>
// <ship-pipeline-graph> — renders a JSON DAG as a hand-rolled horizontal flow.
// Nodes + connectors are generated as a runtime HTML string via bindHtml.
// A delegated listener emits `ship-select { node }` on click / Enter / Space.
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './ship-pipeline-graph.sail'
import { bindHtml } from '../bind-html'

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
const EMPTY = '<div>no steps</div>'

function esc(s: string): string {
  return String(s).replace(/[&<>"]/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c)
}

function node(s: Step): string {
  const state = STATES.has(s.state ?? '') ? s.state! : 'queued'
  const label = esc(s.label ?? s.id)
  const blast = BLAST.has(s.blast ?? '')
    ? `<span>${s.blast}</span>` : ''
  return `<div data-node-id="${esc(s.id)}" tabindex="0" `
    + `role="button" aria-label="${label} — ${state}">`
    + `<div><span></span><span>${label}</span>${blast}</div>`
    + `<div><i></i></div>`
    + `<span>${esc(s.sub ?? '')}</span></div>`
}

function render(steps: Step[]): string {
  if (!Array.isArray(steps) || steps.length === 0) return EMPTY
  return steps
    .map((s, i) => {
      if (i === 0) return node(s)
      return `<div></div>${node(s)}`
    })
    .join('')
}

export default controller((ctx: Scope) => {
  const paint: Action<void> = (el) => {
    const select = (target: unknown) => {
      const card = (target as { closest?: (sel: string) => { getAttribute: (n: string) => string | null } | null } | null)
        ?.closest?.('[data-node-id]')
      const id = card?.getAttribute('data-node-id')
      if (!id) return
      hostDispatch(el, 'ship-select', {
        bubbles: true, composed: true, detail: { node: id },
      })
    }
    const onClick = (e: { target?: unknown }) => select(e.target)
    const onKey = (e: { key?: string; preventDefault?: () => void; target?: unknown }) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const card = (e.target as { closest?: (sel: string) => unknown } | null)?.closest?.('[data-node-id]')
      if (!card) return
      e.preventDefault?.()
      select(e.target)
    }
    el.addEventListener('click', onClick)
    el.addEventListener('keydown', onKey)

    const stop = bindHtml(el, () => {
      try {
        return render(JSON.parse(ctx.steps() || '[]') as Step[])
      } catch {
        return EMPTY
      }
    })

    return {
      destroy: () => {
        stop()
        el.removeEventListener('click', onClick)
        el.removeEventListener('keydown', onKey)
      },
    }
  }
  return { paint }
})
</controller>

<!--
  <ship-pipeline-graph> — the run-graph surface. Renders a JSON DAG as a
  bespoke, hand-rolled horizontal stage flow: each step is a node with a
  status dot + progress bar, joined by completion-tinted connectors. Status
  and gate visuals map to ship tone tokens (success · info · warning ·
  destructive · muted); optional blast tiers C0–C3 escalate through the
  same ramp. Prop-driven only — no SDK calls.

  Props:  steps  — JSON array of { id, label?, state?, sub?, after?, progress?, blast? }
          name   — pipeline name (header)
          status — passing|failing|running|none (header badge)
          run    — run-meta string (header, right-aligned)
  Events: ship-select { node } — a node was activated (click / Enter / Space)
  Parts:  graph · header · flow
-->

<div>
  <div>
    <span>{name()}</span>
    <span>{status()}</span>
    <span>{run()}</span>
  </div>
  <div use:paint></div>
</div>
