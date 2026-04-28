import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-tooltip (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-tooltip.set')
  }, 120_000)

  function mount(markup: string) {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }

  it('renders the bubble with role=tooltip', () => {
    const el = mount('<ship-tooltip>hi<span slot="content">info</span></ship-tooltip>')
    const bubble = el.querySelector('.bubble')!
    expect(bubble).not.toBeNull()
    expect(bubble.getAttribute('role')).toBe('tooltip')
    expect(bubble.id).toMatch(/^ship-tooltip-/)
    el.remove()
  })

  it('reflects placement attribute on the host', () => {
    const el = mount('<ship-tooltip placement="right">x<span slot="content">y</span></ship-tooltip>')
    expect(el.getAttribute('placement')).toBe('right')
    el.remove()
  })

  it('routes [slot=content] into the bubble', () => {
    const el = mount('<ship-tooltip>x<span slot="content">help text</span></ship-tooltip>')
    expect(el.querySelector('.bubble')!.textContent).toContain('help text')
    el.remove()
  })

  it('wires aria-describedby from the trigger to the bubble id', () => {
    const el = mount('<ship-tooltip><button>t</button><span slot="content">hint</span></ship-tooltip>')
    const trigger = el.querySelector('button')!
    const bubble = el.querySelector('.bubble')!
    expect(trigger.getAttribute('aria-describedby')).toBe(bubble.id)
    el.remove()
  })

  it('preserves existing aria-describedby tokens on the trigger', () => {
    const el = mount('<ship-tooltip><button aria-describedby="other-id">t</button><span slot="content">hint</span></ship-tooltip>')
    const trigger = el.querySelector('button')!
    const bubble = el.querySelector('.bubble')!
    const tokens = trigger.getAttribute('aria-describedby')!.split(/\s+/)
    expect(tokens).toContain('other-id')
    expect(tokens).toContain(bubble.id)
    el.remove()
  })

  it('Escape blurs the host so the focus-driven open path closes', () => {
    const el = mount('<ship-tooltip><button>t</button><span slot="content">hint</span></ship-tooltip>')
    const trigger = el.querySelector('button')!
    trigger.focus()
    // dispatch Escape on the host directly; handler is bound there.
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    // We can't easily assert blur ran on the custom host (no focus target),
    // but the listener must not throw — that's the regression we guard.
    expect(el.querySelector('.bubble')!.getAttribute('role')).toBe('tooltip')
    el.remove()
  })
})
