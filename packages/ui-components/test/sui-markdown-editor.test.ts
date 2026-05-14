/**
 * JSDOM smoke tests for <sui-markdown-editor>.
 *
 * TipTap/ProseMirror requires a real browser with layout. Under JSDOM the
 * editor engine does not initialize, so these tests drive the component shell
 * via the test-helper setters (_setBubblePos, _setPending, _setMarks,
 * _setComposerValue) exposed by the controller on mount. Playwright covers
 * the full TipTap editing surface.
 */

import { afterEach, describe, expect, it } from 'vitest'
import '../src/islands/sui-markdown-editor/sui-markdown-editor.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

type SuiMDE = HTMLElement & {
  _setBubblePos?: (p: { top: number; left: number } | null) => void
  _setPending?: (p: { text: string } | null) => void
  _setMarks?: (m: Record<string, boolean>) => void
  _setComposerValue?: (v: string) => void
}

function make(attrs: Record<string, string | boolean> = {}) {
  return mount(reg, 'sui-markdown-editor', attrs)
}

function shadow(el: HTMLElement) { return el.shadowRoot! }
function toolbar(el: HTMLElement) { return shadow(el).querySelector<HTMLElement>('.toolbar') }
function bubble(el: HTMLElement) { return shadow(el).querySelector<HTMLElement>('.bubble') }
function composer(el: HTMLElement) { return shadow(el).querySelector<HTMLElement>('.composer') }
function composerInput(el: HTMLElement) { return shadow(el).querySelector<HTMLTextAreaElement>('[data-composer-input]') }
function submitBtn(el: HTMLElement) { return shadow(el).querySelector<HTMLButtonElement>('.composer .submit') }

describe('<sui-markdown-editor>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-markdown-editor')).toBeDefined()
  })

  it('renders shell, toolbar, and editor host in shadow DOM', () => {
    const { el } = make()
    expect(shadow(el).querySelector('.shell')).not.toBeNull()
    expect(toolbar(el)).not.toBeNull()
    expect(shadow(el).querySelector('[part="editor"]')).not.toBeNull()
  })

  it('reflects size attribute on host', () => {
    const { el } = make({ size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('sets --ed-min-h CSS property from rows', async () => {
    const { el } = make({ rows: '8' })
    await flush()
    expect(el.style.getPropertyValue('--ed-min-h')).toBe('224px')
  })

  it('reflects invalid and disabled attributes', () => {
    const { el } = make({ invalid: true, disabled: true })
    expect(el.hasAttribute('invalid')).toBe(true)
    expect(el.hasAttribute('disabled')).toBe(true)
  })

  it('passes aria-invalid and aria-disabled to shell', async () => {
    const { el } = make({ invalid: true, disabled: true })
    await flush()
    const shell = shadow(el).querySelector('.shell')!
    expect(shell.getAttribute('aria-invalid')).toBe('true')
    expect(shell.getAttribute('aria-disabled')).toBe('true')
  })

  it('uses label attribute for toolbar aria-label', async () => {
    const { el } = make({ label: 'Notes editor' })
    await flush()
    expect(toolbar(el)!.getAttribute('aria-label')).toBe('Notes editor')
  })

  it('falls back to "Editor toolbar" when label is absent', async () => {
    const { el } = make()
    await flush()
    expect(toolbar(el)!.getAttribute('aria-label')).toBe('Editor toolbar')
  })

  it('is form-associated (static property)', () => {
    const ctor = customElements.get('sui-markdown-editor') as { formAssociated?: boolean } | undefined
    expect(ctor?.formAssociated).toBe(true)
  })

  it('toolbar contains the minimum-set buttons with aria-pressed', async () => {
    const { el } = make()
    await flush()
    const labels = Array.from(shadow(el).querySelectorAll<HTMLButtonElement>('.toolbar button'))
      .map(b => b.getAttribute('aria-label'))
    for (const expected of ['Bold', 'Italic', 'Inline code', 'Heading 1', 'Heading 2',
      'Heading 3', 'Bullet list', 'Ordered list', 'Task list', 'Link']) {
      expect(labels).toContain(expected)
    }
    const bold = shadow(el).querySelector<HTMLButtonElement>('[part="tb-bold"]')!
    expect(bold.getAttribute('aria-pressed')).toBe('false')
  })

  it('removes without throwing', () => {
    const { el } = make({ value: '# Hello' })
    expect(() => el.remove()).not.toThrow()
  })

  it('selection bubble is hidden by default (no selection)', async () => {
    const { el } = make()
    await flush()
    const b = bubble(el)!
    expect(b).not.toBeNull()
    expect(b.getAttribute('data-open')).toBe('false')
    expect(b.getAttribute('aria-hidden')).toBe('true')
  })

  it('bubble becomes open when bubblePos is set via _setBubblePos', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setBubblePos?.({ top: 24, left: 120 })
    await flush()
    const b = bubble(el)!
    expect(b.getAttribute('data-open')).toBe('true')
    expect(b.style.top).toBe('24px')
    expect(b.style.left).toBe('120px')
  })

  it('bubble format buttons include Bold, Italic, Strike, Code, Comment by default', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setBubblePos?.({ top: 0, left: 0 })
    await flush()
    const labels = Array.from(shadow(el).querySelectorAll<HTMLButtonElement>('.bubble button'))
      .map(b => b.getAttribute('aria-label'))
    expect(labels).toEqual(expect.arrayContaining(['Bold', 'Italic', 'Strike', 'Code', 'Comment']))
    expect(labels).not.toContain('Generate')
  })

  it('allow-generate adds the Generate button to the bubble', async () => {
    const { el } = make({ 'allow-generate': true }) as { el: SuiMDE }
    await flush()
    el._setBubblePos?.({ top: 0, left: 0 })
    await flush()
    const labels = Array.from(shadow(el).querySelectorAll<HTMLButtonElement>('.bubble button'))
      .map(b => b.getAttribute('aria-label'))
    expect(labels).toContain('Generate')
  })

  it('show-selection-toolbar=false suppresses the bubble entirely', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el.setAttribute('show-selection-toolbar', 'false')
    el._setBubblePos?.({ top: 0, left: 0 })
    await flush()
    expect(bubble(el)).toBeNull()
  })

  it('composer renders when pending is set via _setPending', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setPending?.({ text: 'selected words' })
    await flush()
    const c = composer(el)!
    expect(c).not.toBeNull()
    expect(c.getAttribute('role')).toBe('dialog')
    expect(c.textContent).toContain('selected words')
    expect(composerInput(el)).not.toBeNull()
  })

  it('submit button is disabled when composerValue is blank', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setPending?.({ text: 'x' })
    await flush()
    expect(submitBtn(el)?.disabled).toBe(true)
  })

  it('submit button is enabled when composerValue has content', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setPending?.({ text: 'x' })
    el._setComposerValue?.('my comment')
    await flush()
    expect(submitBtn(el)?.disabled).toBe(false)
  })

  it('clicking submit emits sui-feedback with action:comment', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setPending?.({ text: 'hello there' })
    el._setComposerValue?.('Needs more polish')
    await flush()
    const received: unknown[] = []
    el.addEventListener('sui-feedback', (e) => received.push((e as CustomEvent).detail))
    submitBtn(el)!.click()
    expect(received).toEqual([{ action: 'comment', selection: 'hello there', comment: 'Needs more polish' }])
    await flush()
    expect(composer(el)).toBeNull()
  })

  it('Escape keydown on composer cancels it', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setPending?.({ text: 'x' })
    await flush()
    const ta = composerInput(el)!
    ta.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(composer(el)).toBeNull()
  })

  it('Cmd+Enter on composer textarea submits', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setPending?.({ text: 'snippet' })
    el._setComposerValue?.('the note')
    await flush()
    const received: unknown[] = []
    el.addEventListener('sui-feedback', (e) => received.push((e as CustomEvent).detail))
    composerInput(el)!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true, bubbles: true }))
    expect(received).toEqual([{ action: 'comment', selection: 'snippet', comment: 'the note' }])
  })

  it('toolbar buttons are disabled when component is disabled', async () => {
    const { el } = make({ disabled: true })
    await flush()
    const btns = Array.from(shadow(el).querySelectorAll<HTMLButtonElement>('.toolbar button'))
    expect(btns.length).toBeGreaterThan(0)
    expect(btns.every(b => b.disabled)).toBe(true)
  })

  it('_setMarks updates toolbar button aria-pressed states', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setMarks?.({
      bold: true, italic: false, strike: false, code: false,
      h1: false, h2: false, h3: false, bullet: false, ordered: false, task: false, link: false,
    })
    await flush()
    const bold = shadow(el).querySelector<HTMLButtonElement>('[part="tb-bold"]')!
    expect(bold.getAttribute('aria-pressed')).toBe('true')
  })

  it('bubble is closed when pending is active (composer exclusive)', async () => {
    const { el } = make() as { el: SuiMDE }
    await flush()
    el._setBubblePos?.({ top: 0, left: 0 })
    el._setPending?.({ text: 'selected' })
    await flush()
    expect(bubble(el)?.getAttribute('data-open')).toBe('false')
  })

  it('name attribute reflects on host for form key', () => {
    const { el } = make({ name: 'body' })
    expect(el.getAttribute('name')).toBe('body')
  })
})
