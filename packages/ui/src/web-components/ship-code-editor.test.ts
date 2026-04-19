import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import './ship-code-editor'
import type { ShipCodeEditor } from './ship-code-editor'
import { cleanupTestDom } from './util'

async function mount(markup: string): Promise<ShipCodeEditor> {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipCodeEditor
  await el.updateComplete
  // Allow firstUpdated async work (CodeMirror init) to settle
  await new Promise((r) => setTimeout(r, 50))
  await el.updateComplete
  return el
}

describe('ship-code-editor', () => {
  afterEach(cleanupTestDom)

  it('registers as a custom element', () => {
    expect(customElements.get('ship-code-editor')).toBeDefined()
  })

  it('renders with initial value', async () => {
    const el = await mount(
      '<ship-code-editor value="hello world"></ship-code-editor>',
    )
    expect(el.value).toBe('hello world')
  })

  it('reflects the language attribute', async () => {
    const el = await mount(
      '<ship-code-editor language="yaml"></ship-code-editor>',
    )
    expect(el.language).toBe('yaml')
  })

  it('reflects the readonly attribute', async () => {
    const el = await mount(
      '<ship-code-editor readonly></ship-code-editor>',
    )
    expect(el.readonly).toBe(true)
  })

  it('reflects the placeholder attribute', async () => {
    const el = await mount(
      '<ship-code-editor placeholder="Type here..."></ship-code-editor>',
    )
    expect(el.placeholder).toBe('Type here...')
  })

  it('reflects the size attribute', async () => {
    const el = await mount(
      '<ship-code-editor size="sm"></ship-code-editor>',
    )
    expect(el.size).toBe('sm')
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('sets value programmatically', async () => {
    const el = await mount('<ship-code-editor></ship-code-editor>')
    el.value = 'new content'
    await el.updateComplete
    expect(el.value).toBe('new content')
  })

  it('has formAssociated set to true', async () => {
    const { ShipCodeEditor } = await import('./ship-code-editor')
    expect(ShipCodeEditor.formAssociated).toBe(true)
  })

  it('has shadow DOM with editor-root div', async () => {
    const el = await mount('<ship-code-editor></ship-code-editor>')
    const root = el.shadowRoot!.querySelector('.editor-root')
    expect(root).toBeTruthy()
  })

  it('applies line-numbers attribute defaulting to true', async () => {
    const el = await mount('<ship-code-editor></ship-code-editor>')
    expect(el['line-numbers']).toBe(true)
  })

  it('defaults theme to auto', async () => {
    const el = await mount('<ship-code-editor></ship-code-editor>')
    expect(el.theme).toBe('auto')
  })
})
