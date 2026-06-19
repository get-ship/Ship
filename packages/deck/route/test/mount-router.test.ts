import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

// Record every runtime.mount() call. Hoisted so the vi.mock factory (also
// hoisted) can close over it.
const { mountCalls } = vi.hoisted(() => ({
  mountCalls: [] as Array<{ target: unknown; name: unknown; opts: any }>,
}))

// Mock the runtime instead of importing it: the real @ship/sui-runtime pulls
// chrome.ts → @ship/deck/route, a cycle vitest can't resolve (pre-existing).
// A minimal signal + a recording mount is all the router needs.
vi.mock('@ship/sui-runtime', () => {
  const signal = <T>(initial: T) => {
    let v = initial
    const s = (() => v) as { (): T; set(n: T): void; peek(): T; __peek(): T }
    s.set = (n: T) => { v = n }
    s.peek = () => v
    s.__peek = () => v
    return s
  }
  const mount = vi.fn(async (target: unknown, name: unknown, opts: any) => {
    mountCalls.push({ target, name, opts })
    return { dispose: vi.fn() }
  })
  return { signal, mount }
})

import { mountRouter, type RouterHandle } from '../src/mount-router'
import { navigate, setEditMode } from '../src/index'

const MOD_HOME = { __id: 'home' }
const MOD_THEMES = { __id: 'themes' }

let router: RouterHandle | undefined

beforeEach(() => {
  // Edit mode skips the defineRoutes baseline gate so each test can register
  // its own pattern set without tripping the static-baseline check.
  setEditMode(true)
  history.replaceState({}, '', '/')
  document.body.innerHTML = '<div id="ship-root"></div>'
  mountCalls.length = 0
})

afterEach(() => {
  router?.dispose()
  router = undefined
})

const start = (layouts: { pathPrefix: string; tag: string }[] = []) =>
  mountRouter({
    root: '#ship-root',
    routes: [{ pattern: '/' }, { pattern: '/studio/themes' }],
    layouts,
    modules: { '/': MOD_HOME, '/studio/themes': MOD_THEMES },
  })

const flush = () => vi.waitFor(() => expect(mountCalls.length).toBeGreaterThan(0))

describe('mountRouter', () => {
  it('mounts the matched index module into root on start', async () => {
    router = start()
    await flush()
    expect(mountCalls).toHaveLength(1)
    expect(mountCalls[0].opts.module).toBe(MOD_HOME)
    expect(mountCalls[0].target).toBe(document.getElementById('ship-root'))
  })

  it('navigate() routes through the registered port and mounts the new module', async () => {
    router = start()
    await flush()
    navigate('/studio/themes')
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    expect(mountCalls[1].opts.module).toBe(MOD_THEMES)
    expect(location.pathname).toBe('/studio/themes')
  })

  it('wraps the route in the layout chrome chain and mounts into the inner slot', async () => {
    router = start([{ pathPrefix: '/studio', tag: 'sui-studio-chrome' }])
    await flush()
    navigate('/studio/themes')
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    const root = document.getElementById('ship-root')!
    const chrome = root.querySelector('sui-studio-chrome')
    expect(chrome).not.toBeNull()
    const slot = chrome!.querySelector('div[slot="route"]')
    expect(slot).not.toBeNull()
    expect(mountCalls[1].target).toBe(slot)
  })

  it('intercepts same-origin anchor clicks as SPA navigations', async () => {
    router = start()
    await flush()
    const a = document.createElement('a')
    a.href = '/studio/themes'
    a.textContent = 'Themes'
    document.body.appendChild(a)
    a.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, button: 0 }))
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    expect(mountCalls[1].opts.module).toBe(MOD_THEMES)
    expect(location.pathname).toBe('/studio/themes')
  })

  it('clears root and does not mount when no route matches', async () => {
    router = start()
    await flush()
    navigate('/does-not-exist')
    await vi.waitFor(() => expect(location.pathname).toBe('/does-not-exist'))
    // No matching module → no second mount; root emptied.
    expect(mountCalls).toHaveLength(1)
    expect(document.getElementById('ship-root')!.childNodes.length).toBe(0)
  })

  it('passes clients through generically (no `ship` baked into the router)', async () => {
    const clients = { ship: { tag: 'connect-client' } }
    router = mountRouter({
      root: '#ship-root',
      routes: [{ pattern: '/' }],
      modules: { '/': MOD_HOME },
      clients,
    })
    await flush()
    expect(mountCalls[0].opts.clients).toBe(clients)
  })
})
