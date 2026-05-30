import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-git-diff/sui-git-diff.compiled.js'
import '../src/sui-git-diff/sui-git-diff-file.compiled.js'
import '../src/sui-git-diff/sui-git-diff-hunk.compiled.js'
import '../src/sui-git-diff/sui-git-diff-line.compiled.js'
// sui-icon lives in @ship/sui-primitives.
import '@ship/sui-primitives'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-git-diff> container', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-git-diff')).toBeDefined()
  })

  it('shows the empty placeholder when files === 0', async () => {
    const { el } = mount(reg, 'sui-git-diff')
    await flush()
    expect(shadow(el).querySelector('.empty')?.textContent).toBe('No changes to display')
    expect(shadow(el).querySelector('.toolbar')).toBeNull()
  })

  it('shows the empty placeholder when empty=true even with files', async () => {
    const { el } = mount(reg, 'sui-git-diff', { files: '3', empty: true })
    await flush()
    expect(shadow(el).querySelector('.empty')).not.toBeNull()
  })

  it('renders the toolbar with file count + +/- when files > 0', async () => {
    const { el } = mount(reg, 'sui-git-diff', {
      files: '2', additions: '12', deletions: '4',
    })
    await flush()
    const tb = shadow(el).querySelector('.toolbar')!
    expect(tb.querySelector('.count')?.textContent).toBe('2 files changed')
    expect(tb.querySelector('.adds')?.textContent).toBe('+12')
    expect(tb.querySelector('.dels')?.textContent).toBe('-4')
  })

  it('singularizes the file count when files === 1', async () => {
    const { el } = mount(reg, 'sui-git-diff', { files: '1' })
    await flush()
    expect(shadow(el).querySelector('.count')?.textContent).toBe('1 file changed')
  })

  it('toggleAll flips collapsed on slotted files', async () => {
    const { el } = mount(
      reg,
      'sui-git-diff',
      { files: '2' },
      '<sui-git-diff-file path="a.ts"></sui-git-diff-file>' +
        '<sui-git-diff-file path="b.ts"></sui-git-diff-file>',
    )
    await flush()
    const btn = shadow(el).querySelector('.toggle') as HTMLButtonElement
    btn.click()
    await flush()
    const files = el.querySelectorAll('sui-git-diff-file')
    files.forEach((f) => expect(f.hasAttribute('collapsed')).toBe(true))
    expect(shadow(el).querySelector('.toggle')?.textContent?.trim()).toContain('Expand all')
    btn.click()
    await flush()
    files.forEach((f) => expect(f.hasAttribute('collapsed')).toBe(false))
  })

  it('sets role=region + aria-label', async () => {
    const { el } = mount(reg, 'sui-git-diff', {
      files: '2', additions: '5', deletions: '1',
    })
    await flush()
    expect(el.getAttribute('role')).toBe('region')
    expect(el.getAttribute('aria-label')).toBe('Diff: 2 files changed, 5 additions, 1 deletions')
  })
})

describe('<sui-git-diff-file>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-git-diff-file')).toBeDefined()
  })

  it('renders the path + +/- counts', async () => {
    const { el } = mount(reg, 'sui-git-diff-file', {
      path: 'src/index.ts', additions: '3', deletions: '1',
    })
    await flush()
    expect(shadow(el).querySelector('.path')?.textContent).toBe('src/index.ts')
    expect(shadow(el).querySelector('.adds')?.textContent).toContain('3')
    expect(shadow(el).querySelector('.dels')?.textContent).toContain('1')
  })

  it('clicking the header toggles collapsed', async () => {
    const { el } = mount(reg, 'sui-git-diff-file', { path: 'a.ts' })
    await flush()
    expect(el.hasAttribute('collapsed')).toBe(false)
    ;(shadow(el).querySelector('.header') as HTMLButtonElement).click()
    await flush()
    expect(el.hasAttribute('collapsed')).toBe(true)
    expect(shadow(el).querySelector('.header')?.getAttribute('aria-expanded')).toBe('false')
  })

  it('sets role=group + aria-label', async () => {
    const { el } = mount(reg, 'sui-git-diff-file', {
      path: 'a.ts', additions: '2', deletions: '3',
    })
    await flush()
    expect(el.getAttribute('role')).toBe('group')
    expect(el.getAttribute('aria-label')).toBe('a.ts: +2 -3')
  })
})

describe('<sui-git-diff-hunk>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-git-diff-hunk')).toBeDefined()
  })

  it('renders the header text', async () => {
    const { el } = mount(reg, 'sui-git-diff-hunk', { header: '@@ -10,7 +10,8 @@' })
    await flush()
    // Strip the inline <style> contents — only the rendered hunk text matters.
    const shadowText = Array.from(shadow(el).childNodes)
      .filter((n) => n.nodeType === Node.TEXT_NODE || (n as Element).tagName !== 'STYLE')
      .map((n) => n.textContent ?? '')
      .join('')
      .trim()
    expect(shadowText).toBe('@@ -10,7 +10,8 @@')
  })
})

describe('<sui-git-diff-line>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-git-diff-line')).toBeDefined()
  })

  it('renders old/new gutters and a context space signature', async () => {
    const { el } = mount(reg, 'sui-git-diff-line', { old: '10', new: '10' })
    await flush()
    const nums = shadow(el).querySelectorAll('.num')
    expect(nums[0].textContent).toBe('10')
    expect(nums[1].textContent).toBe('10')
    expect(shadow(el).querySelector('.sig')?.textContent).toBe(' ')
  })

  it('renders + signature for add type', async () => {
    const { el } = mount(reg, 'sui-git-diff-line', { type: 'add', new: '11' })
    await flush()
    expect(shadow(el).querySelector('.sig')?.textContent).toBe('+')
  })

  it('renders - signature for del type', async () => {
    const { el } = mount(reg, 'sui-git-diff-line', { type: 'del', old: '11' })
    await flush()
    expect(shadow(el).querySelector('.sig')?.textContent).toBe('-')
  })

  it('emits sui-comment with old + new when the comment button is clicked', async () => {
    const { el } = mount(reg, 'sui-git-diff-line', { old: '10', new: '11', type: 'add' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-comment', handler)
    ;(shadow(el).querySelector('.comment-btn') as HTMLButtonElement).click()
    expect(handler).toHaveBeenCalledTimes(1)
    expect((handler.mock.calls[0][0] as CustomEvent).detail).toEqual({ old: '10', new: '11' })
  })

  it('updates aria-label per line type', async () => {
    const { el } = mount(reg, 'sui-git-diff-line', { type: 'add', new: '11' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('line 11 added')
  })
})
