import { afterEach, describe, expect, it } from 'vitest'
import './ship-stepper'
import type { ShipStepper } from './ship-stepper'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipStepper
  await el.updateComplete
  return el
}

describe('ship-stepper', () => {
  afterEach(cleanupTestDom)

  it('renders `total` dots', async () => {
    const el = await mount('<ship-stepper total="5" current="1"></ship-stepper>')
    const dots = el.shadowRoot!.querySelectorAll('.dot')
    expect(dots.length).toBe(5)
  })

  it('marks the current dot as active and prior dots as done', async () => {
    const el = await mount('<ship-stepper total="4" current="3"></ship-stepper>')
    const dots = Array.from(el.shadowRoot!.querySelectorAll('.dot'))
    const states = dots.map((d) => d.getAttribute('data-state'))
    expect(states).toEqual(['done', 'done', 'active', 'pending'])
  })

  it('sets progressbar ARIA attributes', async () => {
    const el = await mount('<ship-stepper total="4" current="2" label="Setup"></ship-stepper>')
    expect(el.getAttribute('role')).toBe('progressbar')
    expect(el.getAttribute('aria-valuemin')).toBe('1')
    expect(el.getAttribute('aria-valuemax')).toBe('4')
    expect(el.getAttribute('aria-valuenow')).toBe('2')
    expect(el.getAttribute('aria-label')).toBe('Setup')
  })

  it('clamps out-of-range current into [1, total]', async () => {
    const over = await mount('<ship-stepper total="3" current="99"></ship-stepper>')
    const overStates = Array.from(over.shadowRoot!.querySelectorAll('.dot'))
      .map((d) => d.getAttribute('data-state'))
    expect(overStates).toEqual(['done', 'done', 'active'])
    expect(over.getAttribute('aria-valuenow')).toBe('3')

    const under = await mount('<ship-stepper total="3" current="0"></ship-stepper>')
    const underStates = Array.from(under.shadowRoot!.querySelectorAll('.dot'))
      .map((d) => d.getAttribute('data-state'))
    expect(underStates).toEqual(['active', 'pending', 'pending'])
    expect(under.getAttribute('aria-valuenow')).toBe('1')
  })

  it('derives a default aria-label when none is given', async () => {
    const el = await mount('<ship-stepper total="4" current="2"></ship-stepper>')
    expect(el.getAttribute('aria-label')).toBe('Step 2 of 4')
  })

  it('re-renders when `current` attribute changes', async () => {
    const el = await mount('<ship-stepper total="4" current="1"></ship-stepper>')
    el.setAttribute('current', '4')
    await el.updateComplete
    const states = Array.from(el.shadowRoot!.querySelectorAll('.dot'))
      .map((d) => d.getAttribute('data-state'))
    expect(states).toEqual(['done', 'done', 'done', 'active'])
  })
})
