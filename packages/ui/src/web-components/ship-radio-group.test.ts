import { afterEach, describe, expect, it } from 'vitest'
import './ship-radio-group'
import type { ShipRadio, ShipRadioGroup } from './ship-radio-group'
import { cleanupTestDom } from './util'

async function mount(attrs = '') {
  document.body.insertAdjacentHTML('beforeend', `
    <ship-radio-group ${attrs}>
      <ship-radio value="a">Option A</ship-radio>
      <ship-radio value="b">Option B</ship-radio>
      <ship-radio value="c" disabled>Option C</ship-radio>
    </ship-radio-group>
  `)
  const el = document.body.lastElementChild as ShipRadioGroup
  await el.updateComplete
  const radios = el.querySelectorAll('ship-radio')
  for (const r of radios) await (r as ShipRadio).updateComplete
  return el
}

describe('ship-radio-group', () => {
  afterEach(cleanupTestDom)

  it('renders radios and sets role=radiogroup', async () => {
    const el = await mount()
    expect(el.getAttribute('role')).toBe('radiogroup')
    const radios = el.querySelectorAll('ship-radio')
    expect(radios.length).toBe(3)
  })

  it('selects radio matching value attribute', async () => {
    const el = await mount('value="b"')
    const radios = el.querySelectorAll('ship-radio') as NodeListOf<ShipRadio>
    expect(radios[0].checked).toBe(false)
    expect(radios[1].checked).toBe(true)
  })

  it('clicking a radio selects it and emits ship-change', async () => {
    const el = await mount()
    let detail: { value: string } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })

    const radioA = el.querySelector('ship-radio[value="a"]') as ShipRadio
    const input = radioA.shadowRoot!.querySelector('input') as HTMLInputElement
    input.click()
    await el.updateComplete
    await radioA.updateComplete

    expect(el.value).toBe('a')
    expect(radioA.checked).toBe(true)
    expect(detail).toEqual({ value: 'a' })
  })

  it('arrow key navigates to next radio', async () => {
    const el = await mount('value="a"')
    const radioA = el.querySelector('ship-radio[value="a"]') as ShipRadio
    const input = radioA.shadowRoot!.querySelector('input') as HTMLInputElement

    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, composed: true,
    }))
    await el.updateComplete

    expect(el.value).toBe('b')
    const radioB = el.querySelector('ship-radio[value="b"]') as ShipRadio
    expect(radioB.checked).toBe(true)
  })

  it('disabled radio is skipped during navigation', async () => {
    const el = await mount('value="b"')
    const radioB = el.querySelector('ship-radio[value="b"]') as ShipRadio
    const input = radioB.shadowRoot!.querySelector('input') as HTMLInputElement

    // ArrowDown from b should skip c (disabled) and wrap to a
    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, composed: true,
    }))
    await el.updateComplete

    expect(el.value).toBe('a')
  })

  it('group disabled prevents selection', async () => {
    const el = await mount('disabled value="a"')
    const radioB = el.querySelector('ship-radio[value="b"]') as ShipRadio
    await radioB.updateComplete
    expect(radioB.disabled).toBe(true)
  })
})
