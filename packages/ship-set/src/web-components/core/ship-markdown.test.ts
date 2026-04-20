import { afterEach, describe, expect, it } from 'vitest'
import './ship-markdown'
import { ShipMarkdown, renderMarkdown } from './ship-markdown'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipMarkdown
  await el.updateComplete
  return el
}

describe('ship-markdown', () => {
  afterEach(cleanupTestDom)

  it('renders headings, lists, bold, code', () => {
    const out = renderMarkdown('# Title\n\n- one\n- two\n\n**bold** and `code`')
    expect(out).toContain('<h1>Title</h1>')
    expect(out).toContain('<ul><li>one</li><li>two</li></ul>')
    expect(out).toContain('<strong>bold</strong>')
    expect(out).toContain('<code>code</code>')
  })

  it('renders fenced code blocks with lang', () => {
    const out = renderMarkdown('```ts\nconst a = 1\n```')
    expect(out).toContain('<pre data-lang="ts">')
    expect(out).toContain('const a = 1')
  })

  it('escapes HTML in output', () => {
    const out = renderMarkdown('<script>bad</script>')
    expect(out).not.toContain('<script>')
    expect(out).toContain('&lt;script&gt;')
  })

  it('mounts and reflects value', async () => {
    const el = await mount('<ship-markdown value="# hi"></ship-markdown>')
    expect(el.shadowRoot!.querySelector('h1')?.textContent).toBe('hi')
  })
})
