import { describe, expect, it, beforeAll, vi } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-scroll-area (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-scroll-area.set')
  }, 120_000)

  function mount(attrs = '', body = '<div style="height:400px">tall</div>') {
    const wrap = document.createElement('div')
    wrap.innerHTML = `<ship-scroll-area ${attrs} style="height:100px;display:block">${body}</ship-scroll-area>`
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }

  it('renders a viewport with tabindex=0 wrapping slotted content', () => {
    const el = mount('', '<div class="probe">child</div>')
    const viewport = el.querySelector('.viewport') as HTMLElement
    expect(viewport).not.toBeNull()
    expect(viewport.getAttribute('tabindex')).toBe('0')
    expect(viewport.getAttribute('part')).toBe('viewport')
    expect(viewport.querySelector('.probe')).not.toBeNull()
    el.remove()
  })

  it('defaults to vertical orientation: only the v-bar is rendered', () => {
    const el = mount()
    expect(el.hasAttribute('orientation')).toBe(false)
    expect(el.querySelector('.scrollbar-v')).not.toBeNull()
    expect(el.querySelector('.scrollbar-h')).toBeNull()
    el.remove()
  })

  it('orientation=horizontal renders only the h-bar', () => {
    const el = mount('orientation="horizontal"')
    expect(el.getAttribute('orientation')).toBe('horizontal')
    expect(el.querySelector('.scrollbar-v')).toBeNull()
    expect(el.querySelector('.scrollbar-h')).not.toBeNull()
    el.remove()
  })

  it('orientation=both renders both bars', () => {
    const el = mount('orientation="both"')
    expect(el.getAttribute('orientation')).toBe('both')
    expect(el.querySelector('.scrollbar-v')).not.toBeNull()
    expect(el.querySelector('.scrollbar-h')).not.toBeNull()
    el.remove()
  })

  it('size attribute reflects on the host', () => {
    const el = mount('size="sm"')
    expect(el.getAttribute('size')).toBe('sm')
    el.remove()
  })

  it('scrollbar tracks carry aria-hidden and the part attribute', () => {
    const el = mount('orientation="both"')
    const v = el.querySelector('.scrollbar-v') as HTMLElement
    const h = el.querySelector('.scrollbar-h') as HTMLElement
    expect(v.getAttribute('aria-hidden')).toBe('true')
    expect(v.getAttribute('part')).toBe('scrollbar-v')
    expect(h.getAttribute('aria-hidden')).toBe('true')
    expect(h.getAttribute('part')).toBe('scrollbar-h')
    expect(v.querySelector('.thumb')).not.toBeNull()
    expect(h.querySelector('.thumb')).not.toBeNull()
    el.remove()
  })

  it('flips bar visibility when orientation attribute changes', () => {
    const el = mount('orientation="vertical"')
    expect(el.querySelector('.scrollbar-h')).toBeNull()
    el.setAttribute('orientation', 'both')
    expect(el.querySelector('.scrollbar-v')).not.toBeNull()
    expect(el.querySelector('.scrollbar-h')).not.toBeNull()
    el.setAttribute('orientation', 'horizontal')
    expect(el.querySelector('.scrollbar-v')).toBeNull()
    expect(el.querySelector('.scrollbar-h')).not.toBeNull()
    el.remove()
  })

  it('scrolling marks the bar .active and clears it after 1200ms', () => {
    vi.useFakeTimers()
    try {
      const el = mount('orientation="vertical"')
      const viewport = el.querySelector('.viewport') as HTMLElement
      const vBar = el.querySelector('.scrollbar-v') as HTMLElement
      // jsdom doesn't dispatch a real scroll event for programmatic
      // scrollTop changes — fire one explicitly so the listener runs.
      viewport.dispatchEvent(new Event('scroll'))
      expect(vBar.classList.contains('active')).toBe(true)
      vi.advanceTimersByTime(1200)
      expect(vBar.classList.contains('active')).toBe(false)
      el.remove()
    } finally {
      vi.useRealTimers()
    }
  })

  it('does not declare formAssociated', () => {
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).not.toBe(true)
  })
})
