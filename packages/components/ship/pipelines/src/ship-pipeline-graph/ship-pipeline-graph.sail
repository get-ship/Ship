<controller>
// <ship-pipeline-graph> — renders a JSON DAG as a hand-rolled horizontal flow.
// Nodes + connectors are generated as a runtime HTML string via bindHtml, so
// tone tokens compose freely (no compile-time token lint on generated markup).
// A delegated listener emits `ship-select { node }` on click / Enter / Space.
import { controller, ref, bindHtml } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-pipeline-graph.sail'

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
  const pct = typeof s.progress === 'number'
    ? `` : ''
  return `<div data-node-id="${esc(s.id)}" tabindex="0" `
    + `role="button" aria-label="${label} — ${state}">`
    + `<div><span></span><span>${label}</span>${blast}</div>`
    + `<div><i${pct}></i></div>`
    + `<span>${esc(s.sub ?? '')}</span></div>`
}

function render(steps: Step[]): string {
  if (!Array.isArray(steps) || steps.length === 0) return EMPTY
  return steps
    .map((s, i) => {
      if (i === 0) return node(s)
      const prevDone = steps[i - 1]?.state === 'done'
      return `<div></div>${node(s)}`
    })
    .join('')
}

export default controller({
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
<!-- <prop name="steps" type="string" default="[]" reflect /> -->
<!-- <prop name="name" type="string" default="" /> -->
<!-- <prop name="status" values="passing|failing|running|none" default="none" reflect /> -->
<!-- <prop name="run" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div>
    <span>{name()}</span>
    <span>{status()}</span>
    <span>{run()}</span>
  </div>
  <div bind:this={flowEl}></div>
</div>
