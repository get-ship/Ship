import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-arcade/ship-arcade.compiled.js'

// ship-arcade is launcher CHROME — a prop-driven grid of game tiles rendered
// via bindHtml. It never touches the PixiJS engine, so its full contract is
// reachable in a DOM-only environment (the engine lives behind <sui-stage>,
// which this component does not mount). No engine mock is needed here.

const containers: HTMLElement[] = []
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

function mount(attrs: Record<string, string> = {}) {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  const el = document.createElement('ship-arcade')
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  c.appendChild(el)
  return el
}

const flush = () => new Promise<void>((r) => setTimeout(r, 0))
const shadow = (el: HTMLElement) => el.shadowRoot ?? el
const gamesAttr = (g: unknown) => JSON.stringify(g)

const FIXTURE = [
  { id: 'snake', title: 'Snake', meta: 'hi 1200', tone: 'success', art: 'SN' },
  { id: 'pong', title: 'Pong', meta: '2P', tone: 'info' },
  { id: 'tetris' },
]

describe('<ship-arcade>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-arcade')).toBeDefined()
  })

  it('renders the heading and sub-line from props', async () => {
    const el = mount({ heading: 'Cabinet', sub: '3 games' })
    await flush()
    expect(shadow(el).querySelector('.head h3')?.textContent).toBe('Cabinet')
    expect(shadow(el).querySelector('.head .sub')?.textContent).toBe('3 games')
  })

  it('defaults the heading to "Arcade"', async () => {
    const el = mount({ games: '[]' })
    await flush()
    expect(shadow(el).querySelector('.head h3')?.textContent).toBe('Arcade')
  })

  it('renders one tile per game with a data-game-id', async () => {
    const el = mount({ games: gamesAttr(FIXTURE) })
    await flush()
    const tiles = shadow(el).querySelectorAll('.tile[data-game-id]')
    expect(tiles.length).toBe(3)
    expect([...tiles].map((t) => t.getAttribute('data-game-id'))).toEqual([
      'snake', 'pong', 'tetris',
    ])
  })

  it('maps a known tone to its tile tone class and titles each tile', async () => {
    const el = mount({ games: gamesAttr(FIXTURE) })
    await flush()
    const tiles = shadow(el).querySelectorAll('.tile')
    expect(tiles[0].classList.contains('t-success')).toBe(true)
    expect(tiles[1].classList.contains('t-info')).toBe(true)
    expect(tiles[0].querySelector('.t')?.textContent).toBe('Snake')
  })

  it('falls back to the id as the title when none is given', async () => {
    const el = mount({ games: gamesAttr(FIXTURE) })
    await flush()
    const tetris = shadow(el).querySelector('[data-game-id="tetris"]')!
    expect(tetris.querySelector('.t')?.textContent).toBe('tetris')
    expect(tetris.getAttribute('aria-label')).toBe('tetris')
  })

  it('exposes each tile as a focusable button', async () => {
    const el = mount({ games: gamesAttr(FIXTURE) })
    await flush()
    const tile = shadow(el).querySelector('.tile')!
    expect(tile.getAttribute('role')).toBe('button')
    expect(tile.getAttribute('tabindex')).toBe('0')
  })

  it('emits ship-select with the game id on click', async () => {
    const el = mount({ games: gamesAttr(FIXTURE) })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-select', onSelect)
    ;(shadow(el).querySelectorAll('.tile')[1] as HTMLElement).click()
    expect(onSelect).toHaveBeenCalledTimes(1)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ game: 'pong' })
  })

  it('emits ship-select on Enter and Space keydown', async () => {
    const el = mount({ games: gamesAttr(FIXTURE) })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-select', onSelect)
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    for (const key of ['Enter', ' ']) {
      tile.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))
    }
    expect(onSelect).toHaveBeenCalledTimes(2)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ game: 'snake' })
  })

  it('renders the empty placeholder for no games', async () => {
    const el = mount({ games: '[]' })
    await flush()
    expect(shadow(el).querySelector('.empty')?.textContent).toBe('no games')
  })

  it('falls back to the empty placeholder on invalid JSON', async () => {
    const el = mount({ games: 'not-json' })
    await flush()
    expect(shadow(el).querySelector('.empty')?.textContent).toBe('no games')
  })
})
