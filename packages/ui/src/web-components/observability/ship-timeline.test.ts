import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-timeline'
import type { ShipTimelineItem } from './ship-timeline'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  await Promise.resolve()
  return document.body.lastElementChild as HTMLElement
}

describe('ship-timeline', () => {
  afterEach(cleanupTestDom)

  it('renders items', async () => {
    const host = await mount(`
      <ship-timeline>
        <ship-timeline-item at="1m ago" tone="success">
          <span slot="title">Deployed</span>body
        </ship-timeline-item>
      </ship-timeline>
    `)
    const item = host.querySelector('ship-timeline-item') as ShipTimelineItem
    await item.updateComplete
    expect(item.tone).toBe('success')
    expect(item.shadowRoot!.textContent).toContain('1m ago')
  })

  it('emits ship-select on click when interactive', async () => {
    const host = await mount(`
      <ship-timeline><ship-timeline-item interactive></ship-timeline-item></ship-timeline>
    `)
    const item = host.querySelector('ship-timeline-item') as ShipTimelineItem
    await item.updateComplete
    const fn = vi.fn()
    item.addEventListener('ship-select', fn)
    item.click()
    expect(fn).toHaveBeenCalled()
  })

  it('activates on Enter when interactive', async () => {
    const host = await mount(`
      <ship-timeline><ship-timeline-item interactive></ship-timeline-item></ship-timeline>
    `)
    const item = host.querySelector('ship-timeline-item') as ShipTimelineItem
    await item.updateComplete
    const fn = vi.fn()
    item.addEventListener('ship-select', fn)
    item.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    expect(fn).toHaveBeenCalled()
  })
})
