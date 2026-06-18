import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-artifact-preview/ship-artifact-preview.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

// The preview never touches the SDK — it inspects `record` + `url` props
// and picks a render branch from previewKindFor(mime). The text kind
// additionally fetches the URL in mount; we stub fetch per test.

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

function recordOf(mime: string, name = 'thing.bin'): string {
  return JSON.stringify({ id: 'a1', name, mime, size: 1, owner_actor: null, created_at: '2026-01-01T00:00:00Z' })
}

describe('<ship-artifact-preview>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-artifact-preview')).toBeDefined()
  })

  it('renders an <img> element for image/png mime', async () => {
    const { el } = mount(reg, 'ship-artifact-preview', {
      record: recordOf('image/png', 'arch.png'),
      url: '/api/v1/artifacts/a1',
    })
    await flush()
    const img = shadow(el).querySelector('img[data-test="preview-image"]')
    expect(img).not.toBeNull()
    expect(img?.getAttribute('src')).toBe('/api/v1/artifacts/a1')
  })

  it('renders a sandboxed <iframe> for text/html mime', async () => {
    const { el } = mount(reg, 'ship-artifact-preview', {
      record: recordOf('text/html', 'page.html'),
      url: '/api/v1/artifacts/a1',
    })
    await flush()
    const frame = shadow(el).querySelector('iframe[data-test="preview-frame"]')
    expect(frame).not.toBeNull()
    expect(frame?.hasAttribute('sandbox')).toBe(true)
  })

  it('fetches the URL and renders the text body for text/markdown', async () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      ok: true,
      status: 200,
      text: () => Promise.resolve('# hello\nworld'),
    } as Response)
    try {
      const { el } = mount(reg, 'ship-artifact-preview', {
        record: recordOf('text/markdown', 'notes.md'),
        url: '/api/v1/artifacts/a1',
      })
      await flush()
      await flush()
      const pre = shadow(el).querySelector('pre[data-test="preview-pre"]')
      expect(pre).not.toBeNull()
      expect(pre?.textContent).toContain('# hello')
      expect(fetchSpy).toHaveBeenCalledWith('/api/v1/artifacts/a1')
    } finally {
      fetchSpy.mockRestore()
    }
  })

  it('shows the binary fallback + download link for application/wasm', async () => {
    const { el } = mount(reg, 'ship-artifact-preview', {
      record: recordOf('application/wasm', 'module.wasm'),
      url: '/api/v1/artifacts/a1',
    })
    await flush()
    const bin = shadow(el).querySelector('[data-test="preview-binary"]')
    expect(bin).not.toBeNull()
    expect(bin?.textContent).toContain('download')
    const link = bin?.querySelector('a')
    expect(link?.getAttribute('href')).toBe('/api/v1/artifacts/a1')
  })

  it('shows the error message when the text fetch fails', async () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      ok: false,
      status: 500,
      text: () => Promise.resolve(''),
    } as Response)
    try {
      const { el } = mount(reg, 'ship-artifact-preview', {
        record: recordOf('text/plain', 'log.txt'),
        url: '/api/v1/artifacts/a1',
      })
      await flush()
      await flush()
      const err = shadow(el).querySelector('[data-test="preview-error"]')
      expect(err).not.toBeNull()
      expect(err?.textContent).toContain('500')
    } finally {
      fetchSpy.mockRestore()
    }
  })
})
