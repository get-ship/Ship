import { describe, expect, it, beforeAll, beforeEach, vi } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-copy-button (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-copy-button.set')
  }, 120_000)

  beforeEach(() => {
    Object.assign(navigator, {
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    })
  })

  it('renders a button with type=button + aria-label', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'x')
    el.setAttribute('label', 'Copy me')
    document.body.appendChild(el)
    const btn = el.querySelector('button') as HTMLButtonElement
    expect(btn).not.toBeNull()
    expect(btn.getAttribute('type')).toBe('button')
    expect(btn.getAttribute('aria-label')).toBe('Copy me')
    expect(btn.textContent).toContain('Copy me')
    el.remove()
  })

  it('falls back to the default "Copy" label when label attr is unset', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'x')
    document.body.appendChild(el)
    expect(el.querySelector('button')!.textContent).toContain('Copy')
    el.remove()
  })

  it('copies value to the clipboard and emits ship-copy with detail', async () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'hello')
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-copy', (e: any) => events.push(e.detail))
    const btn = el.querySelector('button') as HTMLButtonElement
    btn.click()
    await Promise.resolve()
    await Promise.resolve()
    expect((navigator as any).clipboard.writeText).toHaveBeenCalledWith('hello')
    expect(events).toEqual([{ value: 'hello' }])
    el.remove()
  })

  it('reflects the copied state to a host attribute after a successful copy', async () => {
    vi.useFakeTimers()
    const el = document.createElement(tag)
    el.setAttribute('value', 'x')
    document.body.appendChild(el)
    const btn = el.querySelector('button') as HTMLButtonElement
    btn.click()
    await Promise.resolve()
    await Promise.resolve()
    expect(el.hasAttribute('copied')).toBe(true)
    vi.advanceTimersByTime(1300)
    expect(el.hasAttribute('copied')).toBe(false)
    vi.useRealTimers()
    el.remove()
  })

  it('renders a slotted custom label when supplied', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'x')
    const span = document.createElement('span')
    span.textContent = 'Yank!'
    el.appendChild(span)
    document.body.appendChild(el)
    expect(el.querySelector('button')!.textContent).toContain('Yank!')
    el.remove()
  })
})
