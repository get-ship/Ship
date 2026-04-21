import { afterEach, describe, expect, it } from 'vitest'
import './ship-segmented'
import type { ShipSegmented } from './ship-segmented'
import { cleanupTestDom } from './util'

async function mount(value = 'rows') {
  document.body.insertAdjacentHTML('beforeend', `
    <ship-segmented value="${value}">
      <ship-segmented-option value="rows">Rows</ship-segmented-option>
      <ship-segmented-option value="schema">Schema</ship-segmented-option>
      <ship-segmented-option value="disabled-opt" disabled>Disabled</ship-segmented-option>
    </ship-segmented>
  `)
  const el = document.body.lastElementChild as ShipSegmented
  await el.updateComplete
  return el
}

describe('ship-segmented', () => {
  afterEach(cleanupTestDom)

  it('renders all option buttons', async () => {
    const el = await mount()
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    expect(btns.length).toBe(3)
  })

  it('aria-checked reflects selected value', async () => {
    const el = await mount('rows')
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    expect(btns[0].getAttribute('aria-checked')).toBe('true')
    expect(btns[1].getAttribute('aria-checked')).toBe('false')
  })

  it('click option updates value and fires ship-change', async () => {
    const el = await mount('rows')
    let detail: { value: string } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    ;(btns[1] as HTMLButtonElement).click()
    await el.updateComplete
    expect(el.value).toBe('schema')
    expect(detail).toEqual({ value: 'schema' })
  })

  it('clicking already-selected option does not re-fire ship-change', async () => {
    const el = await mount('rows')
    let count = 0
    el.addEventListener('ship-change', () => count++)
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    ;(btns[0] as HTMLButtonElement).click()
    await el.updateComplete
    expect(count).toBe(0)
  })

  it('disabled option is blocked', async () => {
    const el = await mount('rows')
    let fired = false
    el.addEventListener('ship-change', () => { fired = true })
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    const disabledBtn = btns[2] as HTMLButtonElement
    // aria-disabled should be set
    expect(disabledBtn.getAttribute('aria-disabled')).toBe('true')
    // clicking disabled button should not fire ship-change
    disabledBtn.click()
    await el.updateComplete
    expect(fired).toBe(false)
  })

  it('host disabled blocks all options', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-segmented value="rows" disabled>
        <ship-segmented-option value="rows">Rows</ship-segmented-option>
        <ship-segmented-option value="schema">Schema</ship-segmented-option>
      </ship-segmented>
    `)
    const el = document.body.lastElementChild as ShipSegmented
    await el.updateComplete
    let fired = false
    el.addEventListener('ship-change', () => { fired = true })
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    ;(btns[1] as HTMLButtonElement).click()
    await el.updateComplete
    expect(fired).toBe(false)
    expect(el.value).toBe('rows')
  })

  it('ArrowRight moves focus and updates value', async () => {
    const el = await mount('rows')
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    const first = btns[0] as HTMLButtonElement
    first.focus()
    first.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await el.updateComplete
    expect(el.value).toBe('schema')
  })

  it('ArrowLeft wraps around to last enabled option', async () => {
    const el = await mount('rows')
    const btns = el.shadowRoot!.querySelectorAll('button[role=radio]')
    const first = btns[0] as HTMLButtonElement
    first.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
    await el.updateComplete
    // disabled option is skipped; wraps to 'schema' (index 1 of enabled)
    expect(el.value).toBe('schema')
  })

  it('host has role=radiogroup', async () => {
    const el = await mount()
    expect(el.getAttribute('role')).toBe('radiogroup')
  })

  it('size=sm attribute reflects', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-segmented value="a" size="sm">
        <ship-segmented-option value="a">A</ship-segmented-option>
      </ship-segmented>
    `)
    const el = document.body.lastElementChild as ShipSegmented
    await el.updateComplete
    expect(el.size).toBe('sm')
    expect(el.getAttribute('size')).toBe('sm')
  })
})
