import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-game-frame/ship-game-frame.compiled.js'

// ship-game-frame is the in-game SURFACE chrome: a title bar (back / pause /
// restart / close), a HUD strip, a default slot for the game canvas, and a
// controls footer. It is chrome only — it never touches the PixiJS engine, so
// its whole contract is reachable in a DOM-only environment. The actual game
// (a <sui-stage> dropped into the default slot) is out of scope here; only the
// frame's prop rendering + button events are exercised.

const containers: HTMLElement[] = []
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

function mount(attrs: Record<string, string> = {}, inner = '') {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  const el = document.createElement('ship-game-frame')
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  if (inner) el.innerHTML = inner
  c.appendChild(el)
  return el
}

const flush = () => new Promise<void>((r) => setTimeout(r, 0))
const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<ship-game-frame>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-game-frame')).toBeDefined()
  })

  it('renders the game name in the title bar, defaulting to "Game"', async () => {
    const dflt = mount()
    await flush()
    expect(shadow(dflt).querySelector('.bar .nm')?.textContent).toBe('Game')

    const named = mount({ name: 'Snake' })
    await flush()
    expect(shadow(named).querySelector('.bar .nm')?.textContent).toBe('Snake')
  })

  it('renders score, lives and level into the HUD cells', async () => {
    const el = mount({ score: '1200', lives: '♥♥♥', level: '3' })
    await flush()
    const cells = shadow(el).querySelectorAll('.hud .v')
    expect(cells[0].textContent).toBe('1200')
    expect(cells[1].textContent).toBe('♥♥♥')
    expect(cells[2].textContent).toBe('3')
  })

  it('renders the controls hint in the footer', async () => {
    const el = mount({ controls: 'Arrows to move' })
    await flush()
    expect(shadow(el).querySelector('.foot .hint')?.textContent).toBe('Arrows to move')
  })

  it('exposes the default slot for the game canvas', async () => {
    const el = mount({}, '<div class="game">stage</div>')
    await flush()
    const slot = shadow(el).querySelector('.canvas slot') as HTMLSlotElement
    expect(slot).not.toBeNull()
    const assigned = slot.assignedNodes().map((n) => (n as HTMLElement).className)
    expect(assigned).toContain('game')
  })

  it('emits ship-action with the matching action for each title-bar button', async () => {
    const el = mount({ name: 'Snake' })
    await flush()
    const onAction = vi.fn()
    el.addEventListener('ship-action', onAction)
    const btn = (label: string) =>
      shadow(el).querySelector(`button[aria-label="${label}"]`) as HTMLElement
    btn('Back').click()
    btn('Pause').click()
    btn('Restart').click()
    btn('Close').click()
    expect(onAction.mock.calls.map((c) => (c[0] as CustomEvent).detail.action)).toEqual([
      'back', 'pause', 'restart', 'close',
    ])
  })

  it('bubbles ship-action through composed shadow boundaries', async () => {
    const el = mount()
    await flush()
    const onAction = vi.fn()
    document.body.addEventListener('ship-action', onAction)
    ;(shadow(el).querySelector('button[aria-label="Pause"]') as HTMLElement).click()
    document.body.removeEventListener('ship-action', onAction)
    expect(onAction).toHaveBeenCalledTimes(1)
    expect((onAction.mock.calls[0][0] as CustomEvent).detail).toEqual({ action: 'pause' })
  })
})
