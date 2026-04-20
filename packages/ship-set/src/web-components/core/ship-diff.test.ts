import { afterEach, describe, expect, it } from 'vitest'
import './ship-diff'
import type { ShipDiff } from './ship-diff'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipDiff
  await el.updateComplete
  return el
}

describe('ship-diff', () => {
  afterEach(cleanupTestDom)

  it('computes diff from from/to', async () => {
    const el = await mount('<ship-diff></ship-diff>')
    el.from = 'a\nb\nc'
    el.to = 'a\nB\nc'
    await el.updateComplete
    const root = el.shadowRoot!
    expect(root.querySelectorAll('.line.del').length).toBe(1)
    expect(root.querySelectorAll('.line.add').length).toBe(1)
    expect(root.querySelectorAll('.line.ctx').length).toBe(2)
  })

  it('parses unified value', async () => {
    const el = await mount('<ship-diff></ship-diff>')
    el.value = '@@ -1,2 +1,2 @@\n-old\n+new\n ctx'
    await el.updateComplete
    const root = el.shadowRoot!
    expect(root.querySelector('.line.hunk')).toBeTruthy()
    expect(root.querySelector('.line.add')?.textContent).toContain('new')
    expect(root.querySelector('.line.del')?.textContent).toContain('old')
  })

  it('reacts to attribute changes', async () => {
    const el = await mount('<ship-diff from="x" to="y"></ship-diff>')
    expect(el.shadowRoot!.querySelectorAll('.line').length).toBe(2)
  })
})
