<!--
  <ship-arcade> — the arcade LAUNCHER surface. Renders a JSON list of games as a
  grid of tiles (art block + initial, title, meta line). Prop-driven only;
  this is chrome over the sui-* arcade primitives — it does not touch the
  PixiJS engine.

  Props:  games   — JSON array of { id, title?, meta?, tone?, art? }
          heading — grid heading (default "Arcade")
          sub     — heading sub-line
  Events: ship-select { game } — a tile was activated (click / Enter / Space)
-->
<scope>
  props: {
    games = '[]'
    heading = 'Arcade'
    sub = ''
  }
  style muted = { type({ size: sm }).fg(muted-foreground) }
  style art = { type({ family: mono, size: 2xl, weight: bold }).fg(muted-foreground) }
  style title = { type({ size: sm, weight: medium }) }
  style meta = { type({ size: xs }).fg(muted-foreground) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './ship-arcade.sail'
import { emitSelect, type Host, type KeyEv } from './host'

interface Game {
  id: string
  title?: string
  meta?: string
  tone?: string
  art?: string
}

interface Tile {
  id: string
  title: string
  meta: string
  art: string
}

function tilesOf(raw: string): Tile[] {
  try {
    const parsed = JSON.parse(raw || '[]') as unknown
    if (!Array.isArray(parsed)) return []
    const out: Tile[] = []
    for (const item of parsed) {
      if (!item || typeof item !== 'object') continue
      const g = item as Game
      if (!g.id) continue
      const title = g.title ?? g.id
      out.push({
        id: g.id,
        title,
        meta: g.meta ?? '',
        art: (g.art ?? (title.charAt(0) || '?')).slice(0, 2),
      })
    }
    return out
  } catch {
    return []
  }
}

export default controller((ctx: ControllerScope<Scope>, host?: Host) => {
  const tiles = derived<Tile[]>(() => tilesOf(ctx.games()))
  const hasGames = derived(() => tiles().length > 0)

  const select = (id: string) => { emitSelect(host, id) }
  const onKey = (e: KeyEv, id: string) => {
    if (e.key !== 'Enter' && e.key !== ' ') return
    e.preventDefault()
    select(id)
  }
  return { tiles, hasGames, select, onKey }
})
</controller>

<column gap="3">
  <row gap="2" align="center">
    <h3>{heading()}</h3>
    <span style(muted)>{sub()}</span>
  </row>
  <if {hasGames()}>
    <column gap="2">
      <for {tiles() as g} key={g.id}>
        <button type="button" label={g.title} on:click={() => select(g.id)} on:keydown={(e) => onKey(e, g.id)}>
          <column gap="1">
            <span style(art)>{g.art}</span>
            <span style(title)>{g.title}</span>
            <span style(meta)>{g.meta}</span>
          </column>
        </button>
      </for>
    </column>
  {else =>
    <span style(muted)>no games</span>
  }</if>
</column>
