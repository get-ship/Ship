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

describe('@sui/chat-elements registration', () => {
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

describe('<sui-message-list>', () => {
  it('renders one <sui-message> per keyed item set via the items prop', async () => {
    const c = mk()
    c.innerHTML = '<sui-message-list></sui-message-list>'
    const el = c.querySelector('sui-message-list')! as HTMLElement & {
      items: unknown[]
    }
    await flush()
    el.items = [
      { id: 'a', role: 'user', content: 'one' },
      { id: 'b', role: 'assistant', content: 'two' },
    ]
    await flush()
    expect(el.shadowRoot!.querySelectorAll('sui-message').length).toBe(2)
  })
})
