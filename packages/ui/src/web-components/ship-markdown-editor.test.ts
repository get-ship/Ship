/**
 * Smoke tests for <ship-markdown-editor>.
 *
 * JSDOM can't run the full ProseMirror/TipTap stack reliably, so these
 * tests focus on the component shell: registration, attribute reflection,
 * form-associated surface, toolbar render, teardown. End-to-end TipTap
 * behavior (markdown round-trip, keyboard shortcuts) is verified in the
 * Playwright pass against a real browser — see the demo editor-test page.
 */

import { afterEach, describe, expect, it } from 'vitest'
import './ship-markdown-editor'
import type { ShipMarkdownEditor } from './ship-markdown-editor'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipMarkdownEditor
  await el.updateComplete
  return el
}

describe('ship-markdown-editor', () => {
  afterEach(cleanupTestDom)

  it('registers the custom element', () => {
    expect(customElements.get('ship-markdown-editor')).toBeTruthy()
  })

  it('renders shell + toolbar + editor host', async () => {
    const el = await mount('<ship-markdown-editor placeholder="Write..."></ship-markdown-editor>')
    const root = el.shadowRoot!
    expect(root.querySelector('.shell')).toBeTruthy()
    expect(root.querySelector('.toolbar')).toBeTruthy()
    expect(root.querySelector('[data-editor-host]')).toBeTruthy()
  })

  it('reflects size attribute and applies --ed-min-h from rows', async () => {
    const el = await mount('<ship-markdown-editor size="sm" rows="8"></ship-markdown-editor>')
    expect(el.getAttribute('size')).toBe('sm')
    expect(el.style.getPropertyValue('--ed-min-h')).toBe('224px')
  })

  it('reflects invalid and disabled state to shadow DOM', async () => {
    const el = await mount('<ship-markdown-editor invalid disabled></ship-markdown-editor>')
    const shell = el.shadowRoot!.querySelector('.shell')!
    expect(shell.getAttribute('aria-invalid')).toBe('true')
    expect(shell.getAttribute('aria-disabled')).toBe('true')
    const firstBtn = el.shadowRoot!.querySelector<HTMLButtonElement>('.toolbar button')!
    expect(firstBtn.disabled).toBe(true)
  })

  it('uses the label attribute for toolbar aria-label', async () => {
    const el = await mount('<ship-markdown-editor label="Notes editor"></ship-markdown-editor>')
    const tb = el.shadowRoot!.querySelector('.toolbar')!
    expect(tb.getAttribute('aria-label')).toBe('Notes editor')
  })

  it('is form-associated', () => {
    expect((customElements.get('ship-markdown-editor') as typeof ShipMarkdownEditor).formAssociated).toBe(true)
  })

  it('toolbar exposes the minimum-set buttons with aria-pressed', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    const names = Array.from(el.shadowRoot!.querySelectorAll<HTMLButtonElement>('.toolbar button'))
      .map(b => b.getAttribute('aria-label'))
    for (const expected of ['Bold', 'Italic', 'Inline code', 'Heading 1', 'Heading 2',
      'Heading 3', 'Bullet list', 'Ordered list', 'Task list', 'Link']) {
      expect(names).toContain(expected)
    }
    const bold = el.shadowRoot!.querySelector<HTMLButtonElement>('[part="tb-bold"]')!
    expect(bold.getAttribute('aria-pressed')).toBe('false')
  })

  it('teardown removes the element without throwing', async () => {
    const el = await mount('<ship-markdown-editor value="# Hello"></ship-markdown-editor>')
    expect(() => el.remove()).not.toThrow()
  })
})
