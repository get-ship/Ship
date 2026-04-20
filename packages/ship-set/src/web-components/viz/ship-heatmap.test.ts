import { afterEach, describe, expect, it } from 'vitest'
import './ship-heatmap'
import type { ShipHeatmap } from './ship-heatmap'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipHeatmap
  await el.updateComplete
  return el
}

describe('ship-heatmap', () => {
  afterEach(cleanupTestDom)

  it('renders rows × cols cells', async () => {
    const el = await mount(
      `<ship-heatmap rows="a,b" cols="x,y,z" data='[{"x":"x","y":"a","value":0.5}]'></ship-heatmap>`,
    )
    expect(el.shadowRoot!.querySelectorAll('.cell').length).toBe(6)
  })

  it('applies color-mix style for matching cell', async () => {
    const el = await mount(
      `<ship-heatmap rows="a" cols="x" data='[{"x":"x","y":"a","value":0.8}]'></ship-heatmap>`,
    )
    const cell = el.shadowRoot!.querySelector('.cell') as HTMLElement
    expect(cell.getAttribute('style') ?? '').toContain('color-mix')
  })

  it('reacts to data changes', async () => {
    const el = await mount(
      `<ship-heatmap rows="a" cols="x,y" data='[]'></ship-heatmap>`,
    )
    el.setAttribute('data', '[{"x":"x","y":"a","value":1}]')
    await el.updateComplete
    const cells = el.shadowRoot!.querySelectorAll('.cell')
    expect(cells[0].getAttribute('style') ?? '').toContain('color-mix')
    expect(cells[1].getAttribute('style') ?? '').toBe('')
  })
})
