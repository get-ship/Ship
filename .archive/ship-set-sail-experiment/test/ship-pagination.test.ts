import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-pagination (sail)', () => {
  let tag: string
  beforeAll(async () => {
    // ship-button is a transitive dependency rendered inside the bar.
    await loadSailComponent('ship-button.set')
    tag = await loadSailComponent('ship-pagination.set')
  }, 120_000)

  it('renders nav.bar with aria-label=Pagination + two ship-buttons', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const nav = el.querySelector('nav.bar')!
    expect(nav).not.toBeNull()
    expect(nav.getAttribute('aria-label')).toBe('Pagination')
    const btns = el.querySelectorAll('ship-button')
    expect(btns.length).toBe(2)
    el.remove()
  })

  it('default page=1 / total=0 / per-page=25 disables both buttons', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const [prev, next] = Array.from(el.querySelectorAll('ship-button'))
    expect(prev.hasAttribute('disabled')).toBe(true)
    expect(next.hasAttribute('disabled')).toBe(true)
    el.remove()
  })

  it('page=2 of 5 enables both buttons', () => {
    const el = document.createElement(tag)
    el.setAttribute('page', '2')
    el.setAttribute('total', '125')
    el.setAttribute('per-page', '25')
    document.body.appendChild(el)
    const [prev, next] = Array.from(el.querySelectorAll('ship-button'))
    expect(prev.hasAttribute('disabled')).toBe(false)
    expect(next.hasAttribute('disabled')).toBe(false)
    el.remove()
  })

  it('renders "Page N of total" info', () => {
    const el = document.createElement(tag)
    el.setAttribute('page', '2')
    el.setAttribute('total', '50')
    el.setAttribute('per-page', '25')
    document.body.appendChild(el)
    const info = el.querySelector('.info')!
    expect(info.textContent).toContain('Page 2 of 2')
    expect(info.textContent).toContain('26')
    expect(info.textContent).toContain('50')
    el.remove()
  })

  it('emits ship-change { page } when next is clicked', () => {
    const el = document.createElement(tag)
    el.setAttribute('page', '1')
    el.setAttribute('total', '50')
    el.setAttribute('per-page', '25')
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const next = el.querySelectorAll('ship-button')[1] as HTMLElement
    // Trigger the inner <button> click that the wrapper sets up.
    const inner = next.querySelector('button') as HTMLButtonElement
    inner.click()
    expect(events).toEqual([{ page: 2 }])
    el.remove()
  })

  it('does not emit when prev is clicked at page=1', () => {
    const el = document.createElement(tag)
    el.setAttribute('page', '1')
    el.setAttribute('total', '50')
    el.setAttribute('per-page', '25')
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const prev = el.querySelectorAll('ship-button')[0] as HTMLElement
    const inner = prev.querySelector('button') as HTMLButtonElement
    if (!inner.disabled) inner.click()
    expect(events.length).toBe(0)
    el.remove()
  })
})
