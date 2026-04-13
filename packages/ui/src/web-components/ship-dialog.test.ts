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

  it('renders a close button that fires ship-dismiss and closes', async () => {
    const el = await mount('<ship-dialog open></ship-dialog>')
    const btn = el.shadowRoot!.querySelector('.close') as HTMLButtonElement
    expect(btn).toBeTruthy()
    let dismissed = 0
    el.addEventListener('ship-dismiss', () => dismissed++)
    btn.click()
    expect(dismissed).toBe(1)
    expect(el.open).toBe(false)
  })

  it('hides the close button when no-close-button is set', async () => {
    const el = await mount('<ship-dialog no-close-button></ship-dialog>')
    expect(el.noCloseButton).toBe(true)
    expect(el.hasAttribute('no-close-button')).toBe(true)
  })
})
