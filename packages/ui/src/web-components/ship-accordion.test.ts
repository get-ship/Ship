import { afterEach, describe, expect, it } from 'vitest'
import './ship-accordion'
import type { ShipAccordion, ShipAccordionItem } from './ship-accordion'
import { cleanupTestDom } from './util'

async function mount(attrs = '') {
  document.body.insertAdjacentHTML('beforeend', `
    <ship-accordion ${attrs}>
      <ship-accordion-item value="a">
        <span slot="trigger">Item A</span>
        Content A
      </ship-accordion-item>
      <ship-accordion-item value="b">
        <span slot="trigger">Item B</span>
        Content B
      </ship-accordion-item>
      <ship-accordion-item value="c" disabled>
        <span slot="trigger">Item C</span>
        Content C
      </ship-accordion-item>
    </ship-accordion>
  `)
  const el = document.body.lastElementChild as ShipAccordion
  await el.updateComplete
  // Wait for items to render
  const items = el.querySelectorAll('ship-accordion-item')
  for (const item of items) {
    await (item as ShipAccordionItem).updateComplete
  }
  return el
}

describe('ship-accordion', () => {
  afterEach(cleanupTestDom)

  it('renders all items closed by default', async () => {
    const el = await mount()
    const items = el.querySelectorAll('ship-accordion-item') as NodeListOf<ShipAccordionItem>
    expect(items.length).toBe(3)
    for (const item of items) {
      expect(item.open).toBe(false)
    }
  })

  it('opens item via value attribute', async () => {
    const el = await mount('value="a"')
    const items = el.querySelectorAll('ship-accordion-item') as NodeListOf<ShipAccordionItem>
    expect(items[0].open).toBe(true)
    expect(items[1].open).toBe(false)
  })

  it('single mode: clicking opens one item, closes previous', async () => {
    const el = await mount()
    let detail: { value: string } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })

    const itemA = el.querySelector('ship-accordion-item[value="a"]') as ShipAccordionItem
    const triggerA = itemA.shadowRoot!.querySelector('button') as HTMLButtonElement
    triggerA.click()
    await el.updateComplete
    await itemA.updateComplete

    expect(itemA.open).toBe(true)
    expect(detail).toEqual({ value: 'a' })

    // Click second item
    const itemB = el.querySelector('ship-accordion-item[value="b"]') as ShipAccordionItem
    const triggerB = itemB.shadowRoot!.querySelector('button') as HTMLButtonElement
    triggerB.click()
    await el.updateComplete
    await itemA.updateComplete
    await itemB.updateComplete

    expect(itemA.open).toBe(false)
    expect(itemB.open).toBe(true)
    expect(detail).toEqual({ value: 'b' })
  })

  it('multiple mode: allows several items open', async () => {
    const el = await mount('type="multiple"')

    const itemA = el.querySelector('ship-accordion-item[value="a"]') as ShipAccordionItem
    const triggerA = itemA.shadowRoot!.querySelector('button') as HTMLButtonElement
    triggerA.click()
    await el.updateComplete
    await itemA.updateComplete

    const itemB = el.querySelector('ship-accordion-item[value="b"]') as ShipAccordionItem
    const triggerB = itemB.shadowRoot!.querySelector('button') as HTMLButtonElement
    triggerB.click()
    await el.updateComplete
    await itemB.updateComplete

    expect(itemA.open).toBe(true)
    expect(itemB.open).toBe(true)
  })

  it('disabled item does not toggle', async () => {
    const el = await mount()
    const itemC = el.querySelector('ship-accordion-item[value="c"]') as ShipAccordionItem
    const triggerC = itemC.shadowRoot!.querySelector('button') as HTMLButtonElement
    triggerC.click()
    await el.updateComplete
    await itemC.updateComplete
    expect(itemC.open).toBe(false)
  })

  it('item has correct ARIA attributes', async () => {
    const el = await mount('value="a"')
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as ShipAccordionItem
    await itemA.updateComplete
    const trigger = itemA.shadowRoot!.querySelector('button')!
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    expect(trigger.getAttribute('aria-controls')).toBeTruthy()
    const region = itemA.shadowRoot!.querySelector('[role="region"]')!
    expect(region.getAttribute('aria-labelledby')).toBe(trigger.id)
  })
})
