import { afterEach, describe, expect, it } from 'vitest'
import './ship-popover'
import type { ShipPopover } from './ship-popover'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipPopover
  await el.updateComplete
  return el
}

describe('ship-popover', () => {
  afterEach(cleanupTestDom)

  it('toggle() flips open state', async () => {
    const el = await mount('<ship-popover><button slot="trigger">x</button>body</ship-popover>')
    expect(el.open).toBe(false)
    el.toggle()
    expect(el.open).toBe(true)
    el.toggle()
    expect(el.open).toBe(false)
  })

  it('emits ship-open/ship-close', async () => {
    const el = await mount('<ship-popover>x</ship-popover>')
    let opened = 0, closed = 0
    el.addEventListener('ship-open', () => opened++)
    el.addEventListener('ship-close', () => closed++)
    el.show()
    el.hide()
    expect(opened).toBe(1)
    expect(closed).toBe(1)
  })
})
