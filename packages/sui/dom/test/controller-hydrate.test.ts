// End-to-end test: a `controller()` export hydrated through the same
// flow the daemon uses for `views/<name>/`. Proves the new typed-
// controller surface composes with `<data>` declarations, `<state>`
// declarations, `derived` memos, `on.*` handlers, and template
// references.
//
// Per the "state is declarative-only" ADR, controllers no longer have
// a `state:` section — every test that needs local state declares it
// in the template via `<state name="..." value="..." />`.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { hydrate } from '../src/legacy'
import {
  controller,
  signal,
  type LiveSignal,
  type LiveSource,
  type LiveState,
} from '@ship/sui-runtime'

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

/** Build a minimal `LiveSource` whose `subscribe` immediately drops
 * the signal into `ok` with `value`. Mirrors what `ship-set-sdk-client`
 * returns for a happy-path SSE call. */
function fakeLive<T>(value: T): LiveSource<T> {
  return {
    subscribe: (sig) => {
      sig.set({ kind: 'ok', value } as LiveState<T>)
      return () => {}
    },
  }
}

describe('hydrate — controller() with template <data>', () => {
  it('derived reads template-declared live signal; handler navigates', async () => {
    const container = mkContainer()
    const navigate = vi.fn<(path: string) => void>()
    const ship = {
      agents: {
        summaries: () => fakeLive({ items: [{ id: 'a-1', name: 'Atlas' }, { id: 'a-2', name: 'Beacon' }] }),
      },
    }

    const def = controller({
      derived: {
        items: ({ agents }: { agents: LiveSignal<{ items: { id: string; name: string }[] }> }) => {
          const s = agents()
          return s.kind === 'ok' ? s.value.items : []
        },
        total: ({ items }: { items: () => { id: string; name: string }[] }) => items().length,
      },
      on: {
        newAgent: ({ navigate: n }: { navigate: (p: string) => void }) => n('/agents/create'),
      },
    })

    const template = `
      <data name="agents" source="ship.agents.summaries()" live />

      <p data-test="total">{total()} agents</p>
      <button data-test="new" on:click={newAgent}>new</button>
      {#if agents().kind === 'ok'}
        <ul>
          {#for a of items() key={a.id}}
            <li data-test="agent">{a.name}</li>
          {/for}
        </ul>
      {/if}
    `

    const handle = hydrate(template, def.setup, { container, ship, navigate })
    await flush()

    expect(container.querySelector('[data-test="total"]')!.textContent).toBe('2 agents')

    expect(container.querySelectorAll('[data-test="agent"]')).toHaveLength(2)
    const names = Array.from(container.querySelectorAll('[data-test="agent"]')).map(
      (n) => n.textContent,
    )
    expect(names).toEqual(['Atlas', 'Beacon'])

    ;(container.querySelector('[data-test="new"]') as HTMLButtonElement).click()
    expect(navigate).toHaveBeenCalledWith('/agents/create')

    handle.dispose()
  })

  it('template <state> + on handler updates DOM through derived', async () => {
    const container = mkContainer()
    const def = controller({
      derived: {
        doubled: ({ count }: { count: () => number }) => count() * 2,
      },
      on: {
        incr: (s: { count: number }) => { s.count++ },
      },
    })

    const template = `
      <state name="count" value="0" />
      <p data-test="count">{count()}</p>
      <p data-test="doubled">{doubled()}</p>
      <button data-test="btn" on:click={incr}>+</button>
    `

    const handle = hydrate(template, def.setup, { container })
    await flush()

    expect(container.querySelector('[data-test="count"]')!.textContent).toBe('0')
    expect(container.querySelector('[data-test="doubled"]')!.textContent).toBe('0')

    const btn = container.querySelector('[data-test="btn"]') as HTMLButtonElement
    btn.click()
    btn.click()
    btn.click()
    await flush()

    expect(container.querySelector('[data-test="count"]')!.textContent).toBe('3')
    expect(container.querySelector('[data-test="doubled"]')!.textContent).toBe('6')

    handle.dispose()
  })

  it('mount runs after render with state proxy + container', async () => {
    const container = mkContainer()
    const mountSpy = vi.fn()
    const def = controller({
      mount: (s, el) => {
        mountSpy(s, el)
        // mutating state in mount should propagate to the DOM
        ;(s as { count: number }).count = 42
      },
    })

    const template = `
      <state name="count" value="0" />
      <p data-test="count">{count()}</p>
    `
    const handle = hydrate(template, def.setup, { container })
    await flush()

    expect(mountSpy).toHaveBeenCalledOnce()
    const [stateArg, elArg] = mountSpy.mock.calls[0]!
    expect(elArg).toBe(container)
    // state proxy: reading 'count' returns the current signal value
    expect((stateArg as { count: number }).count).toBe(42)
    // DOM reflects the mount-time mutation
    expect(container.querySelector('[data-test="count"]')!.textContent).toBe('42')

    handle.dispose()
  })

  it('unmount runs on dispose with state proxy', async () => {
    const container = mkContainer()
    const unmountSpy = vi.fn()
    const def = controller({
      unmount: (s) => unmountSpy(s),
    })

    const template = `
      <state name="count" value="5" />
      <p>{count()}</p>
    `
    const handle = hydrate(template, def.setup, { container })
    await flush()

    expect(unmountSpy).not.toHaveBeenCalled()
    handle.dispose()
    expect(unmountSpy).toHaveBeenCalledOnce()
    expect((unmountSpy.mock.calls[0]![0] as { count: number }).count).toBe(5)
  })

  it('mount/unmount errors are caught and logged, not thrown', async () => {
    const container = mkContainer()
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const def = controller({
      mount: () => { throw new Error('mount-boom') },
      unmount: () => { throw new Error('unmount-boom') },
    })

    const handle = hydrate(
      `<state name="x" value="0" /><p>{x()}</p>`,
      def.setup,
      { container },
    )
    expect(errorSpy).toHaveBeenCalledWith(
      expect.stringContaining('controller mount() threw'),
      expect.any(Error),
    )
    handle.dispose()
    expect(errorSpy).toHaveBeenCalledWith(
      expect.stringContaining('controller unmount() threw'),
      expect.any(Error),
    )
    errorSpy.mockRestore()
  })

  it('controller hook in use: namespaces under the key', async () => {
    const container = mkContainer()
    const items = signal(['apple', 'banana', 'cherry'])

    // Inline hook to keep the test self-contained — hooks keep their
    // own state-bearing shape per the ADR.
    const useFilteredList = (
      source: () => string[],
      pred: (item: string, q: string) => boolean,
    ) => ({
      state: { query: '' },
      derived: {
        filtered: ({ query }: { query: () => string }) =>
          source().filter((i) => pred(i, query())),
      },
      on: {
        setQuery: (s: { query: string }, q: string) => { s.query = q },
      },
    })

    const def = controller({
      use: { list: useFilteredList(() => items(), (s, q) => s.includes(q)) },
    })

    const template = `
      <p data-test="count">{list.filtered().length}</p>
      <button data-test="filter-an" on:click={() => list.setQuery('an')}>an</button>
    `

    const handle = hydrate(template, def.setup, { container })
    await flush()

    expect(container.querySelector('[data-test="count"]')!.textContent).toBe('3')
    ;(container.querySelector('[data-test="filter-an"]') as HTMLButtonElement).click()
    await flush()
    expect(container.querySelector('[data-test="count"]')!.textContent).toBe('1')
    handle.dispose()
  })
})
