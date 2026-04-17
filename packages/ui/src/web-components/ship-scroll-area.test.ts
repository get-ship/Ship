import { afterEach, describe, expect, it } from 'vitest'
import './ship-scroll-area'
import type { ShipScrollArea } from './ship-scroll-area'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipScrollArea
  await el.updateComplete
  return el
}

describe('ship-scroll-area', () => {
  afterEach(cleanupTestDom)

  it('renders with default vertical orientation', async () => {
    const el = await mount(`
      <ship-scroll-area style="height:100px">
        <div style="height:400px">tall content</div>
      </ship-scroll-area>
    `)
    expect(el.orientation).toBe('vertical')
    const viewport = el.renderRoot.querySelector('.viewport') as HTMLElement
    expect(viewport).toBeTruthy()
    expect(viewport.getAttribute('tabindex')).toBe('0')
  })

  it('reflects orientation attribute', async () => {
    const el = await mount(`
      <ship-scroll-area orientation="both">
        <div>content</div>
      </ship-scroll-area>
    `)
    expect(el.orientation).toBe('both')
    expect(el.getAttribute('orientation')).toBe('both')
  })

  it('shows vertical scrollbar track when orientation is vertical', async () => {
    const el = await mount(`
      <ship-scroll-area orientation="vertical">
        <div>content</div>
      </ship-scroll-area>
    `)
    const vBar = el.renderRoot.querySelector('.scrollbar-v')
    const hBar = el.renderRoot.querySelector('.scrollbar-h')
    expect(vBar).toBeTruthy()
    expect(hBar).toBeFalsy()
  })

  it('shows both scrollbar tracks when orientation is both', async () => {
    const el = await mount(`
      <ship-scroll-area orientation="both">
        <div>content</div>
      </ship-scroll-area>
    `)
    const vBar = el.renderRoot.querySelector('.scrollbar-v')
    const hBar = el.renderRoot.querySelector('.scrollbar-h')
    expect(vBar).toBeTruthy()
    expect(hBar).toBeTruthy()
  })

  it('reflects size attribute', async () => {
    const el = await mount(`
      <ship-scroll-area size="sm">
        <div>content</div>
      </ship-scroll-area>
    `)
    expect(el.size).toBe('sm')
    expect(el.getAttribute('size')).toBe('sm')
  })
})
