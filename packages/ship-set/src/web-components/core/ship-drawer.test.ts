import { afterEach, describe, expect, it } from 'vitest'
import './ship-drawer'
import type { ShipDrawer } from './ship-drawer'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipDrawer
  await el.updateComplete
  return el
}

describe('ship-drawer', () => {
  afterEach(cleanupTestDom)

  it('renders title/description/default/footer slots', async () => {
    const el = await mount(`
      <ship-drawer>
        <span slot="title">Edit item</span>
        <span slot="description">Editing row abc</span>
        Body content
        <span slot="footer">Footer</span>
      </ship-drawer>
    `)
    const dialog = el.shadowRoot!.querySelector('dialog')
    expect(dialog).toBeTruthy()
    const slots = el.shadowRoot!.querySelectorAll('slot')
    // title, description, default, footer = 4 slots
    expect(slots.length).toBe(4)
  })

  it('starts closed', async () => {
    const el = await mount('<ship-drawer></ship-drawer>')
    expect(el.open).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('open/close methods toggle state', async () => {
    const el = await mount('<ship-drawer></ship-drawer>')
    try { el.show() } catch { /* jsdom */ }
    expect(el.open).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
    el.close()
    expect(el.open).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('open attribute toggles state', async () => {
    const el = await mount('<ship-drawer open></ship-drawer>')
    expect(el.open).toBe(true)
  })

  it('close button emits ship-dismiss and closes', async () => {
    const el = await mount('<ship-drawer open></ship-drawer>')
    const btn = el.shadowRoot!.querySelector('.close') as HTMLButtonElement
    expect(btn).toBeTruthy()
    let dismissed = 0
    el.addEventListener('ship-dismiss', () => dismissed++)
    btn.click()
    expect(dismissed).toBe(1)
    expect(el.open).toBe(false)
  })

  it('close button emits ship-close', async () => {
    const el = await mount('<ship-drawer open></ship-drawer>')
    const btn = el.shadowRoot!.querySelector('.close') as HTMLButtonElement
    let closed = 0
    el.addEventListener('ship-close', () => closed++)
    btn.click()
    expect(closed).toBe(1)
  })

  it('dismissible=false: close button still dismisses', async () => {
    // Close button always works regardless of dismissible
    const el = await mount('<ship-drawer open dismissible="false"></ship-drawer>')
    el.dismissible = false
    await el.updateComplete
    const btn = el.shadowRoot!.querySelector('.close') as HTMLButtonElement
    btn.click()
    expect(el.open).toBe(false)
  })

  it('backdrop click closes when dismissible=true', async () => {
    const el = await mount('<ship-drawer open></ship-drawer>')
    const dialog = el.shadowRoot!.querySelector('dialog') as HTMLDialogElement
    // Simulate backdrop click — target must be the dialog itself.
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dialog, writable: false })
    dialog.dispatchEvent(evt)
    expect(el.open).toBe(false)
  })

  it('backdrop click blocked when dismissible=false', async () => {
    const el = await mount('<ship-drawer open></ship-drawer>')
    el.dismissible = false
    await el.updateComplete
    const dialog = el.shadowRoot!.querySelector('dialog') as HTMLDialogElement
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dialog, writable: false })
    dialog.dispatchEvent(evt)
    // Should remain open because dismissible=false
    expect(el.open).toBe(true)
  })

  it('reflects side and size attributes', async () => {
    const el = await mount('<ship-drawer side="left" size="lg"></ship-drawer>')
    expect(el.side).toBe('left')
    expect(el.size).toBe('lg')
    expect(el.getAttribute('side')).toBe('left')
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('default side is right', async () => {
    const el = await mount('<ship-drawer></ship-drawer>')
    expect(el.side).toBe('right')
  })

  it('default size is md', async () => {
    const el = await mount('<ship-drawer></ship-drawer>')
    expect(el.size).toBe('md')
  })

  it('default dismissible is true', async () => {
    const el = await mount('<ship-drawer></ship-drawer>')
    expect(el.dismissible).toBe(true)
  })

  it('has four slots', async () => {
    const el = await mount('<ship-drawer></ship-drawer>')
    const slots = Array.from(el.shadowRoot!.querySelectorAll('slot'))
    const names = slots.map((s) => s.getAttribute('name'))
    expect(names).toContain('title')
    expect(names).toContain('description')
    expect(names).toContain('footer')
    // Default (unnamed) slot has null name attribute
    expect(names).toContain(null)
  })
})
