import { afterEach, describe, expect, it } from 'vitest'
import './ship-dropdown-menu'
import type { ShipDropdownMenu } from './ship-dropdown-menu'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipDropdownMenu
  await el.updateComplete
  return el
}

describe('ship-dropdown-menu', () => {
  afterEach(cleanupTestDom)

  it('fires ship-select with data-value on item click', async () => {
    const el = await mount(`
      <ship-dropdown-menu open>
        <button data-value="rename">Rename</button>
        <button data-value="delete">Delete</button>
      </ship-dropdown-menu>
    `)
    let detail: { value: string } | null = null
    el.addEventListener('ship-select', (e) => { detail = (e as CustomEvent).detail })
    ;(el.querySelectorAll('button')[1] as HTMLButtonElement).click()
    expect(detail).toEqual({ value: 'delete' })
    expect(el.open).toBe(false)
  })
})
