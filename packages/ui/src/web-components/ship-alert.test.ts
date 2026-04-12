import { afterEach, describe, expect, it } from 'vitest'
import './ship-alert'
import type { ShipAlert } from './ship-alert'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipAlert
  await el.updateComplete
  return el
}

describe('ship-alert', () => {
  afterEach(cleanupTestDom)

  it('renders with default tone=info', async () => {
    const el = await mount('<ship-alert>heads up</ship-alert>')
    expect(el.tone).toBe('info')
  })

  it('dismissible: fires ship-dismiss and removes self', async () => {
    const el = await mount('<ship-alert dismissible>x</ship-alert>')
    let fired = 0
    document.addEventListener('ship-dismiss', () => fired++)
    const close = el.shadowRoot!.querySelector('.close') as HTMLButtonElement
    close.click()
    expect(fired).toBe(1)
    expect(document.querySelector('ship-alert')).toBeNull()
  })
})
