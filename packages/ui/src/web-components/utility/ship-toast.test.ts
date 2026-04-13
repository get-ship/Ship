import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-toast'
import type { ShipToast, ShipToastContainer } from './ship-toast'
import { cleanupTestDom } from '../util'

async function mount<T extends Element>(markup: string): Promise<T> {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as T & { updateComplete?: Promise<unknown> }
  if (el.updateComplete) await el.updateComplete
  return el as T
}

describe('ship-toast-container', () => {
  afterEach(cleanupTestDom)

  it('defaults to top-right position', async () => {
    const el = await mount<ShipToastContainer>('<ship-toast-container></ship-toast-container>')
    expect(el.position).toBe('top-right')
  })

  it('reflects position attribute', async () => {
    const el = await mount<ShipToastContainer>('<ship-toast-container position="bottom-left"></ship-toast-container>')
    expect(el.getAttribute('position')).toBe('bottom-left')
  })
})

describe('ship-toast', () => {
  afterEach(cleanupTestDom)

  it('renders default slot and close button', async () => {
    const el = await mount<ShipToast>('<ship-toast duration="0">Saved</ship-toast>')
    expect(el.textContent).toContain('Saved')
    expect(el.shadowRoot!.querySelector('.close')).not.toBeNull()
  })

  it('emits ship-dismiss and removes on close click', async () => {
    const el = await mount<ShipToast>('<ship-toast duration="0">x</ship-toast>')
    const spy = vi.fn()
    el.addEventListener('ship-dismiss', spy)
    ;(el.shadowRoot!.querySelector('.close') as HTMLButtonElement).click()
    expect(spy).toHaveBeenCalledTimes(1)
    expect(el.isConnected).toBe(false)
  })

  it('auto-dismisses after duration', async () => {
    vi.useFakeTimers()
    const el = await mount<ShipToast>('<ship-toast duration="1000">tick</ship-toast>')
    const spy = vi.fn()
    el.addEventListener('ship-dismiss', spy)
    vi.advanceTimersByTime(1000)
    expect(spy).toHaveBeenCalledTimes(1)
    vi.useRealTimers()
  })

  it('hides close when dismissible=false', async () => {
    const el = await mount<ShipToast>('<ship-toast duration="0" .dismissible=${false}>x</ship-toast>')
    // Attribute approach
    el.dismissible = false
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('.close')).toBeNull()
  })
})
