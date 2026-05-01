// End-to-end tests for the three v1 template hooks via hydrate:
// `<use-kbd>`, `<use-click-outside>`, `<use-shell>`.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { hydrate } from '../src'
import { controller } from '@ship/ui-runtime'

const containers: HTMLElement[] = []
const mkContainer = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

const flush = () => new Promise((r) => setTimeout(r, 0))

describe('<use-kbd>', () => {
  it('routes document keydown to the action named on each <on>', async () => {
    const container = mkContainer()
    const onSearch = vi.fn()
    const onCreate = vi.fn()
    const def = controller({
      on: { search: onSearch, create: onCreate },
    })

    const template = `
      <use-kbd>
        <on key="/"      action="search" />
        <on key="cmd+n"  action="create" />
      </use-kbd>
      <p>x</p>
    `

    const handle = hydrate(template, def.setup, { container })
    await flush()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: '/' }))
    expect(onSearch).toHaveBeenCalledOnce()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', metaKey: true }))
    expect(onCreate).toHaveBeenCalledOnce()

    handle.dispose()
    // After dispose, document listener is removed.
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '/' }))
    expect(onSearch).toHaveBeenCalledOnce() // unchanged
  })

  it('ignores keys with no matching shortcut', async () => {
    const container = mkContainer()
    const onSearch = vi.fn()
    const def = controller({ on: { search: onSearch } })
    const template = `<use-kbd><on key="/" action="search" /></use-kbd><p>x</p>`
    const handle = hydrate(template, def.setup, { container })
    await flush()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }))
    expect(onSearch).not.toHaveBeenCalled()
    handle.dispose()
  })
})

describe('<use-click-outside>', () => {
  it('fires the action when a click lands outside the bound target', async () => {
    const container = mkContainer()
    const onClose = vi.fn()
    const def = controller({ on: { close: onClose } })

    // We simulate the binding by exposing `dialogEl` via state and
    // setting it from mount. In a full app the value would come from
    // a component ref; the binding mechanism is the same.
    const target = document.createElement('div')
    target.id = 'dlg'
    document.body.appendChild(target)

    // Provide a custom setup that returns `dialogEl` as a scope value
    // (no signal — `<use-click-outside>` reads attrs once at mount).
    const customSetup = () => ({
      ...def.setup({ ship: undefined }),
      dialogEl: target,
    })

    const template = `
      <use-click-outside bind:target={dialogEl} bind:action={close} />
      <p>x</p>
    `

    const handle = hydrate(template, customSetup, { container })
    await flush()

    // Click inside target — no action.
    target.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    expect(onClose).not.toHaveBeenCalled()

    // Click outside — action fires.
    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
    expect(onClose).toHaveBeenCalledOnce()

    handle.dispose()
    target.remove()
  })
})

describe('<use-shell>', () => {
  it('contributes shell context under the as= identifier', async () => {
    const container = mkContainer()
    // Stub the shell global the hook reads.
    ;(window as unknown as { __shipShell: unknown }).__shipShell = {
      theme: 'dark',
      currentUser: { id: 'u1', name: 'Alice' },
    }

    const template = `
      <use-shell as="shell" />
      <p data-test="theme">{shell.theme}</p>
      <p data-test="user">{shell.currentUser.name}</p>
    `

    const handle = hydrate(template, undefined, { container })
    await flush()

    expect(container.querySelector('[data-test="theme"]')!.textContent).toBe('dark')
    expect(container.querySelector('[data-test="user"]')!.textContent).toBe('Alice')

    handle.dispose()
    delete (window as unknown as { __shipShell?: unknown }).__shipShell
  })

  it('returns an empty object when no shell global is set', async () => {
    const container = mkContainer()
    delete (window as unknown as { __shipShell?: unknown }).__shipShell

    // The hook returns `{}` so destructuring `shell.theme` is undefined,
    // not a crash. Use a defensive expression in the template.
    const template = `
      <use-shell as="shell" />
      <p data-test="theme">{shell.theme ?? 'default'}</p>
    `

    const handle = hydrate(template, undefined, { container })
    await flush()

    expect(container.querySelector('[data-test="theme"]')!.textContent).toBe('default')
    handle.dispose()
  })
})

describe('unknown <use-*>', () => {
  it('logs a warning and continues hydrating', async () => {
    const container = mkContainer()
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})

    const template = `
      <use-not-a-real-hook foo="bar" />
      <p data-test="x">ok</p>
    `

    const handle = hydrate(template, undefined, { container })
    await flush()

    expect(warn).toHaveBeenCalledWith(
      expect.stringContaining('no template hook registered for <use-not-a-real-hook>'),
    )
    expect(container.querySelector('[data-test="x"]')!.textContent).toBe('ok')
    handle.dispose()
    warn.mockRestore()
  })
})
