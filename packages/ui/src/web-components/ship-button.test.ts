import { afterEach, describe, expect, it } from 'vitest'
import './ship-button'
import type { ShipButton } from './ship-button'
import { cleanupTestDom } from './util'

async function mount(markup: string): Promise<ShipButton> {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipButton
  await el.updateComplete
  return el
}

describe('ship-button', () => {
  afterEach(cleanupTestDom)

  it('renders a native button with the slotted label', async () => {
    const el = await mount('<ship-button>Save</ship-button>')
    const btn = el.shadowRoot!.querySelector('button')!
    expect(btn).toBeTruthy()
    expect(btn.type).toBe('button')
    expect(el.textContent).toBe('Save')
  })

  it('reflects variant, size, tone to host attributes', async () => {
    const el = await mount('<ship-button variant="ghost" size="sm" tone="destructive">x</ship-button>')
    expect(el.getAttribute('variant')).toBe('ghost')
    expect(el.getAttribute('size')).toBe('sm')
    expect(el.getAttribute('tone')).toBe('destructive')
  })

  it('disables the inner button when loading', async () => {
    const el = await mount('<ship-button loading>Saving</ship-button>')
    const btn = el.shadowRoot!.querySelector('button')!
    expect(btn.disabled).toBe(true)
    expect(btn.getAttribute('aria-busy')).toBe('true')
    expect(el.shadowRoot!.querySelector('.spinner')).toBeTruthy()
  })

  it('dispatches native click through the host', async () => {
    const el = await mount('<ship-button>Go</ship-button>')
    let fired = 0
    el.addEventListener('click', () => fired++)
    el.shadowRoot!.querySelector('button')!.click()
    expect(fired).toBe(1)
  })

  it('does not click when disabled', async () => {
    const el = await mount('<ship-button disabled>Go</ship-button>')
    let fired = 0
    el.addEventListener('click', () => fired++)
    el.shadowRoot!.querySelector('button')!.click()
    expect(fired).toBe(0)
  })
})
