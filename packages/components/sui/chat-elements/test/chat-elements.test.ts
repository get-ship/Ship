// Register-as-custom-element coverage for every tag in the pack, plus
// a few behavioral spot-checks on the controller-backed components.
//
// AOT registration: importing each compiled output calls
// customElements.define() at module load.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-message/sui-message.compiled.js'
import '../src/sui-message-list/sui-message-list.compiled.js'
import '../src/sui-composer/sui-composer.compiled.js'
import '../src/sui-thinking-dots/sui-thinking-dots.compiled.js'
import '../src/sui-tool-call/sui-tool-call.compiled.js'
import '../src/sui-citation/sui-citation.compiled.js'

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})
const flush = () => new Promise((r) => setTimeout(r, 0))

const TAGS = [
  'sui-message',
  'sui-message-list',
  'sui-composer',
  'sui-thinking-dots',
  'sui-tool-call',
  'sui-citation',
] as const

describe('@ship/sui-chat-elements registration', () => {
  for (const tag of TAGS) {
    it(`registers <${tag}> with the global registry`, () => {
      expect(customElements.get(tag)).toBeDefined()
    })

    it(`<${tag}> mounts a shadow root with styles`, () => {
      const c = mk()
      c.innerHTML = `<${tag}></${tag}>`
      const el = c.querySelector(tag)!
      expect(el.shadowRoot).not.toBeNull()
      expect(el.shadowRoot!.querySelector('style')).not.toBeNull()
    })
  }
})

describe('<sui-message>', () => {
  it('reflects role / content-type / status onto the host', () => {
    const c = mk()
    c.innerHTML =
      '<sui-message role="user" content-type="code" status="error">hi</sui-message>'
    const el = c.querySelector('sui-message')!
    expect(el.getAttribute('role')).toBe('user')
    expect(el.getAttribute('content-type')).toBe('code')
    expect(el.getAttribute('status')).toBe('error')
    expect(el.shadowRoot!.querySelector('.bubble')).not.toBeNull()
  })
})

describe('<sui-thinking-dots>', () => {
  it('exposes a polite live region with an accessible label', () => {
    const c = mk()
    c.innerHTML = '<sui-thinking-dots></sui-thinking-dots>'
    const el = c.querySelector('sui-thinking-dots')!
    const dots = el.shadowRoot!.querySelector('.dots')!
    expect(dots.getAttribute('aria-live')).toBe('polite')
    expect(dots.getAttribute('aria-label')).toBe('Assistant is thinking')
    expect(el.shadowRoot!.querySelectorAll('.dot').length).toBe(3)
  })

  it('overrides the announced label via the label prop', () => {
    const c = mk()
    c.innerHTML = '<sui-thinking-dots label="Running tool…"></sui-thinking-dots>'
    const el = c.querySelector('sui-thinking-dots')!
    expect(el.shadowRoot!.querySelector('.dots')!.getAttribute('aria-label')).toBe(
      'Running tool…',
    )
  })

  it('reflects tone + size onto the host so CSS can scale/color the dots', () => {
    const c = mk()
    c.innerHTML = '<sui-thinking-dots tone="primary" size="lg"></sui-thinking-dots>'
    const el = c.querySelector('sui-thinking-dots')!
    expect(el.getAttribute('tone')).toBe('primary')
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('keeps the three dots decorative (aria-hidden), not announced individually', () => {
    const c = mk()
    c.innerHTML = '<sui-thinking-dots></sui-thinking-dots>'
    const el = c.querySelector('sui-thinking-dots')!
    const dots = el.shadowRoot!.querySelectorAll<HTMLElement>('.dot')
    expect(dots.length).toBe(3)
    for (const d of dots) expect(d.getAttribute('aria-hidden')).toBe('true')
  })

  it('appears + disappears as a sibling of the message list (stream lifecycle)', async () => {
    // The thinking indicator is toggled by the consumer alongside the list;
    // assert the element can be added then removed without leaving residue.
    const c = mk()
    c.innerHTML =
      '<sui-message-list></sui-message-list><sui-thinking-dots></sui-thinking-dots>'
    await flush()
    expect(c.querySelector('sui-thinking-dots')).not.toBeNull()
    c.querySelector('sui-thinking-dots')!.remove()
    await flush()
    expect(c.querySelector('sui-thinking-dots')).toBeNull()
  })
})

describe('<sui-tool-call>', () => {
  it('renders the tool name + status text in the header', () => {
    const c = mk()
    c.innerHTML =
      '<sui-tool-call name="read_file" status="running">{"path":"x"}</sui-tool-call>'
    const el = c.querySelector('sui-tool-call')!
    expect(el.shadowRoot!.querySelector('.name')!.textContent).toBe('read_file')
    expect(el.shadowRoot!.querySelector('.status-text')!.textContent).toBe('running')
  })

  it('reflects status onto the host so the dot/edge colors track running→ok→error', async () => {
    const c = mk()
    c.innerHTML = '<sui-tool-call name="grep" status="running"></sui-tool-call>'
    const el = c.querySelector('sui-tool-call')!
    expect(el.getAttribute('status')).toBe('running')
    // Mid-call resolution: the consumer flips the attribute; the header follows.
    el.setAttribute('status', 'ok')
    await flush()
    expect(el.getAttribute('status')).toBe('ok')
    expect(el.shadowRoot!.querySelector('.status-text')!.textContent).toBe('ok')
  })

  it('projects args (default slot) + result (result slot) into monospace bodies', () => {
    const c = mk()
    c.innerHTML =
      '<sui-tool-call name="run" open>' +
      '<span class="a">--flag</span>' +
      '<span slot="result" class="r">done</span>' +
      '</sui-tool-call>'
    const el = c.querySelector('sui-tool-call')!
    // The disclosure body carries an Arguments + Result region; the consumer's
    // light-DOM nodes slot into them.
    const labels = Array.from(
      el.shadowRoot!.querySelectorAll('.body-label'),
    ).map((n) => n.textContent)
    expect(labels).toContain('Arguments')
    expect(labels).toContain('Result')
    expect(el.querySelector('.a')!.textContent).toBe('--flag')
    expect(el.querySelector('.r')!.getAttribute('slot')).toBe('result')
  })

  it('reflects open so the disclosure can start expanded', () => {
    const c = mk()
    c.innerHTML = '<sui-tool-call name="x" open></sui-tool-call>'
    const el = c.querySelector('sui-tool-call')!
    expect(el.hasAttribute('open')).toBe(true)
  })
})

describe('<sui-citation>', () => {
  it('emits sui-citation-open with source + index on click', async () => {
    const c = mk()
    c.innerHTML = '<sui-citation source="docs/ship.md" index="2"></sui-citation>'
    const el = c.querySelector('sui-citation')!
    await flush()
    let detail: { source: string; index: number } | null = null
    el.addEventListener('sui-citation-open', (e) => {
      detail = (e as CustomEvent).detail
    })
    el.shadowRoot!.querySelector<HTMLButtonElement>('.pill')!.click()
    expect(detail).toEqual({ source: 'docs/ship.md', index: 2 })
  })
})

describe('<sui-composer>', () => {
  it('emits sui-composer-submit with trimmed text on Enter', async () => {
    const c = mk()
    c.innerHTML = '<sui-composer></sui-composer>'
    const el = c.querySelector('sui-composer')!
    await flush()
    const field = el.shadowRoot!.querySelector<HTMLInputElement>('.field')!
    let detail: { text: string; attachments: unknown[] } | null = null
    el.addEventListener('sui-composer-submit', (e) => {
      detail = (e as CustomEvent).detail
    })
    field.value = '  hello  '
    field.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    field.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }),
    )
    expect(detail).toEqual({ text: 'hello', attachments: [] })
  })
})

interface MsgItem {
  id: string
  role?: string
  content?: string
  contentType?: string
  timestamp?: string
  status?: string
}
type MessageListEl = HTMLElement & { items: MsgItem[] }

/** Give the scroll viewport a measurable, scrollable geometry. JSDOM reports 0
 *  for all layout, so the controller's pinned/at-bottom check has nothing to
 *  read; stub the metrics it consults. The virtualizer scrolls via
 *  `el.scrollTo({top})` (a JSDOM no-op), so `scrollTop` stays a settable plain
 *  value the consumer / test drives directly. */
function stubScrollGeometry(
  scroll: HTMLElement,
  opts: { scrollHeight: number; clientHeight: number; scrollTop?: number },
): void {
  let top = opts.scrollTop ?? 0
  Object.defineProperty(scroll, 'scrollHeight', {
    configurable: true,
    get: () => opts.scrollHeight,
  })
  Object.defineProperty(scroll, 'clientHeight', {
    configurable: true,
    get: () => opts.clientHeight,
  })
  Object.defineProperty(scroll, 'offsetHeight', {
    configurable: true,
    get: () => opts.clientHeight,
  })
  Object.defineProperty(scroll, 'scrollTop', {
    configurable: true,
    get: () => top,
    set: (v: number) => { top = v },
  })
}

/** Spy on the viewport's `scrollTo` — the channel the virtualizer uses to move
 *  the viewport (JSDOM's native `scrollTo` is an inert no-op, so observing the
 *  CALL is the behavioral signal that auto-scroll fired). Returns the recorded
 *  target `top` offsets. */
function spyScrollTo(scroll: HTMLElement): { tops: number[] } {
  const rec = { tops: [] as number[] }
  Object.defineProperty(scroll, 'scrollTo', {
    configurable: true,
    writable: true,
    value: (arg: number | ScrollToOptions, y?: number) => {
      if (typeof arg === 'object') rec.tops.push(arg.top ?? 0)
      else rec.tops.push(y ?? 0)
    },
  })
  return rec
}

const mountList = async (): Promise<MessageListEl> => {
  const c = mk()
  c.innerHTML = '<sui-message-list></sui-message-list>'
  const el = c.querySelector('sui-message-list')! as MessageListEl
  await flush()
  return el
}

const scrollOf = (el: MessageListEl): HTMLElement =>
  el.shadowRoot!.querySelector<HTMLElement>('.scroll')!

const renderedMessages = (el: MessageListEl): NodeListOf<HTMLElement> =>
  el.shadowRoot!.querySelectorAll<HTMLElement>('sui-message')

describe('<sui-message-list>', () => {
  it('renders one <sui-message> per keyed item set via the items prop', async () => {
    const el = await mountList()
    el.items = [
      { id: 'a', role: 'user', content: 'one' },
      { id: 'b', role: 'assistant', content: 'two' },
    ]
    await flush()
    const msgs = renderedMessages(el)
    expect(msgs.length).toBe(2)
    expect(msgs[0]!.getAttribute('role')).toBe('user')
    expect(msgs[0]!.textContent).toBe('one')
    expect(msgs[1]!.getAttribute('role')).toBe('assistant')
    expect(msgs[1]!.textContent).toBe('two')
  })

  it('windows a long thread: renders only a slice, not every message', async () => {
    const el = await mountList()
    const big: MsgItem[] = Array.from({ length: 5000 }, (_, i) => ({
      id: String(i),
      role: i % 2 ? 'assistant' : 'user',
      content: `msg ${i}`,
    }))
    el.items = big
    await flush()
    const msgs = renderedMessages(el)
    // A window (+overscan), FAR fewer than 5000 DOM bubbles.
    expect(msgs.length).toBeGreaterThan(0)
    expect(msgs.length).toBeLessThan(500)
    // The spacer encodes the FULL thread height so the scrollbar is honest.
    const spacer = el.shadowRoot!.querySelector<HTMLElement>('.spacer')!
    const h = parseFloat(spacer.style.height || '0')
    expect(h).toBeGreaterThan(5000)
  })

  it('windowed rows carry virtual translateY offsets (absolute positioning)', async () => {
    const el = await mountList()
    el.items = Array.from({ length: 2000 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    await flush()
    const rows = el.shadowRoot!.querySelectorAll<HTMLElement>('.row')
    expect(rows.length).toBeGreaterThan(1)
    // Distinct, monotonically increasing translateY offsets — not stacked at 0.
    const offsets = Array.from(rows).map((r) => r.style.transform)
    expect(offsets[0]).toContain('translateY(0px)')
    expect(new Set(offsets).size).toBe(offsets.length)
  })

  it('streams: appending a message incrementally adds one bubble, keeping prior DOM', async () => {
    const el = await mountList()
    el.items = [{ id: 'a', content: 'hello' }]
    await flush()
    expect(renderedMessages(el).length).toBe(1)
    const firstBubble = renderedMessages(el)[0]!
    // Append a streamed turn (consumer replaces the array reference).
    el.items = [...el.items, { id: 'b', content: 'world' }]
    await flush()
    const msgs = renderedMessages(el)
    expect(msgs.length).toBe(2)
    // The keyed window recycles the existing 'a' bubble rather than recreating it.
    expect(msgs[0]).toBe(firstBubble)
    expect(msgs[1]!.textContent).toBe('world')
  })

  it('streams: growing the last message in place updates its content node', async () => {
    const el = await mountList()
    el.items = [{ id: 'a', content: 'The ' }]
    await flush()
    const bubble = renderedMessages(el)[0]!
    expect(bubble.textContent).toBe('The ')
    // Streaming token growth: same id, longer content (new array reference).
    el.items = [{ id: 'a', content: 'The quick brown fox' }]
    await flush()
    // Same DOM bubble (keyed recycle), updated text.
    expect(renderedMessages(el)[0]).toBe(bubble)
    expect(bubble.textContent).toBe('The quick brown fox')
  })

  it('pinned-to-bottom: a new streamed message auto-scrolls toward the newest', async () => {
    const el = await mountList()
    const scroll = scrollOf(el)
    // At the bottom: scrollHeight - scrollTop - clientHeight = 0 ≤ threshold.
    stubScrollGeometry(scroll, { scrollHeight: 1000, clientHeight: 200, scrollTop: 800 })
    el.items = [{ id: 'a', content: 'one' }]
    await flush()
    // Spy from this point on so we observe ONLY the auto-scroll for the append.
    const spy = spyScrollTo(scroll)
    el.items = [...el.items, { id: 'b', content: 'two' }]
    await flush()
    await flush()
    // While pinned, the controller drove the virtualizer to scroll the newest
    // message into view — `scrollTo` fired toward a positive offset.
    expect(spy.tops.length).toBeGreaterThan(0)
    expect(Math.max(...spy.tops)).toBeGreaterThan(0)
  })

  it('scrolled-up: a new message does NOT yank the viewport back to the bottom', async () => {
    const el = await mountList()
    const scroll = scrollOf(el)
    el.items = Array.from({ length: 50 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    await flush()
    // User scrolls up to read history — far from the bottom → un-pins.
    stubScrollGeometry(scroll, { scrollHeight: 4000, clientHeight: 200, scrollTop: 10 })
    scroll.dispatchEvent(new Event('scroll'))
    await flush()
    const spy = spyScrollTo(scroll)
    // A new message streams in while the user is reading up-thread.
    el.items = [...el.items, { id: 'new', content: 'incoming' }]
    await flush()
    await flush()
    // No auto-scroll fired — the viewport is left where the user parked it.
    expect(spy.tops.length).toBe(0)
  })

  it('re-pins when the user scrolls back to the bottom, resuming auto-scroll', async () => {
    const el = await mountList()
    const scroll = scrollOf(el)
    el.items = Array.from({ length: 30 }, (_, i) => ({ id: String(i), content: `m${i}` }))
    await flush()
    // Scroll up (un-pin)…
    stubScrollGeometry(scroll, { scrollHeight: 3000, clientHeight: 200, scrollTop: 5 })
    scroll.dispatchEvent(new Event('scroll'))
    await flush()
    // …then scroll back down to the bottom (re-pin).
    scroll.scrollTop = 2800
    scroll.dispatchEvent(new Event('scroll'))
    await flush()
    const spy = spyScrollTo(scroll)
    el.items = [...el.items, { id: 'tail', content: 'caught up' }]
    await flush()
    await flush()
    // Auto-scroll resumed once re-pinned.
    expect(spy.tops.length).toBeGreaterThan(0)
  })

  it('sticky-bottom=false disables auto-scroll entirely', async () => {
    const c = mk()
    c.innerHTML = '<sui-message-list sticky-bottom="false"></sui-message-list>'
    const el = c.querySelector('sui-message-list')! as MessageListEl
    await flush()
    const scroll = scrollOf(el)
    stubScrollGeometry(scroll, { scrollHeight: 1000, clientHeight: 200, scrollTop: 0 })
    el.items = [{ id: 'a', content: 'one' }]
    await flush()
    const spy = spyScrollTo(scroll)
    el.items = [...el.items, { id: 'b', content: 'two' }]
    await flush()
    await flush()
    // Sticky disabled → the controller never asks the virtualizer to scroll.
    expect(spy.tops.length).toBe(0)
  })

  it('shows the empty slot only while there are no messages', async () => {
    const c = mk()
    c.innerHTML =
      '<sui-message-list><div slot="empty">No messages</div></sui-message-list>'
    const el = c.querySelector('sui-message-list')! as MessageListEl
    await flush()
    expect(el.shadowRoot!.querySelector('slot[name="empty"]')).not.toBeNull()
    el.items = [{ id: 'a', content: 'hi' }]
    await flush()
    expect(el.shadowRoot!.querySelector('slot[name="empty"]')).toBeNull()
  })
})
