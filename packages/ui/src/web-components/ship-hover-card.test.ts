import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-hover-card'
import type { ShipHoverCard } from './ship-hover-card'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipHoverCard
  await el.updateComplete
  return el
}

describe('ship-hover-card', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    cleanupTestDom()
  })

  it('renders closed by default', async () => {
    const el = await mount(`
      <ship-hover-card>
        <button slot="trigger">Hover me</button>
        <div>Card content</div>
      </ship-hover-card>
    `)
    expect(el.open).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('opens via show() and closes via hide()', async () => {
    const el = await mount(`
      <ship-hover-card>
        <button slot="trigger">Hover</button>
        <div>Content</div>
      </ship-hover-card>
    `)
    const events: string[] = []
    el.addEventListener('ship-open', () => events.push('open'))
    el.addEventListener('ship-close', () => events.push('close'))

    el.show()
    await el.updateComplete
    expect(el.open).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
    expect(events).toContain('open')

    el.hide()
    await el.updateComplete
    expect(el.open).toBe(false)
    expect(events).toContain('close')
  })

  it('reflects placement attribute', async () => {
    const el = await mount(`
      <ship-hover-card placement="top">
        <button slot="trigger">T</button>
        <div>C</div>
      </ship-hover-card>
    `)
    expect(el.placement).toBe('top')
    expect(el.getAttribute('placement')).toBe('top')
  })

  it('sets aria-describedby on trigger', async () => {
    const el = await mount(`
      <ship-hover-card>
        <button slot="trigger">Hover</button>
        <div>Content</div>
      </ship-hover-card>
    `)
    await el.updateComplete
    const trigger = el.querySelector('[slot="trigger"]') as HTMLElement
    const cardId = el.renderRoot.querySelector('.card')?.id
    expect(trigger.getAttribute('aria-describedby')).toBe(cardId)
  })
})
