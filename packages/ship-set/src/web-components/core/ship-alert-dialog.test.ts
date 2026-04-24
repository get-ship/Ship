import { afterEach, describe, expect, it } from 'vitest'
import './ship-alert-dialog'
import type { ShipAlertDialog } from './ship-alert-dialog'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipAlertDialog
  await el.updateComplete
  await el.updateComplete
  return el
}

describe('ship-alert-dialog', () => {
  afterEach(cleanupTestDom)

  it('renders composed ship-dialog + buttons', async () => {
    const el = await mount('<ship-alert-dialog confirm-label="Delete" tone="destructive"><span slot="title">T</span>body</ship-alert-dialog>')
    const dialog = el.shadowRoot!.querySelector('ship-dialog')
    expect(dialog).toBeTruthy()
  })

  it('emits ship-confirm on confirm click and closes', async () => {
    const el = await mount('<ship-alert-dialog open>x</ship-alert-dialog>')
    let confirmed = 0
    el.addEventListener('ship-confirm', () => confirmed++)
    const buttons = el.shadowRoot!.querySelectorAll('ship-button')
    ;(buttons[1] as HTMLElement).click()
    expect(confirmed).toBe(1)
    expect(el.open).toBe(false)
  })
})
