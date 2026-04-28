import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-hover-card (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-hover-card.set')
  }, 120_000)

  afterEach(() => {
    vi.useRealTimers()
    document.body.innerHTML = ''
  })

  function mount(markup: string) {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as any
    document.body.appendChild(el)
    return el
  }

  it('renders the card with role=tooltip and a unique id', () => {
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
    const card = el.querySelector('.card')!
    expect(card).not.toBeNull()
    expect(card.getAttribute('role')).toBe('tooltip')
    expect(card.id).toMatch(/^ship-hover-card-/)
  })

  it('starts closed and reflects data-state', () => {
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(el.querySelector('.card')!.getAttribute('aria-hidden')).toBe('true')
  })

  it('show() opens; hide() closes; both reflect data-state and the open attribute', () => {
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
    el.show()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
    expect(el.querySelector('.card')!.getAttribute('aria-hidden')).toBe('false')
    el.hide()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(el.querySelector('.card')!.getAttribute('aria-hidden')).toBe('true')
  })

  it('emits ship-open / ship-close once per show/hide', () => {
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
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
  })

  it('mouseenter shows after showDelay; mouseleave hides after hideDelay', () => {
    vi.useFakeTimers()
    const el = mount('<ship-hover-card show-delay="200" hide-delay="50"><button slot="trigger">t</button>body</ship-hover-card>')
    el.dispatchEvent(new MouseEvent('mouseenter'))
    expect(el.hasAttribute('open')).toBe(false)
    vi.advanceTimersByTime(199)
    expect(el.hasAttribute('open')).toBe(false)
    vi.advanceTimersByTime(1)
    expect(el.hasAttribute('open')).toBe(true)

    el.dispatchEvent(new MouseEvent('mouseleave'))
    expect(el.hasAttribute('open')).toBe(true)
    vi.advanceTimersByTime(49)
    expect(el.hasAttribute('open')).toBe(true)
    vi.advanceTimersByTime(1)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('mouseenter cancels a pending hide timer', () => {
    vi.useFakeTimers()
    const el = mount('<ship-hover-card show-delay="100" hide-delay="100"><button slot="trigger">t</button>body</ship-hover-card>')
    el.show()
    el.dispatchEvent(new MouseEvent('mouseleave')) // schedule hide
    el.dispatchEvent(new MouseEvent('mouseenter')) // cancels hide, schedules show
    vi.advanceTimersByTime(200)
    // mouseenter clears prior timers, then re-schedules show — show() is
    // idempotent when already open, so the host stays open.
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('focusin opens immediately (no delay)', () => {
    vi.useFakeTimers()
    const el = mount('<ship-hover-card show-delay="500"><button slot="trigger">t</button>body</ship-hover-card>')
    el.dispatchEvent(new FocusEvent('focusin'))
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('focusout schedules a hide after hideDelay', () => {
    vi.useFakeTimers()
    const el = mount('<ship-hover-card hide-delay="40"><button slot="trigger">t</button>body</ship-hover-card>')
    el.show()
    el.dispatchEvent(new FocusEvent('focusout'))
    expect(el.hasAttribute('open')).toBe(true)
    vi.advanceTimersByTime(40)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('wires aria-describedby on the slotted trigger', () => {
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    const card = el.querySelector('.card') as HTMLElement
    expect(trigger.getAttribute('aria-describedby')).toBe(card.id)
  })

  it('placement attribute is reflected on the host (drives CSS positioning)', () => {
    for (const p of ['top', 'bottom', 'left', 'right']) {
      const el = mount(`<ship-hover-card placement="${p}"><button slot="trigger">t</button>body</ship-hover-card>`)
      expect(el.getAttribute('placement')).toBe(p)
      expect(el.querySelector('.card')).not.toBeNull()
    }
  })

  it('show-delay / hide-delay are decoded as numbers (default 300 / 100)', () => {
    vi.useFakeTimers()
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
    el.dispatchEvent(new MouseEvent('mouseenter'))
    vi.advanceTimersByTime(299)
    expect(el.hasAttribute('open')).toBe(false)
    vi.advanceTimersByTime(1)
    expect(el.hasAttribute('open')).toBe(true)

    el.dispatchEvent(new MouseEvent('mouseleave'))
    vi.advanceTimersByTime(99)
    expect(el.hasAttribute('open')).toBe(true)
    vi.advanceTimersByTime(1)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('cleanup removes host listeners and clears pending timers on disconnect', () => {
    vi.useFakeTimers()
    const el = mount('<ship-hover-card show-delay="100"><button slot="trigger">t</button>body</ship-hover-card>')
    el.dispatchEvent(new MouseEvent('mouseenter'))
    el.remove()
    // Pending timer must be cleared — advancing real time after removal must
    // not toggle a disposed component (the host attr would also be gone).
    expect(() => vi.advanceTimersByTime(500)).not.toThrow()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('setting the open attribute externally syncs data-state + aria-hidden', () => {
    const el = mount('<ship-hover-card><button slot="trigger">t</button>body</ship-hover-card>')
    el.setAttribute('open', '')
    expect(el.getAttribute('data-state')).toBe('open')
    expect(el.querySelector('.card')!.getAttribute('aria-hidden')).toBe('false')
    el.removeAttribute('open')
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(el.querySelector('.card')!.getAttribute('aria-hidden')).toBe('true')
  })
})
