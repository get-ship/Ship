/**
 * Smoke tests for <ship-markdown-editor>.
 *
 * JSDOM can't run the full ProseMirror/TipTap stack reliably, so these
 * tests focus on the component shell: registration, attribute reflection,
 * form-associated surface, toolbar render, teardown, and the new
 * selection-bubble/comment-composer surface. End-to-end TipTap behavior
 * (markdown round-trip, keyboard shortcuts, real selection → bubble) is
 * verified in the Playwright pass — see .ship/views/canvas/docs/editor-test.
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

// Force an internal render so we can inspect the bubble + composer. The
// component refreshes its bubble/composer on selection changes via TipTap,
// which JSDOM can't drive — so we simulate by poking private fields and
// triggering requestUpdate().
async function forceRender(el: ShipMarkdownEditor,
                           patch: Partial<Record<'bubblePos' | 'pending' | 'marks', unknown>>) {
  const anyEl = el as unknown as Record<string, unknown>
  for (const [k, v] of Object.entries(patch)) anyEl[k] = v
  ;(el as unknown as { requestUpdate(): void }).requestUpdate()
  await el.updateComplete
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

  it('selection bubble is hidden by default (no selection)', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    const bubble = el.shadowRoot!.querySelector('.bubble')!
    expect(bubble).toBeTruthy()
    expect(bubble.getAttribute('data-open')).toBe('false')
    expect(bubble.getAttribute('aria-hidden')).toBe('true')
  })

  it('selection bubble becomes open when bubblePos is set', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { bubblePos: { top: 24, left: 120 } })
    const bubble = el.shadowRoot!.querySelector<HTMLElement>('.bubble')!
    expect(bubble.getAttribute('data-open')).toBe('true')
    expect(bubble.style.top).toBe('24px')
    expect(bubble.style.left).toBe('120px')
  })

  it('bubble format buttons render the minimum set (B/I/S/code)', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { bubblePos: { top: 0, left: 0 } })
    const labels = Array.from(el.shadowRoot!.querySelectorAll<HTMLButtonElement>('.bubble button'))
      .map(b => b.getAttribute('aria-label'))
    expect(labels).toEqual(expect.arrayContaining(['Bold', 'Italic', 'Strike', 'Code', 'Comment']))
    expect(labels).not.toContain('Generate')
  })

  it('allow-generate opt-in adds the Generate button', async () => {
    const el = await mount('<ship-markdown-editor allow-generate></ship-markdown-editor>')
    await forceRender(el, { bubblePos: { top: 0, left: 0 } })
    const labels = Array.from(el.shadowRoot!.querySelectorAll<HTMLButtonElement>('.bubble button'))
      .map(b => b.getAttribute('aria-label'))
    expect(labels).toContain('Generate')
  })

  it('allow-comment="false" hides the Comment button', async () => {
    // Lit's boolean-attribute coercion: empty string = true; omit = false.
    // For an explicit off we rely on the property default and don't set the attr.
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    el.allowComment = false
    await forceRender(el, { bubblePos: { top: 0, left: 0 } })
    const labels = Array.from(el.shadowRoot!.querySelectorAll<HTMLButtonElement>('.bubble button'))
      .map(b => b.getAttribute('aria-label'))
    expect(labels).not.toContain('Comment')
  })

  it('show-selection-toolbar=false suppresses the bubble entirely', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    el.showSelectionToolbar = false
    await forceRender(el, { bubblePos: { top: 0, left: 0 } })
    expect(el.shadowRoot!.querySelector('.bubble')).toBeNull()
  })

  it('renders the comment composer when pending is set', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { pending: { text: 'selected words', value: '' } })
    const composer = el.shadowRoot!.querySelector<HTMLElement>('.composer')!
    expect(composer).toBeTruthy()
    expect(composer.getAttribute('role')).toBe('dialog')
    expect(composer.textContent).toContain('selected words')
    const ta = el.shadowRoot!.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
    expect(ta).toBeTruthy()
  })

  it('submit emits ship-feedback with action:comment and both fields', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { pending: { text: 'hello there', value: 'Needs more polish' } })
    const received: unknown[] = []
    el.addEventListener('ship-feedback', (e: Event) => received.push((e as CustomEvent).detail))
    const submit = el.shadowRoot!.querySelector<HTMLButtonElement>('.composer .submit')!
    submit.click()
    expect(received).toEqual([{ action: 'comment', selection: 'hello there', comment: 'Needs more polish' }])
  })

  it('submit is disabled when comment is blank', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { pending: { text: 'x', value: '   ' } })
    const submit = el.shadowRoot!.querySelector<HTMLButtonElement>('.composer .submit')!
    expect(submit.disabled).toBe(true)
  })

  it('Escape in composer cancels; composer disappears', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { pending: { text: 'x', value: '' } })
    const ta = el.shadowRoot!.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
    ta.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('.composer')).toBeNull()
  })

  it('Cmd+Enter in composer submits', async () => {
    const el = await mount('<ship-markdown-editor></ship-markdown-editor>')
    await forceRender(el, { pending: { text: 'snippet', value: 'the note' } })
    const received: unknown[] = []
    el.addEventListener('ship-feedback', (e: Event) => received.push((e as CustomEvent).detail))
    const ta = el.shadowRoot!.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
    ta.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true, bubbles: true }))
    expect(received).toEqual([{ action: 'comment', selection: 'snippet', comment: 'the note' }])
  })
})
