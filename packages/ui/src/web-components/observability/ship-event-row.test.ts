import { afterEach, describe, expect, it } from 'vitest'
import './ship-event-row'
import type { ShipEventRow } from './ship-event-row'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipEventRow
  await el.updateComplete
  return el
}

describe('ship-event-row', () => {
  afterEach(cleanupTestDom)

  it('renders actor/verb/target/at', async () => {
    const el = await mount(
      '<ship-event-row actor="alice" verb="opened" target="PR #3" at="5m ago"></ship-event-row>',
    )
    const txt = el.shadowRoot!.textContent!
    expect(txt).toContain('alice')
    expect(txt).toContain('opened')
    expect(txt).toContain('PR #3')
    expect(txt).toContain('5m ago')
  })

  it('reacts to attribute changes', async () => {
    const el = await mount('<ship-event-row actor="a"></ship-event-row>')
    el.setAttribute('actor', 'bob')
    await el.updateComplete
    expect(el.shadowRoot!.textContent).toContain('bob')
  })

  it('slots payload content', async () => {
    const el = await mount('<ship-event-row><pre>body</pre></ship-event-row>')
    expect(el.querySelector('pre')?.textContent).toBe('body')
  })
})
