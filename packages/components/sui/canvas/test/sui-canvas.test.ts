import { afterEach, describe, expect, it, vi } from 'vitest'
import { act } from 'react'

// Mock Excalidraw — the real React component needs a canvas + layout
// engine jsdom can't provide. The stub captures the bridge's `onChange`
// so a test can simulate a programmatic scene edit, and tracks mount /
// unmount of the React tree so we can assert clean teardown.
const hooks = vi.hoisted(() => ({
  onChange: null as
    | ((els: readonly unknown[], app: Record<string, unknown>, files: Record<string, unknown>) => void)
    | null,
  mounted: false,
  lastTheme: '' as string,
  lastViewMode: false,
}))

vi.mock('@excalidraw/excalidraw', () => {
  const React = require('react') as typeof import('react')
  function Excalidraw(props: {
    onChange?: typeof hooks.onChange
    theme?: string
    viewModeEnabled?: boolean
  }) {
    hooks.onChange = props.onChange ?? null
    hooks.lastTheme = props.theme ?? ''
    hooks.lastViewMode = Boolean(props.viewModeEnabled)
    React.useEffect(() => {
      hooks.mounted = true
      return () => {
        hooks.mounted = false
      }
    }, [])
    return React.createElement('div', { 'data-testid': 'excalidraw-stub' })
  }
  return {
    Excalidraw,
    getSceneVersion: (els: readonly unknown[]) => els.length,
  }
})

import '../src/sui-canvas/sui-canvas.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => {
  reg.cleanup()
  hooks.onChange = null
  hooks.mounted = false
})

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms))

describe('<sui-canvas>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-canvas')).toBeDefined()
  })

  it('attaches a shadow root with a surface container', async () => {
    const { el } = mount(reg, 'sui-canvas')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('.surface')).not.toBeNull()
  })

  it('mounts the Excalidraw React tree on connect', async () => {
    mount(reg, 'sui-canvas')
    await act(async () => {
      await flush()
    })
    expect(hooks.mounted).toBe(true)
  })

  it('passes readonly through as viewModeEnabled', async () => {
    mount(reg, 'sui-canvas', { readonly: true })
    await act(async () => {
      await flush()
    })
    expect(hooks.lastViewMode).toBe(true)
  })

  it('theme="dark" switches Excalidraw to dark mode', async () => {
    mount(reg, 'sui-canvas', { theme: 'dark' })
    await act(async () => {
      await flush()
    })
    expect(hooks.lastTheme).toBe('dark')
  })

  it('fires debounced sui-canvas-change on a scene edit', async () => {
    const { el } = mount(reg, 'sui-canvas', { value: '' })
    await act(async () => {
      await flush()
    })

    const events: Array<{ value: string; sceneVersion: number }> = []
    el.addEventListener('sui-canvas-change', (e) => {
      events.push((e as CustomEvent).detail)
    })

    // Simulate a programmatic Excalidraw edit (two elements drawn).
    expect(hooks.onChange).not.toBeNull()
    hooks.onChange!([{ id: 'a' }, { id: 'b' }], { viewBackgroundColor: '#fff' }, {})

    // Debounced at 200ms — nothing yet.
    expect(events.length).toBe(0)
    await wait(260)

    expect(events.length).toBe(1)
    expect(events[0]!.sceneVersion).toBe(2)
    const parsed = JSON.parse(events[0]!.value)
    expect(parsed.elements).toHaveLength(2)
    expect(parsed.type).toBe('excalidraw')
  })

  it('disposes the React tree cleanly on disconnect', async () => {
    const { el, container } = mount(reg, 'sui-canvas')
    await act(async () => {
      await flush()
    })
    expect(hooks.mounted).toBe(true)

    await act(async () => {
      container.removeChild(el)
      await flush()
    })
    expect(hooks.mounted).toBe(false)
  })
})
