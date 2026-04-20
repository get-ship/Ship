import { afterEach, describe, expect, it } from 'vitest'
import './ship-toggle-group'
import type { ShipToggle, ShipToggleGroup } from './ship-toggle-group'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipToggle | ShipToggleGroup
  await el.updateComplete
  return el
}

describe('ship-toggle', () => {
  afterEach(cleanupTestDom)

  it('renders with aria-pressed false by default', async () => {
    const el = await mount(`<ship-toggle value="bold">B</ship-toggle>`) as ShipToggle
    const btn = el.renderRoot.querySelector('button')!
    expect(btn.getAttribute('aria-pressed')).toBe('false')
    expect(el.pressed).toBe(false)
  })

  it('toggles pressed on click and fires ship-toggle', async () => {
    const el = await mount(`<ship-toggle value="bold">B</ship-toggle>`) as ShipToggle
    let detail: { pressed: boolean; value: string } | null = null
    el.addEventListener('ship-toggle', (e) => { detail = (e as CustomEvent).detail })
    el.renderRoot.querySelector('button')!.click()
    expect(el.pressed).toBe(true)
    expect(detail).toEqual({ pressed: true, value: 'bold' })
  })

  it('reflects disabled attribute', async () => {
    const el = await mount(`<ship-toggle disabled value="x">X</ship-toggle>`) as ShipToggle
    expect(el.disabled).toBe(true)
    const btn = el.renderRoot.querySelector('button')!
    expect(btn.disabled).toBe(true)
  })
})

describe('ship-toggle-group', () => {
  afterEach(cleanupTestDom)

  it('sets role="group"', async () => {
    const el = await mount(`
      <ship-toggle-group type="single">
        <ship-toggle value="a">A</ship-toggle>
        <ship-toggle value="b">B</ship-toggle>
      </ship-toggle-group>
    `) as ShipToggleGroup
    expect(el.getAttribute('role')).toBe('group')
  })

  it('single mode: selects one at a time via ship-change', async () => {
    const el = await mount(`
      <ship-toggle-group type="single">
        <ship-toggle value="a">A</ship-toggle>
        <ship-toggle value="b">B</ship-toggle>
      </ship-toggle-group>
    `) as ShipToggleGroup
    // Wait for children
    const toggles = el.querySelectorAll('ship-toggle') as NodeListOf<ShipToggle>
    await toggles[0].updateComplete
    await toggles[1].updateComplete

    let lastValue = ''
    el.addEventListener('ship-change', (e) => { lastValue = (e as CustomEvent).detail.value })

    // Click toggle A
    toggles[0].renderRoot.querySelector('button')!.click()
    expect(lastValue).toBe('a')
    await el.updateComplete
    expect(toggles[0].pressed).toBe(true)
    expect(toggles[1].pressed).toBe(false)

    // Click toggle B — A should deselect
    toggles[1].renderRoot.querySelector('button')!.click()
    expect(lastValue).toBe('b')
    await el.updateComplete
    expect(toggles[0].pressed).toBe(false)
    expect(toggles[1].pressed).toBe(true)
  })

  it('multiple mode: allows multiple selections', async () => {
    const el = await mount(`
      <ship-toggle-group type="multiple">
        <ship-toggle value="a">A</ship-toggle>
        <ship-toggle value="b">B</ship-toggle>
      </ship-toggle-group>
    `) as ShipToggleGroup
    const toggles = el.querySelectorAll('ship-toggle') as NodeListOf<ShipToggle>
    await toggles[0].updateComplete
    await toggles[1].updateComplete

    toggles[0].renderRoot.querySelector('button')!.click()
    await el.updateComplete
    toggles[1].renderRoot.querySelector('button')!.click()
    await el.updateComplete

    expect(el.value).toBe('a,b')
    expect(toggles[0].pressed).toBe(true)
    expect(toggles[1].pressed).toBe(true)
  })
})
