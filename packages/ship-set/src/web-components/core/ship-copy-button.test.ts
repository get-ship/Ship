import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import './ship-copy-button'
import type { ShipCopyButton } from './ship-copy-button'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipCopyButton
  await el.updateComplete
  return el
}

describe('ship-copy-button', () => {
  beforeEach(() => {
    Object.assign(navigator, {
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    })
  })
  afterEach(cleanupTestDom)

  it('renders label slot fallback', async () => {
    const el = await mount('<ship-copy-button value="x" label="Copy me"></ship-copy-button>')
    expect(el.shadowRoot!.textContent).toContain('Copy me')
  })

  it('copies value and emits ship-copy', async () => {
    const el = await mount('<ship-copy-button value="hello"></ship-copy-button>')
    const listener = vi.fn()
    el.addEventListener('ship-copy', listener)
    const btn = el.shadowRoot!.querySelector('button') as HTMLButtonElement
    btn.click()
    await new Promise((r) => setTimeout(r, 0))
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('hello')
    expect(listener).toHaveBeenCalled()
    expect((listener.mock.calls[0][0] as CustomEvent).detail).toEqual({ value: 'hello' })
    expect(el.copied).toBe(true)
  })

  it('resets copied state after timeout', async () => {
    vi.useFakeTimers()
    const el = await mount('<ship-copy-button value="x"></ship-copy-button>')
    const btn = el.shadowRoot!.querySelector('button') as HTMLButtonElement
    btn.click()
    await Promise.resolve()
    await Promise.resolve()
    expect(el.copied).toBe(true)
    vi.advanceTimersByTime(1300)
    expect(el.copied).toBe(false)
    vi.useRealTimers()
  })
})
