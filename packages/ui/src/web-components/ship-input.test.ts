import { afterEach, describe, expect, it } from 'vitest'
import './ship-input'
import type { ShipInput } from './ship-input'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipInput
  await el.updateComplete
  return el
}

describe('ship-input', () => {
  afterEach(cleanupTestDom)

  it('renders a native input with placeholder', async () => {
    const el = await mount('<ship-input placeholder="Name"></ship-input>')
    const input = el.shadowRoot!.querySelector('input')!
    expect(input.placeholder).toBe('Name')
    expect(input.type).toBe('text')
  })

  it('reflects value changes through ship-input event', async () => {
    const el = await mount('<ship-input></ship-input>')
    let detail: { value: string } | null = null
    el.addEventListener('ship-input', (e) => {
      detail = (e as CustomEvent).detail
    })
    const input = el.shadowRoot!.querySelector('input')!
    input.value = 'hello'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    expect(detail).toEqual({ value: 'hello' })
    expect(el.value).toBe('hello')
  })

  it('applies invalid state to aria-invalid', async () => {
    const el = await mount('<ship-input invalid></ship-input>')
    expect(el.shadowRoot!.querySelector('input')!.getAttribute('aria-invalid')).toBe('true')
  })

  it('renders inline spinner in loading state, replacing trailing slot', async () => {
    const el = await mount('<ship-input loading></ship-input>')
    expect(el.loading).toBe(true)
    expect(el.shadowRoot!.querySelector('.spinner')).toBeTruthy()
    expect(el.shadowRoot!.querySelector('slot[name="trailing"]')).toBeNull()
  })
})
