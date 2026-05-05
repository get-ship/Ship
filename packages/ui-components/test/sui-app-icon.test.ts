import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-app-icon/dist/sui-app-icon.compiled.js'
import '../src/sui-icon/dist/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-app-icon>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-app-icon')).toBeDefined()
  })

  it('has a shadow root containing .frame', () => {
    const { shadow } = mount(reg, 'sui-app-icon', { name: 'agents' })
    expect(shadow!.querySelector('.frame')).not.toBeNull()
  })

  it('renders the canonical Lucide glyph for each known app name', async () => {
    const apps: [string, string][] = [
      ['dashboard', 'layout-dashboard'],
      ['pipelines', 'workflow'],
      ['agents', 'bot'],
      ['skills', 'sparkles'],
      ['git', 'git-branch'],
      ['collections', 'database'],
      ['canvas', 'frame'],
      ['settings', 'settings'],
      ['chat', 'messages-square'],
      ['events', 'radio'],
      ['specs', 'file-text'],
      ['theme', 'palette'],
      ['setup', 'rocket'],
      ['overview', 'compass'],
    ]
    for (const [name, glyph] of apps) {
      const { el, shadow } = mount(reg, 'sui-app-icon', { name })
      await flush()
      const icon = shadow!.querySelector('sui-icon')
      expect(icon, `expected glyph for ${name}`).not.toBeNull()
      expect(icon!.getAttribute('name')).toBe(glyph)
      el.remove()
    }
  })

  it('falls back to dot glyph for unknown app name', async () => {
    const { shadow } = mount(reg, 'sui-app-icon', { name: 'not-an-app' })
    await flush()
    const icon = shadow!.querySelector('sui-icon')
    expect(icon!.getAttribute('name')).toBe('dot')
  })

  it('resolves theme-studio alias to palette glyph', async () => {
    const { shadow } = mount(reg, 'sui-app-icon', { name: 'theme-studio' })
    await flush()
    const icon = shadow!.querySelector('sui-icon')
    expect(icon!.getAttribute('name')).toBe('palette')
  })

  it('reflects size attribute', () => {
    const { el } = mount(reg, 'sui-app-icon', { name: 'agents', size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('reflects each variant', () => {
    for (const variant of ['glyph', 'tile', 'solid']) {
      const { el, shadow } = mount(reg, 'sui-app-icon', { name: 'pipelines', variant })
      expect(el.getAttribute('variant')).toBe(variant)
      expect(shadow!.querySelector('.frame')).not.toBeNull()
    }
  })

  it('sets role=img and aria-label when label is provided', async () => {
    const { el } = mount(reg, 'sui-app-icon', { name: 'git', label: 'Git' })
    await flush()
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Git')
    expect(el.hasAttribute('aria-hidden')).toBe(false)
  })

  it('sets aria-hidden when no label', async () => {
    const { el } = mount(reg, 'sui-app-icon', { name: 'git' })
    await flush()
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.hasAttribute('role')).toBe(false)
  })

  it('updates glyph when name attribute changes', async () => {
    const { el, shadow } = mount(reg, 'sui-app-icon', { name: 'agents' })
    await flush()
    expect(shadow!.querySelector('sui-icon')!.getAttribute('name')).toBe('bot')
    el.setAttribute('name', 'skills')
    await flush()
    expect(shadow!.querySelector('sui-icon')!.getAttribute('name')).toBe('sparkles')
  })

  it('accepts slotted custom glyph', () => {
    const { el } = mount(reg, 'sui-app-icon', { name: 'canvas' }, '<svg data-custom></svg>')
    expect(el.querySelector('svg[data-custom]')).not.toBeNull()
  })
})
