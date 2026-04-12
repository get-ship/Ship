import { afterEach, describe, expect, it } from 'vitest'
import './ship-dialog'
import type { ShipDialog } from './ship-dialog'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipDialog
  await el.updateComplete
  return el
}

describe('ship-dialog', () => {
  afterEach(cleanupTestDom)

  it('renders header/default/footer slots', async () => {
    const el = await mount(`
      <ship-dialog>
        <span slot="header">Title</span>
        Body
        <span slot="footer">F</span>
      </ship-dialog>
    `)
    expect(el.shadowRoot!.querySelector('dialog')).toBeTruthy()
    expect(el.shadowRoot!.querySelectorAll('slot').length).toBe(3)
  })

  it('has show() and close() methods that toggle open attr', async () => {
    const el = await mount('<ship-dialog></ship-dialog>')
    expect(el.open).toBe(false)
    // jsdom may not implement showModal(); guard just exercises state.
    try {
      el.show()
    } catch { /* jsdom */ }
    expect(el.open).toBe(true)
    el.close()
    expect(el.open).toBe(false)
  })
})
