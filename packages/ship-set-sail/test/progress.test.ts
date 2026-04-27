import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-progress (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-progress.sail')
  }, 120_000)

  it('determinate: bar width follows value/max', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '50')
    el.setAttribute('max', '200')
    document.body.appendChild(el)
    const bar = el.querySelector('.bar') as HTMLElement
    // 50/200 → 25%
    expect(bar.style.width).toBe('25%')
    el.remove()
  })

  it('clamps value to [0, 100%]', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '500') // over max
    el.setAttribute('max', '100')
    document.body.appendChild(el)
    const bar = el.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('100%')

    el.setAttribute('value', '-50')
    expect(bar.style.width).toBe('0%')
    el.remove()
  })

  it('indeterminate (no value attr) leaves bar without inline width', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const bar = el.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('')
    // ARIA reflects busy state.
    const track = el.querySelector('.track')!
    expect(track.getAttribute('aria-busy')).toBe('true')
    el.remove()
  })

  it('value change updates bar reactively', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '10')
    document.body.appendChild(el)
    const bar = el.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('10%')
    el.setAttribute('value', '75')
    expect(bar.style.width).toBe('75%')
    el.remove()
  })

  it('aria-valuenow tracks value; aria-valuemax tracks max', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '40')
    el.setAttribute('max', '80')
    document.body.appendChild(el)
    const track = el.querySelector('.track')!
    expect(track.getAttribute('aria-valuenow')).toBe('40')
    expect(track.getAttribute('aria-valuemax')).toBe('80')
    expect(track.getAttribute('aria-busy')).toBe('false')
    el.remove()
  })

  it('label flows to aria-label', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Loading project')
    document.body.appendChild(el)
    const track = el.querySelector('.track')!
    expect(track.getAttribute('aria-label')).toBe('Loading project')
    el.remove()
  })

  it('size attribute is reflected on host for [size=sm] style hook', () => {
    const el = document.createElement(tag)
    el.setAttribute('size', 'sm')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('sm')
    el.remove()
  })
})
