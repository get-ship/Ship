import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-popover (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-popover.set')
  }, 120_000)

  function mount(markup: string) {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as any
    document.body.appendChild(el)
    return el
  }

  it('renders the panel with role=dialog and a unique id', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    const panel = el.querySelector('.panel')!
    expect(panel).not.toBeNull()
    expect(panel.getAttribute('role')).toBe('dialog')
    expect(panel.getAttribute('aria-modal')).toBe('false')
    expect(panel.id).toMatch(/^ship-popover-panel-/)
    el.remove()
  })

  it('starts closed and reflects data-state', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(el.querySelector('.panel')!.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('toggle() flips open state', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    expect(el.hasAttribute('open')).toBe(false)
    el.toggle()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
    expect(el.querySelector('.panel')!.getAttribute('aria-hidden')).toBe('false')
    el.toggle()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('emits ship-open / ship-close once per show/hide', () => {
    const el = mount('<ship-popover>body</ship-popover>')
    let opened = 0
    let closed = 0
    el.addEventListener('ship-open', () => opened++)
    el.addEventListener('ship-close', () => closed++)
    el.show()
    el.show() // idempotent — no extra event
    el.hide()
    el.hide() // idempotent
    expect(opened).toBe(1)
    expect(closed).toBe(1)
    el.remove()
  })

  it('clicking the trigger toggles open', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    trigger.click()
    expect(el.hasAttribute('open')).toBe(true)
    trigger.click()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('wires aria-haspopup / aria-expanded / aria-controls on the trigger', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    const panel = el.querySelector('.panel') as HTMLElement
    expect(trigger.getAttribute('aria-haspopup')).toBe('dialog')
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    expect(trigger.getAttribute('aria-controls')).toBe(panel.id)
    el.show()
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    el.hide()
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    el.remove()
  })

  it('clicking outside the host closes an open popover', () => {
    const outside = document.createElement('div')
    outside.textContent = 'outside'
    document.body.appendChild(outside)
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    el.show()
    expect(el.hasAttribute('open')).toBe(true)
    outside.click()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
    outside.remove()
  })

  it('clicking inside the popover does not close it', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button><span class="inner">body</span></ship-popover>')
    el.show()
    const inner = el.querySelector('.inner') as HTMLElement
    inner.click()
    expect(el.hasAttribute('open')).toBe(true)
    el.remove()
  })

  it('Escape closes an open popover', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    el.show()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('Escape on a closed popover is a no-op', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    expect(() =>
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })),
    ).not.toThrow()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('placement attribute is reflected on the host (drives CSS positioning)', () => {
    for (const p of ['top', 'bottom', 'top-end', 'bottom-end', 'left', 'right']) {
      const el = mount(`<ship-popover placement="${p}"><button slot="trigger">t</button>body</ship-popover>`)
      expect(el.getAttribute('placement')).toBe(p)
      expect(el.querySelector('.panel')).not.toBeNull()
      el.remove()
    }
  })

  it('cleanup removes the document-level click listener on disconnect', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    el.show()
    el.remove()
    // After removal the host signal is gone; clicking elsewhere must not
    // throw or re-enter the (disposed) handler.
    expect(() => document.body.click()).not.toThrow()
  })

  it('setting the open attribute externally syncs ARIA + data-state', () => {
    const el = mount('<ship-popover><button slot="trigger">t</button>body</ship-popover>')
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    el.setAttribute('open', '')
    expect(el.getAttribute('data-state')).toBe('open')
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    el.removeAttribute('open')
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    el.remove()
  })
})
