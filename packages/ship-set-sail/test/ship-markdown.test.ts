import { describe, it, expect, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-markdown (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-markdown.set')
  }, 120_000)

  it('renders a heading from value attribute', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '# Hello World')
    document.body.appendChild(el)
    expect(el.querySelector('h1')?.textContent?.trim()).toBe('Hello World')
    el.remove()
  })

  it('renders h2 through h6 headings', () => {
    for (let n = 2; n <= 6; n++) {
      const el = document.createElement(tag)
      el.setAttribute('value', `${'#'.repeat(n)} Heading ${n}`)
      document.body.appendChild(el)
      expect(el.querySelector(`h${n}`)).not.toBeNull()
      el.remove()
    }
  })

  it('renders unordered list items', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '- one\n- two\n- three')
    document.body.appendChild(el)
    const ul = el.querySelector('ul')
    expect(ul).not.toBeNull()
    const items = ul!.querySelectorAll('li')
    expect(items.length).toBe(3)
    expect(items[0].textContent).toBe('one')
    expect(items[2].textContent).toBe('three')
    el.remove()
  })

  it('renders ordered list items', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '1. first\n2. second\n3. third')
    document.body.appendChild(el)
    const ol = el.querySelector('ol')
    expect(ol).not.toBeNull()
    expect(ol!.querySelectorAll('li').length).toBe(3)
    el.remove()
  })

  it('renders bold via **text**', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '**bold text**')
    document.body.appendChild(el)
    expect(el.querySelector('strong')).not.toBeNull()
    expect(el.querySelector('strong')!.textContent).toBe('bold text')
    el.remove()
  })

  it('renders inline code via `code`', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'Use `console.log()` here')
    document.body.appendChild(el)
    expect(el.querySelector('code')).not.toBeNull()
    expect(el.querySelector('code')!.textContent).toBe('console.log()')
    el.remove()
  })

  it('renders fenced code block with data-lang attribute', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '```ts\nconst x = 1\n```')
    document.body.appendChild(el)
    const pre = el.querySelector('pre')
    expect(pre).not.toBeNull()
    expect(pre!.getAttribute('data-lang')).toBe('ts')
    expect(pre!.querySelector('code')?.textContent).toBe('const x = 1')
    el.remove()
  })

  it('escapes raw HTML — no script element injected', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '<script>bad</script>')
    document.body.appendChild(el)
    expect(el.querySelector('script')).toBeNull()
    // The literal characters appear as text, not as a DOM element
    expect(el.textContent).toContain('<script>')
    el.remove()
  })

  it('renders blockquote', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '> a quoted line')
    document.body.appendChild(el)
    const bq = el.querySelector('blockquote')
    expect(bq).not.toBeNull()
    expect(bq!.textContent).toContain('a quoted line')
    el.remove()
  })

  it('renders links with rel=noopener noreferrer', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '[Ship](https://ship.dev)')
    document.body.appendChild(el)
    const a = el.querySelector('a')
    expect(a).not.toBeNull()
    expect(a!.getAttribute('href')).toBe('https://ship.dev')
    expect(a!.getAttribute('rel')).toBe('noopener noreferrer')
    expect(a!.textContent).toBe('Ship')
    el.remove()
  })

  it('renders plain text as a paragraph', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'Just a paragraph of text')
    document.body.appendChild(el)
    expect(el.querySelector('p')).not.toBeNull()
    el.remove()
  })

  it('empty value renders no meaningful content', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.textContent?.trim()).toBe('')
    el.remove()
  })

  it('renders mixed markdown — headings, lists, bold, code', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '# Title\n\n- one\n- two\n\n**bold** and `code`')
    document.body.appendChild(el)
    expect(el.querySelector('h1')?.textContent?.trim()).toBe('Title')
    expect(el.querySelector('ul')).not.toBeNull()
    expect(el.querySelector('ul')!.querySelectorAll('li').length).toBe(2)
    expect(el.querySelector('strong')).not.toBeNull()
    expect(el.querySelector('code')).not.toBeNull()
    el.remove()
  })
})
