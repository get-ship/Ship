import { afterEach, describe, expect, it } from 'vitest'
import './ship-step-segments'
import type { ShipStepSegments } from './ship-step-segments'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipStepSegments
  await el.updateComplete
  return el
}

describe('ship-step-segments', () => {
  afterEach(cleanupTestDom)

  it('renders one segment per comma-separated value', async () => {
    const el = await mount('<ship-step-segments values="done,done,pending"></ship-step-segments>')
    const segs = el.shadowRoot!.querySelectorAll('.seg')
    expect(segs.length).toBe(3)
  })

  it('applies state class from token', async () => {
    const el = await mount('<ship-step-segments values="done,failed,running"></ship-step-segments>')
    const segs = el.shadowRoot!.querySelectorAll('.seg')
    expect(segs[0].classList.contains('done')).toBe(true)
    expect(segs[1].classList.contains('failed')).toBe(true)
    expect(segs[2].classList.contains('running')).toBe(true)
  })

  it('coerces unknown tokens to pending', async () => {
    const el = await mount('<ship-step-segments values="done,wat,banana"></ship-step-segments>')
    const segs = el.shadowRoot!.querySelectorAll('.seg')
    expect(segs.length).toBe(3)
    expect(segs[1].getAttribute('data-state')).toBe('pending')
    expect(segs[2].getAttribute('data-state')).toBe('pending')
  })

  it('renders nothing when values is empty', async () => {
    const el = await mount('<ship-step-segments></ship-step-segments>')
    expect(el.shadowRoot!.querySelectorAll('.seg').length).toBe(0)
  })

  it('ignores empty entries and whitespace', async () => {
    const el = await mount('<ship-step-segments values=" done , , failed "></ship-step-segments>')
    const segs = el.shadowRoot!.querySelectorAll('.seg')
    expect(segs.length).toBe(2)
    expect(segs[0].classList.contains('done')).toBe(true)
    expect(segs[1].classList.contains('failed')).toBe(true)
  })

  it('reflects size attribute', async () => {
    const el = await mount('<ship-step-segments values="done" size="md"></ship-step-segments>')
    expect(el.getAttribute('size')).toBe('md')
  })
})
