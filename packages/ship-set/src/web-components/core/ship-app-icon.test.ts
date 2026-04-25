import { afterEach, describe, expect, it } from 'vitest'
import './ship-app-icon'
import './ship-icon'
import type { ShipAppIcon } from './ship-app-icon'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipAppIcon
  await el.updateComplete
  // wait for nested ship-icon
  const inner = el.shadowRoot!.querySelector('ship-icon')
  if (inner) await (inner as any).updateComplete
  return el
}

describe('ship-app-icon', () => {
  afterEach(cleanupTestDom)

  it('renders the canonical Lucide glyph for each known app name', async () => {
    const apps = [
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
      const el = await mount(`<ship-app-icon name="${name}"></ship-app-icon>`)
      const inner = el.shadowRoot!.querySelector('ship-icon')
      expect(inner, `expected glyph for ${name}`).not.toBeNull()
      expect(inner!.getAttribute('name')).toBe(glyph)
    }
  })

  it('falls back to a neutral dot glyph for an unknown app name', async () => {
    const el = await mount('<ship-app-icon name="not-an-app"></ship-app-icon>')
    const inner = el.shadowRoot!.querySelector('ship-icon')
    expect(inner!.getAttribute('name')).toBe('dot')
  })

  it('lets the default slot override the glyph for marketplace views', async () => {
    const el = await mount('<ship-app-icon name="canvas"><svg data-custom></svg></ship-app-icon>')
    expect(el.querySelector('svg[data-custom]')).not.toBeNull()
  })

  it('reflects size attribute and reacts to change', async () => {
    const el = await mount('<ship-app-icon name="agents" size="sm"></ship-app-icon>')
    expect(el.getAttribute('size')).toBe('sm')
    el.size = 'lg'
    await el.updateComplete
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('reflects each variant', async () => {
    for (const variant of ['glyph', 'tile', 'solid']) {
      const el = await mount(`<ship-app-icon name="pipelines" variant="${variant}"></ship-app-icon>`)
      expect(el.getAttribute('variant')).toBe(variant)
      expect(el.shadowRoot!.querySelector('.frame')).not.toBeNull()
    }
  })

  it('sets accessibility attrs based on label presence', async () => {
    const labeled = await mount('<ship-app-icon name="git" label="Git"></ship-app-icon>')
    expect(labeled.getAttribute('role')).toBe('img')
    expect(labeled.getAttribute('aria-label')).toBe('Git')

    const bare = await mount('<ship-app-icon name="git"></ship-app-icon>')
    expect(bare.getAttribute('aria-hidden')).toBe('true')
  })

  it('reacts to name attribute changes by rerendering the inner glyph', async () => {
    const el = await mount('<ship-app-icon name="agents"></ship-app-icon>')
    expect(el.shadowRoot!.querySelector('ship-icon')!.getAttribute('name')).toBe('bot')
    el.name = 'skills'
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('ship-icon')!.getAttribute('name')).toBe('sparkles')
  })
})
