import { afterEach, describe, expect, it } from 'vitest'
import './ship-collapsible'
import type { ShipCollapsible } from './ship-collapsible'
import { cleanupTestDom } from './util'

async function mount(attrs = '') {
  document.body.insertAdjacentHTML('beforeend', `
    <ship-collapsible ${attrs}>
      <button slot="trigger">Toggle</button>
      <div>Content here</div>
    </ship-collapsible>
  `)
  const el = document.body.lastElementChild as ShipCollapsible
  await el.updateComplete
  return el
}

describe('ship-collapsible', () => {
  afterEach(cleanupTestDom)

  it('renders closed by default', async () => {
    const el = await mount()
    expect(el.open).toBe(false)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('opens when open attribute is set', async () => {
    const el = await mount('open')
    expect(el.open).toBe(true)
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('toggles on trigger click and emits ship-toggle', async () => {
    const el = await mount()
    let detail: { open: boolean } | null = null
    el.addEventListener('ship-toggle', (e) => { detail = (e as CustomEvent).detail })

    const trigger = el.querySelector('[slot=trigger]') as HTMLElement
    trigger.click()
    await el.updateComplete

    expect(el.open).toBe(true)
    expect(detail).toEqual({ open: true })
  })

  it('sets aria-expanded and aria-controls on trigger', async () => {
    const el = await mount()
    await el.updateComplete
    const trigger = el.querySelector('[slot=trigger]') as HTMLElement
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    expect(trigger.getAttribute('aria-controls')).toBeTruthy()
  })

  it('does not toggle when disabled', async () => {
    const el = await mount('disabled')
    const trigger = el.querySelector('[slot=trigger]') as HTMLElement
    trigger.click()
    await el.updateComplete
    expect(el.open).toBe(false)
  })
})
