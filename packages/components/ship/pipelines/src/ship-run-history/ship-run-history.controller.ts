// <ship-run-history> — renders a JSON array of runs as hairline table rows.
// Rows are generated as a runtime HTML string via bindHtml (tone tokens
// compose freely). A delegated listener emits `ship-select { run }` on
// click / Enter / Space.
import { controller, ref, bindHtml } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-run-history.sui'

interface Run {
  id: string
  status?: string
  trigger?: string
  duration?: string
  durationPct?: number
  started?: string
}

const STATUS = new Set(['success', 'failed', 'running'])
const EMPTY = '<div class="empty">no runs</div>'

function esc(s: string): string {
  return String(s).replace(/[&<>"]/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c)
}

function row(r: Run): string {
  const status = STATUS.has(r.status ?? '') ? r.status! : ''
  const pct = Math.max(0, Math.min(100, typeof r.durationPct === 'number' ? r.durationPct : 0))
  return `<div class="row" data-run-id="${esc(r.id)}" tabindex="0" role="button" `
    + `aria-label="run ${esc(r.id)}${status ? ' — ' + status : ''}">`
    + `<span class="dot${status ? ' s-' + status : ''}"></span>`
    + `<span class="rid">${esc(r.id)}</span>`
    + `<span class="trig">${esc(r.trigger ?? '')}</span>`
    + `<span class="dur"><span class="n">${esc(r.duration ?? '')}</span>`
    + `<span class="b"><i style="width:${pct}%"></i></span></span>`
    + `<span class="when">${esc(r.started ?? '')}</span></div>`
}

function render(runs: Run[]): string {
  if (!Array.isArray(runs) || runs.length === 0) return EMPTY
  return runs.map(row).join('')
}

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => ({ rowsEl: ref<HTMLElement>(null) }),

  mount(state, host) {
    const el = state.rowsEl as HTMLElement | null
    if (!el) return () => {}

    const select = (target: EventTarget | null) => {
      const r = (target as HTMLElement | null)?.closest?.('[data-run-id]')
      const id = r?.getAttribute('data-run-id')
      if (!id) return
      host.dispatchEvent(new CustomEvent('ship-select', {
        detail: { run: id }, bubbles: true, composed: true,
      }))
    }
    const onClick = (e: Event) => select(e.target)
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const r = (e.target as HTMLElement | null)?.closest?.('[data-run-id]')
      if (!r) return
      e.preventDefault()
      select(e.target)
    }
    el.addEventListener('click', onClick)
    el.addEventListener('keydown', onKey)

    const stop = bindHtml(el, () => {
      try {
        return render(JSON.parse((state.runs as string) || '[]') as Run[])
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
