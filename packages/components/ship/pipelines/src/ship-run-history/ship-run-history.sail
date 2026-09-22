<scope>
  props: {
    runs = '[]'
    title = 'Recent runs'
    meta = ''
  }
</scope>
<controller>
// <ship-run-history> — renders a JSON array of runs as hairline table rows.
// Rows are generated as a runtime HTML string via bindHtml. A delegated
// listener emits `ship-select { run }` on click / Enter / Space.
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './ship-run-history.sail'
import { bindHtml } from '../bind-html'

interface Run {
  id: string
  status?: string
  trigger?: string
  duration?: string
  durationPct?: number
  started?: string
}

const STATUS = new Set(['success', 'failed', 'running'])
const EMPTY = '<div>no runs</div>'

function esc(s: string): string {
  return String(s).replace(/[&<>"]/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c)
}

function row(r: Run): string {
  const status = STATUS.has(r.status ?? '') ? r.status! : ''
  return `<div data-run-id="${esc(r.id)}" tabindex="0" role="button" `
    + `aria-label="run ${esc(r.id)}${status ? ' — ' + status : ''}">`
    + `<span></span>`
    + `<span>${esc(r.id)}</span>`
    + `<span>${esc(r.trigger ?? '')}</span>`
    + `<span><span>${esc(r.duration ?? '')}</span>`
    + `<span><i></i></span></span>`
    + `<span>${esc(r.started ?? '')}</span></div>`
}

function render(runs: Run[]): string {
  if (!Array.isArray(runs) || runs.length === 0) return EMPTY
  return runs.map(row).join('')
}

export default controller((ctx: Scope) => {
  const paint: Action<void> = (el) => {
    const select = (target: unknown) => {
      const r = (target as { closest?: (sel: string) => { getAttribute: (n: string) => string | null } | null } | null)
        ?.closest?.('[data-run-id]')
      const id = r?.getAttribute('data-run-id')
      if (!id) return
      hostDispatch(el, 'ship-select', {
        bubbles: true, composed: true, detail: { run: id },
      })
    }
    const onClick = (e: { target?: unknown }) => select(e.target)
    const onKey = (e: { key?: string; preventDefault?: () => void; target?: unknown }) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const r = (e.target as { closest?: (sel: string) => unknown } | null)?.closest?.('[data-run-id]')
      if (!r) return
      e.preventDefault?.()
      select(e.target)
    }
    el.addEventListener('click', onClick)
    el.addEventListener('keydown', onKey)

    const stop = bindHtml(el, () => {
      try {
        return render(JSON.parse(ctx.runs() || '[]') as Run[])
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
  <ship-run-history> — the run-history surface. Renders a JSON array of runs as
  a hairline table: status dot · run id · trigger · duration (number + relative
  bar) · started. Status maps to ship tone tokens (success · destructive ·
  info). Prop-driven only — no SDK calls.

  Props:  runs    — JSON array of { id, status?, trigger?, duration?, durationPct?, started? }
          title   — header label (default "Recent runs")
          meta    — header right-aligned context line
  Events: ship-select { run } — a row was activated (click / Enter / Space)
  Parts:  history · header
-->

<div>
  <div><span>{title()}</span><span>{meta()}</span></div>
  <div><span></span><span>Run</span><span>Trigger</span><span>Duration</span><span>Started</span></div>
  <div use:paint></div>
</div>
