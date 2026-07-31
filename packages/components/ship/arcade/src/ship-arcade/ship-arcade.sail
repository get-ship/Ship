<controller>
// <ship-arcade> — renders a JSON list of games as launcher tiles. Tiles are
// generated as a runtime HTML string via bindHtml (tone tokens compose
// freely). A delegated listener emits `ship-select { game }` on click /
// Enter / Space. Chrome only — no contact with the PixiJS engine.
import { controller, ref, bindHtml } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-arcade.sail'

interface Game {
  id: string
  title?: string
  meta?: string
  tone?: string
  art?: string
}

const TONES = new Set(['success', 'info', 'warning', 'destructive', 'accent', 'primary'])
const EMPTY = '<div>no games</div>'

function esc(s: string): string {
  return String(s).replace(/[&<>"]/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c)
}

function tile(g: Game): string {
  const title = g.title ?? g.id
  const tone = TONES.has(g.tone ?? '') ? ` t-${g.tone}` : ''
  const art = esc((g.art ?? (title.charAt(0) || '?')).slice(0, 2))
  return `<div data-game-id="${esc(g.id)}" tabindex="0" `
    + `role="button" aria-label="${esc(title)}">`
    + `<div>${art}</div>`
    + `<div><div>${esc(title)}</div>`
    + `<div>${esc(g.meta ?? '')}</div></div></div>`
}

function render(games: Game[]): string {
  if (!Array.isArray(games) || games.length === 0) return EMPTY
  return games.map(tile).join('')
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ gridEl: ref<HTMLElement>(null) }),

  mount(state, host) {
    const el = state.gridEl as HTMLElement | null
    if (!el) return () => {}

    const select = (target: EventTarget | null) => {
      const t = (target as HTMLElement | null)?.closest?.('[data-game-id]')
      const id = t?.getAttribute('data-game-id')
      if (!id) return
      host.dispatchEvent(new CustomEvent('ship-select', {
        detail: { game: id }, bubbles: true, composed: true,
      }))
    }
    const onClick = (e: Event) => select(e.target)
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const t = (e.target as HTMLElement | null)?.closest?.('[data-game-id]')
      if (!t) return
      e.preventDefault()
      select(e.target)
    }
    el.addEventListener('click', onClick)
    el.addEventListener('keydown', onKey)

    const stop = bindHtml(el, () => {
      try {
        return render(JSON.parse((state.games as string) || '[]') as Game[])
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
  <ship-arcade> — the arcade LAUNCHER surface. Renders a JSON list of games as a
  grid of tone-tinted tiles (art block + initial, title, meta line). Tile tone
  maps to a ship tone token so themes re-skin the whole grid. Prop-driven only;
  this is chrome over the sui-* arcade primitives — it does not touch the
  PixiJS engine.

  Props:  games   — JSON array of { id, title?, meta?, tone?, art? }
          heading — grid heading (default "Arcade")
          sub     — heading sub-line
  Events: ship-select { game } — a tile was activated (click / Enter / Space)
  Parts:  surface · grid
-->
<!-- <prop name="games" type="string" default="[]" reflect /> -->
<!-- <prop name="heading" type="string" default="Arcade" /> -->
<!-- <prop name="sub" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div><h3>{heading()}</h3><span>{sub()}</span></div>
  <div bind:this={gridEl}></div>
</div>
