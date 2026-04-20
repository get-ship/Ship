import { afterEach, describe, expect, it } from 'vitest'
import './ship-context-menu'
import type { ShipContextMenu } from './ship-context-menu'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipContextMenu
  await el.updateComplete
  return el
}

describe('ship-context-menu', () => {
  afterEach(cleanupTestDom)

  it('renders and reflects open attribute', async () => {
    const el = await mount(`
      <ship-context-menu>
        <span slot="trigger">Right-click me</span>
        <button data-value="cut">Cut</button>
        <button data-value="copy">Copy</button>
      </ship-context-menu>
    `)
    expect(el.open).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('opens via show() and sets data-state', async () => {
    const el = await mount(`
      <ship-context-menu>
        <span slot="trigger">Trigger</span>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    el.show(100, 200)
    await el.updateComplete
    expect(el.open).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('fires ship-select with data-value on item click', async () => {
    const el = await mount(`
      <ship-context-menu open>
        <button data-value="cut">Cut</button>
        <button data-value="copy">Copy</button>
      </ship-context-menu>
    `)
    let detail: { value: string } | null = null
    el.addEventListener('ship-select', (e) => { detail = (e as CustomEvent).detail })
    ;(el.querySelectorAll('button')[1] as HTMLButtonElement).click()
    expect(detail).toEqual({ value: 'copy' })
    expect(el.open).toBe(false)
  })

  it('closes on Escape key', async () => {
    const el = await mount(`
      <ship-context-menu open>
        <button data-value="x">X</button>
      </ship-context-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(el.open).toBe(false)
  })

  it('assigns role="menuitem" to items', async () => {
    const el = await mount(`
      <ship-context-menu open>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
      </ship-context-menu>
    `)
    await el.updateComplete
    const items = el.querySelectorAll('button')
    expect(items[0].getAttribute('role')).toBe('menuitem')
    expect(items[1].getAttribute('role')).toBe('menuitem')
  })
})
