import { afterEach, describe, expect, it } from 'vitest'
import './ship-error-state'
import type { ShipErrorState } from './ship-error-state'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipErrorState
  await el.updateComplete
  return el
}

describe('ship-error-state', () => {
  afterEach(cleanupTestDom)

  it('renders title attr as default title text', async () => {
    const el = await mount(`<ship-error-state title="Boom"></ship-error-state>`)
    const title = el.shadowRoot!.querySelector('.title')!.textContent!
    expect(title).toContain('Boom')
  })

  it('falls back to "Something went wrong" when no title attr', async () => {
    const el = await mount(`<ship-error-state></ship-error-state>`)
    const title = el.shadowRoot!.querySelector('.title')!.textContent!
    expect(title).toContain('Something went wrong')
  })

  it('toggles details on click when details slot has content', async () => {
    const el = await mount(`
      <ship-error-state title="Oops">
        Try again.
        <pre slot="details">stack trace here</pre>
      </ship-error-state>
    `)
    const toggle = el.shadowRoot!.querySelector('.details-toggle') as HTMLButtonElement
    expect(toggle).toBeTruthy()
    expect(toggle.getAttribute('aria-expanded')).toBe('false')
    toggle.click()
    await el.updateComplete
    expect(toggle.getAttribute('aria-expanded')).toBe('true')
  })

  it('omits the details toggle when no details slot is supplied', async () => {
    const el = await mount(`<ship-error-state title="x"></ship-error-state>`)
    const toggle = el.shadowRoot!.querySelector('.details-toggle')
    expect(toggle).toBeNull()
  })

  it('role=alert for assistive tech', async () => {
    const el = await mount(`<ship-error-state title="x"></ship-error-state>`)
    const box = el.shadowRoot!.querySelector('.box')!
    expect(box.getAttribute('role')).toBe('alert')
  })
})
