// Smoke tests for <ship-markdown-editor> .set port.
//
// Same caveat as the Lit suite: JSDOM can't run the full ProseMirror /
// TipTap stack, so these tests focus on the shell — light-DOM
// rendering, attribute reflection, form-associated shape, sub-component
// composition, and the bubble + composer surface (driven by attribute
// flips, since the parent's TipTap-coordinate path is unreachable in
// JSDOM). End-to-end behavior is verified in Playwright.

import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-markdown-editor (sail)', () => {
  let mainTag: string
  let toolbarTag: string
  let bubbleTag: string

  beforeAll(async () => {
    toolbarTag = await loadSailComponent('ship-markdown-editor-toolbar.set')
    bubbleTag = await loadSailComponent('ship-markdown-editor-bubble.set')
    mainTag = await loadSailComponent('ship-markdown-editor.set')
  }, 120_000)

  afterEach(() => { document.body.innerHTML = '' })

  function mount(markup: string) {
    document.body.insertAdjacentHTML('beforeend', markup)
    return document.body.lastElementChild as HTMLElement
  }

  it('registers all three custom elements', () => {
    expect(customElements.get(mainTag)).toBeTruthy()
    expect(customElements.get(toolbarTag)).toBeTruthy()
    expect(customElements.get(bubbleTag)).toBeTruthy()
  })

  it('renders shell + toolbar + editor host', () => {
    const el = mount(`<${mainTag} placeholder="Write..."></${mainTag}>`)
    expect(el.querySelector('.shell')).toBeTruthy()
    expect(el.querySelector(toolbarTag)).toBeTruthy()
    expect(el.querySelector('[data-editor-host]')).toBeTruthy()
  })

  it('rows attribute drives --ed-min-h', () => {
    const el = mount(`<${mainTag} size="sm" rows="8"></${mainTag}>`)
    expect(el.getAttribute('size')).toBe('sm')
    expect(el.style.getPropertyValue('--ed-min-h')).toBe('224px')
  })

  it('reflects invalid + disabled to ARIA on the host', () => {
    const el = mount(`<${mainTag} invalid disabled></${mainTag}>`)
    expect(el.getAttribute('aria-invalid')).toBe('true')
    expect(el.getAttribute('aria-disabled')).toBe('true')
    const tb = el.querySelector(toolbarTag)!
    const firstBtn = tb.querySelector<HTMLButtonElement>('button')!
    expect(firstBtn.disabled).toBe(true)
  })

  it('label flows to the toolbar aria-label', () => {
    const el = mount(`<${mainTag} label="Notes editor"></${mainTag}>`)
    const tb = el.querySelector(toolbarTag)!
    const role = tb.querySelector('[role="toolbar"]')!
    expect(role.getAttribute('aria-label')).toBe('Notes editor')
  })

  it('toolbar default aria-label is "Editor toolbar" when label empty', () => {
    const el = mount(`<${mainTag}></${mainTag}>`)
    const role = el.querySelector('[role="toolbar"]')!
    expect(role.getAttribute('aria-label')).toBe('Editor toolbar')
  })

  it('main is form-associated', () => {
    const ctor = customElements.get(mainTag) as { formAssociated?: boolean }
    expect(ctor.formAssociated).toBe(true)
  })

  it('toolbar exposes the minimum-set buttons with aria-pressed', () => {
    const el = mount(`<${mainTag}></${mainTag}>`)
    const labels = Array.from(el.querySelectorAll<HTMLButtonElement>(`${toolbarTag} button`))
      .map(b => b.getAttribute('aria-label'))
    for (const expected of ['Bold', 'Italic', 'Inline code', 'Heading 1', 'Heading 2',
      'Heading 3', 'Bullet list', 'Ordered list', 'Task list', 'Link']) {
      expect(labels).toContain(expected)
    }
    const bold = el.querySelector<HTMLButtonElement>('[part="tb-bold"]')!
    expect(bold.getAttribute('aria-pressed')).toBe('false')
  })

  it('teardown removes the element without throwing', () => {
    const el = mount(`<${mainTag} value="# Hello"></${mainTag}>`)
    expect(() => el.remove()).not.toThrow()
  })

  it('toolbar mark click bubbles ship-mark-toggle to the host', () => {
    const el = mount(`<${mainTag}></${mainTag}>`)
    const received: unknown[] = []
    el.addEventListener('ship-mark-toggle', (e) => received.push((e as CustomEvent).detail))
    const bold = el.querySelector<HTMLButtonElement>('[part="tb-bold"]')!
    bold.click()
    expect(received).toEqual([{ kind: 'bold' }])
  })

  it('toolbar link click bubbles ship-link-click to the host', () => {
    const el = mount(`<${mainTag}></${mainTag}>`)
    let fired = false
    el.addEventListener('ship-link-click', () => { fired = true })
    const link = el.querySelector<HTMLButtonElement>('[part="tb-link"]')!
    link.click()
    expect(fired).toBe(true)
  })

  it('selection bubble lives in the DOM by default and is hidden', () => {
    const el = mount(`<${mainTag}></${mainTag}>`)
    const bubble = el.querySelector(bubbleTag)!
    expect(bubble).toBeTruthy()
    const inner = bubble.querySelector('.bubble')!
    expect(inner.getAttribute('data-open')).toBe('false')
    expect(inner.getAttribute('aria-hidden')).toBe('true')
  })

  it('show-selection-toolbar=false suppresses the bubble entirely', () => {
    document.body.insertAdjacentHTML('beforeend',
      `<${mainTag} show-selection-toolbar="false"></${mainTag}>`)
    const el = document.body.lastElementChild as HTMLElement
    expect(el.querySelector(bubbleTag)).toBeNull()
  })

  describe('bubble sub-component', () => {
    function mountBubble(attrs: string = '') {
      document.body.insertAdjacentHTML('beforeend',
        `<${bubbleTag} ${attrs}></${bubbleTag}>`)
      return document.body.lastElementChild as HTMLElement
    }

    it('renders B/I/S/code + Comment by default', () => {
      const el = mountBubble('visible bubble-top="24" bubble-left="120"')
      const labels = Array.from(el.querySelectorAll<HTMLButtonElement>('.bubble button'))
        .map(b => b.getAttribute('aria-label'))
      expect(labels).toEqual(expect.arrayContaining(['Bold', 'Italic', 'Strike', 'Code', 'Comment']))
      expect(labels).not.toContain('Generate')
    })

    it('allow-generate adds the Generate button', () => {
      const el = mountBubble('visible allow-generate bubble-top="0" bubble-left="0"')
      const labels = Array.from(el.querySelectorAll<HTMLButtonElement>('.bubble button'))
        .map(b => b.getAttribute('aria-label'))
      expect(labels).toContain('Generate')
    })

    it('allow-comment="false" hides the Comment button', () => {
      const el = mountBubble('visible allow-comment="false" bubble-top="0" bubble-left="0"')
      const labels = Array.from(el.querySelectorAll<HTMLButtonElement>('.bubble button'))
        .map(b => b.getAttribute('aria-label'))
      expect(labels).not.toContain('Comment')
    })

    it('opens with data-open=true and the supplied position when visible', () => {
      const el = mountBubble('visible bubble-top="24" bubble-left="120"')
      const bubble = el.querySelector<HTMLElement>('.bubble')!
      expect(bubble.getAttribute('data-open')).toBe('true')
      expect(bubble.style.top).toBe('24px')
      expect(bubble.style.left).toBe('120px')
    })

    it('mark click dispatches ship-mark-toggle', () => {
      const el = mountBubble('visible bubble-top="0" bubble-left="0"')
      const seen: unknown[] = []
      el.addEventListener('ship-mark-toggle', (e) => seen.push((e as CustomEvent).detail))
      const strike = el.querySelector<HTMLButtonElement>('[part="bubble-strike"]')!
      strike.click()
      expect(seen).toEqual([{ kind: 'strike' }])
    })

    it('comment click dispatches ship-bubble-action with action:comment', () => {
      const el = mountBubble('visible bubble-top="0" bubble-left="0"')
      const seen: unknown[] = []
      el.addEventListener('ship-bubble-action', (e) => seen.push((e as CustomEvent).detail))
      const cmt = el.querySelector<HTMLButtonElement>('[part="bubble-comment"]')!
      cmt.click()
      expect(seen).toEqual([{ action: 'comment' }])
    })

    it('renders the comment composer when pending-active is set', () => {
      const el = mountBubble('pending-active pending-text="selected words"')
      const composer = el.querySelector<HTMLElement>('.composer')!
      expect(composer).toBeTruthy()
      expect(composer.getAttribute('role')).toBe('dialog')
      expect(composer.textContent).toContain('selected words')
      const ta = el.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
      expect(ta).toBeTruthy()
    })

    it('submit emits ship-composer-submit with comment + selection', async () => {
      const el = mountBubble('pending-active pending-text="hello there"')
      const seen: unknown[] = []
      el.addEventListener('ship-composer-submit', (e) => seen.push((e as CustomEvent).detail))
      const ta = el.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
      ta.value = 'Needs more polish'
      ta.dispatchEvent(new Event('input', { bubbles: true }))
      const submit = el.querySelector<HTMLButtonElement>('.composer .submit')!
      submit.click()
      expect(seen).toEqual([{ comment: 'Needs more polish', selection: 'hello there' }])
    })

    it('submit is disabled while comment is blank', () => {
      const el = mountBubble('pending-active pending-text="x"')
      const submit = el.querySelector<HTMLButtonElement>('.composer .submit')!
      expect(submit.disabled).toBe(true)
    })

    it('Escape in composer dispatches ship-composer-cancel', () => {
      const el = mountBubble('pending-active pending-text="x"')
      let cancelled = false
      el.addEventListener('ship-composer-cancel', () => { cancelled = true })
      const ta = el.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
      ta.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
      expect(cancelled).toBe(true)
    })

    it('Cmd+Enter in composer dispatches ship-composer-submit', () => {
      const el = mountBubble('pending-active pending-text="snippet"')
      const seen: unknown[] = []
      el.addEventListener('ship-composer-submit', (e) => seen.push((e as CustomEvent).detail))
      const ta = el.querySelector<HTMLTextAreaElement>('[data-composer-input]')!
      ta.value = 'the note'
      ta.dispatchEvent(new Event('input', { bubbles: true }))
      ta.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true, bubbles: true }))
      expect(seen).toEqual([{ comment: 'the note', selection: 'snippet' }])
    })
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        const el = document.createElement(mainTag)
        el.setAttribute('tone', tone)
        el.setAttribute('size', size)
        document.body.appendChild(el)
        expect(el.querySelector('.shell')).toBeTruthy()
        el.remove()
        n++
      }
    }
    expect(n).toBe(7 * 3)
  })
})
