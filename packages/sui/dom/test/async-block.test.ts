// Tests for the `{#async}` template directive — parse-time desugar to
// while-loading + while-error + if-ok with optional `as v` reactive
// scope binding.

import { describe, expect, it } from 'vitest'
import { hydrate } from '../src/legacy'
import { signal, type LiveSignal, type LiveSource } from '@ship/sui-runtime'

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
const flush = () => new Promise((r) => setTimeout(r, 0))

// Fake LiveSource that fires a single state value on subscribe.
const live = <T>(state: { kind: 'ok'; value: T } | { kind: 'loading' } | { kind: 'error'; error: Error }): LiveSource<T> => ({
  subscribe: (sig) => {
    sig.set(state as Parameters<typeof sig.set>[0])
    return () => {}
  },
})

const mountWith = (template: string, ship: unknown) => {
  const c = mk()
  return { handle: hydrate(template, undefined, { container: c, ship }), c }
}

describe('{#async} directive', () => {
  it('renders {:ok} when the signal is in the ok state', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d}
         {:ok}<div class="ok">ready</div>
       {/async}`,
      { fake: () => live({ kind: 'ok', value: 42 }) },
    )
    await flush()
    expect(c.querySelector('.ok')?.textContent).toBe('ready')
    handle.dispose()
  })

  it('renders {:loading} body while the signal is loading', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d}
         {:loading}<div class="load">loading…</div>
         {:ok}<div class="ok">ready</div>
       {/async}`,
      { fake: () => live({ kind: 'loading' }) },
    )
    await flush()
    expect(c.querySelector('.load')?.textContent).toBe('loading…')
    expect(c.querySelector('.ok')).toBeNull()
    handle.dispose()
  })

  it('renders {:error err} body and binds err', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d}
         {:error err}<div class="err">{err.message}</div>
         {:ok}<div class="ok">ready</div>
       {/async}`,
      { fake: () => live({ kind: 'error', error: new Error('boom') }) },
    )
    await flush()
    expect(c.querySelector('.err')?.textContent).toBe('boom')
    handle.dispose()
  })

  it('falls back to default <sui-spinner centered> when {:loading} omitted', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d}
         {:ok}<div class="ok">ready</div>
       {/async}`,
      { fake: () => live({ kind: 'loading' }) },
    )
    await flush()
    expect(c.querySelector('sui-spinner')).not.toBeNull()
    expect(c.querySelector('sui-spinner')?.getAttribute('centered')).toBe('')
    handle.dispose()
  })

  it('falls back to default <sui-error-state> when {:error} omitted', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d}
         {:ok}<div class="ok">ready</div>
       {/async}`,
      { fake: () => live({ kind: 'error', error: new Error('nope') }) },
    )
    await flush()
    expect(c.querySelector('sui-error-state')?.textContent).toContain('nope')
    handle.dispose()
  })

  it('`as v` binds v to .value inside :ok', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d as v}
         {:ok}<div class="ok">value is {v.field}</div>
       {/async}`,
      { fake: () => live({ kind: 'ok', value: { field: 'hello' } }) },
    )
    await flush()
    expect(c.querySelector('.ok')?.textContent).toBe('value is hello')
    handle.dispose()
  })

  it('throws on missing {:ok}', () => {
    expect(() =>
      hydrate(
        `<data name="d" source="ship.fake()" live />
         {#async d}
           {:loading}<div>nope</div>
         {/async}`,
        ((_s: any) => ({ fake: () => live({ kind: 'loading' }) })) as any,
        { container: mk() },
      ),
    ).toThrow(/requires.*{:ok}/)
  })

  it('throws on malformed `as` variable', () => {
    expect(() =>
      hydrate(
        `<data name="d" source="ship.fake()" live />
         {#async d as 1bad}
           {:ok}<div>x</div>
         {/async}`,
        ((_s: any) => ({ fake: () => live({ kind: 'ok', value: 1 }) })) as any,
        { container: mk() },
      ),
    ).toThrow(/bad "as" variable/)
  })

  it('cleans up on dispose', async () => {
    const { c, handle } = mountWith(
      `<data name="d" source="ship.fake()" live />
       {#async d}
         {:ok}<div class="ok">ready</div>
       {/async}`,
      { fake: () => live({ kind: 'ok', value: 1 }) },
    )
    await flush()
    expect(c.querySelector('.ok')).not.toBeNull()
    handle.dispose()
    c.remove()
    expect(document.body.contains(c)).toBe(false)
  })
})
