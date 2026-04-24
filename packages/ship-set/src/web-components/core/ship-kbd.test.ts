import { afterEach, describe, expect, it } from 'vitest'
import './ship-kbd'
import type { ShipKbd } from './ship-kbd'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipKbd
  await el.updateComplete
  return el
}

describe('ship-kbd', () => {
  afterEach(cleanupTestDom)

  it('splits keys on +', async () => {
    const el = await mount('<ship-kbd keys="Cmd+K"></ship-kbd>')
    const kbds = el.shadowRoot!.querySelectorAll('kbd')
    expect(kbds.length).toBe(2)
    expect(kbds[0].textContent).toBe('Cmd')
    expect(kbds[1].textContent).toBe('K')
  })

  it('renders + separator between keys', async () => {
    const el = await mount('<ship-kbd keys="Ctrl+Shift+P"></ship-kbd>')
    expect(el.shadowRoot!.querySelectorAll('.plus').length).toBe(2)
  })

  it('re-renders when keys attribute changes', async () => {
    const el = await mount('<ship-kbd keys="A"></ship-kbd>')
    expect(el.shadowRoot!.querySelectorAll('kbd').length).toBe(1)
    el.keys = 'A+B+C'
    await el.updateComplete
    expect(el.shadowRoot!.querySelectorAll('kbd').length).toBe(3)
  })
})
