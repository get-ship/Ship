import { afterEach, describe, expect, it } from 'vitest'
import './ship-empty-state'
import type { ShipEmptyState } from './ship-empty-state'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipEmptyState
  await el.updateComplete
  return el
}

describe('ship-empty-state', () => {
  afterEach(cleanupTestDom)

  it('slot-only: renders icon/title/default/action slots', async () => {
    const el = await mount(`
      <ship-empty-state>
        <span slot="icon">◎</span>
        <span slot="title">No runs</span>
        Kick off a pipeline.
        <button slot="action">Start</button>
      </ship-empty-state>
    `)
    const slots = el.shadowRoot!.querySelectorAll('slot')
    expect(slots.length).toBe(4)
  })

  it('reflects variant', async () => {
    const el = await mount('<ship-empty-state variant="connecting">x</ship-empty-state>')
    expect(el.variant).toBe('connecting')
  })
})
