import { afterEach, describe, expect, it } from 'vitest'
import './ship-tabs'
import type { ShipTabs, ShipTabPanel } from './ship-tabs'
import { cleanupTestDom } from './util'

async function mount() {
  document.body.insertAdjacentHTML('beforeend', `
    <ship-tabs active="a">
      <ship-tab name="a" label="A" count="3"></ship-tab>
      <ship-tab name="b" label="B"></ship-tab>
      <ship-tab-panel name="a">panel A</ship-tab-panel>
      <ship-tab-panel name="b">panel B</ship-tab-panel>
    </ship-tabs>
  `)
  const el = document.body.lastElementChild as ShipTabs
  await el.updateComplete
  return el
}

describe('ship-tabs', () => {
  afterEach(cleanupTestDom)

  it('renders tab buttons with labels + count chips', async () => {
    const el = await mount()
    const buttons = el.shadowRoot!.querySelectorAll('button[role=tab]')
    expect(buttons.length).toBe(2)
    expect(buttons[0].textContent!.replace(/\s+/g, ' ').trim()).toContain('A')
    expect(buttons[0].querySelector('.count')!.textContent).toBe('3')
  })

  it('sets active panel visible via active attribute', async () => {
    const el = await mount()
    const panelA = el.querySelector('ship-tab-panel[name="a"]') as ShipTabPanel
    const panelB = el.querySelector('ship-tab-panel[name="b"]') as ShipTabPanel
    expect(panelA.active).toBe(true)
    expect(panelB.active).toBe(false)
  })

  it('activates the clicked tab and emits ship-tab-change', async () => {
    const el = await mount()
    let detail: { name: string } | null = null
    el.addEventListener('ship-tab-change', (e) => { detail = (e as CustomEvent).detail })
    const buttons = el.shadowRoot!.querySelectorAll('button[role=tab]')
    ;(buttons[1] as HTMLButtonElement).click()
    await el.updateComplete
    expect(detail).toEqual({ name: 'b' })
    expect(el.active).toBe('b')
    const panelB = el.querySelector('ship-tab-panel[name="b"]') as ShipTabPanel
    expect(panelB.active).toBe(true)
  })

  it('vertical orientation: aria-orientation set, ArrowDown moves to next tab', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-tabs orientation="vertical" active="a">
        <ship-tab name="a" label="A"></ship-tab>
        <ship-tab name="b" label="B"></ship-tab>
      </ship-tabs>
    `)
    const el = document.body.lastElementChild as ShipTabs
    await el.updateComplete
    const tablist = el.shadowRoot!.querySelector('[role=tablist]')!
    expect(tablist.getAttribute('aria-orientation')).toBe('vertical')
    tablist.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    await el.updateComplete
    expect(el.active).toBe('b')
  })

  it('supports segmented variant', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-tabs variant="segmented" active="x">
        <ship-tab name="x" label="X"></ship-tab>
      </ship-tabs>
    `)
    const el = document.body.lastElementChild as ShipTabs
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('.tablist.segmented')).toBeTruthy()
  })
})
