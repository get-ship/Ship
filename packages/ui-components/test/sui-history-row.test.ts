import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-history-row/sui-history-row.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-history-row>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-history-row')).toBeDefined()
  })

  it('sets role=button and tabindex=0 on mount', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'abc1234', subject: 'fix: a thing' })
    await flush()
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')
  })

  it('renders the avatar initial from author', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'a', subject: 'x', author: 'ada' })
    await flush()
    expect(shadow(el).querySelector('.avatar')?.textContent).toBe('A')
  })

  it('falls back to ? when author is missing', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'a', subject: 'x' })
    await flush()
    expect(shadow(el).querySelector('.avatar')?.textContent).toBe('?')
  })

  it('splits a conventional-commit subject into scope + rest', async () => {
    const { el } = mount(reg, 'sui-history-row', {
      sha: 'a', subject: 'feat(views): wire data',
    })
    await flush()
    const title = shadow(el).querySelector('.title')!
    expect(title.querySelector('.scope')?.textContent).toBe('feat(views):')
    expect(title.textContent?.includes('wire data')).toBe(true)
  })

  it('renders the whole subject when no conventional prefix', async () => {
    const { el } = mount(reg, 'sui-history-row', {
      sha: 'a', subject: 'just a commit message',
    })
    await flush()
    const title = shadow(el).querySelector('.title')!
    expect(title.querySelector('.scope')).toBeNull()
    expect(title.textContent?.trim()).toBe('just a commit message')
  })

  it('renders sha + timestamp in the meta row', async () => {
    const { el } = mount(reg, 'sui-history-row', {
      sha: 'a1b2c3d', subject: 'x', timestamp: '3h ago',
    })
    await flush()
    const meta = shadow(el).querySelector('.meta')!
    expect(meta.textContent).toContain('a1b2c3d')
    expect(meta.textContent).toContain('3h ago')
  })

  it('omits +/- stats when zero', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'a', subject: 'x' })
    await flush()
    expect(shadow(el).querySelector('.add')).toBeNull()
    expect(shadow(el).querySelector('.del')).toBeNull()
  })

  it('renders both +/- stats when non-zero', async () => {
    const { el } = mount(reg, 'sui-history-row', {
      sha: 'a', subject: 'x', additions: '7', deletions: '3',
    })
    await flush()
    expect(shadow(el).querySelector('.add')?.textContent).toBe('+7')
    expect(shadow(el).querySelector('.del')?.textContent).toBe('−3')
  })

  it('emits sui-select with sha when the message area is clicked', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'cafebabe', subject: 'x' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-select', handler)
    ;(shadow(el).querySelector('.msg') as HTMLElement).click()
    expect(handler).toHaveBeenCalledTimes(1)
    expect((handler.mock.calls[0][0] as CustomEvent).detail).toEqual({ sha: 'cafebabe' })
  })

  it('emits sui-select with sha when the lane is clicked', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'cafebabe', subject: 'x' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-select', handler)
    ;(shadow(el).querySelector('.lane') as HTMLElement).click()
    expect(handler).toHaveBeenCalledTimes(1)
  })

  it('emits sui-select on Enter and Space', async () => {
    const { el } = mount(reg, 'sui-history-row', { sha: 'deadbeef', subject: 'x' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-select', handler)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(handler).toHaveBeenCalledTimes(2)
    expect((handler.mock.calls[0][0] as CustomEvent).detail).toEqual({ sha: 'deadbeef' })
  })

  it('grows the lane dot when active', async () => {
    const { el } = mount(reg, 'sui-history-row', {
      sha: 'a', subject: 'x', active: true,
    })
    await flush()
    const circle = shadow(el).querySelector('circle')!
    expect(circle.getAttribute('r')).toBe('6')
    expect(circle.getAttribute('fill')).toBe('var(--primary)')
  })
})
