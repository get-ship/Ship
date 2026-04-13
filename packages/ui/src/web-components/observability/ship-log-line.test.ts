import { afterEach, describe, expect, it } from 'vitest'
import './ship-log-line'
import type { ShipLogLine } from './ship-log-line'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipLogLine
  await el.updateComplete
  return el
}

describe('ship-log-line', () => {
  afterEach(cleanupTestDom)

  it('renders slotted message', async () => {
    const el = await mount('<ship-log-line>hello</ship-log-line>')
    expect(el.textContent).toContain('hello')
    expect(el.level).toBe('info')
  })

  it('reflects level attribute', async () => {
    const el = await mount('<ship-log-line level="error">boom</ship-log-line>')
    expect(el.level).toBe('error')
    expect(el.getAttribute('level')).toBe('error')
  })

  it('renders timestamp and source', async () => {
    const el = await mount('<ship-log-line timestamp="2m" source="daemon">msg</ship-log-line>')
    const shadow = el.shadowRoot!.textContent!
    expect(shadow).toContain('2m')
    expect(shadow).toContain('daemon')
  })
})
