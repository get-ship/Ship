// Reactive `<data>` arg re-subscribe.
//
// When a `<data live>` source thunk reads a state signal via `$scope`,
// `live()` tracks that read; on signal change the prior subscription
// disposes + a new one opens with the new args. This test pins the
// behavior end-to-end through hydrate.

import { afterEach, describe, expect, it } from 'vitest'
import { hydrate } from '../src/legacy'
import type { LiveSignal, LiveState } from '@ship/sui-runtime'

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

interface TrackingSource<T> {
  /** Args supplied each time the source thunk runs. */
  argsPerCall: Array<unknown>
  /** Disposes called per subscribe. */
  disposeCalls: number
  /** Subscribers called per subscribe. */
  subscribeCalls: number
  /** Build a `LiveSource<T>` that records its args + lifecycle. */
  build(args: unknown, value: T): {
    subscribe: (sig: LiveSignal<T>) => () => void
  }
}

function makeTrackingSource<T>(): TrackingSource<T> {
  const tracker: TrackingSource<T> = {
    argsPerCall: [],
    disposeCalls: 0,
    subscribeCalls: 0,
    build(args, value) {
      tracker.argsPerCall.push(args)
      return {
        subscribe(sig) {
          tracker.subscribeCalls += 1
          sig.set({ kind: 'ok', value } as LiveState<T>)
          return () => {
            tracker.disposeCalls += 1
          }
        },
      }
    },
  }
  return tracker
}

describe('<data live> reactive args', () => {
  it('re-fires source + reopens subscription when a state signal in $scope changes', async () => {
    const container = mkContainer()
    const tracker = makeTrackingSource<{ id: string; name: string }>()
    const ship = {
      agents: {
        get: (args: { id: string }) =>
          tracker.build(args, { id: args.id, name: `Agent ${args.id}` }),
      },
    }

    const template = `
      <state name="selectedId" value="'a'" />
      <data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />

      {#if agent().kind === 'ok'}
        <p data-test="name">{agent().value.name}</p>
      {/if}
    `

    const handle = hydrate(template, undefined, { container, ship })
    await flush()

    expect(tracker.subscribeCalls).toBe(1)
    expect(tracker.argsPerCall).toEqual([{ id: 'a' }])
    expect(container.querySelector('[data-test="name"]')!.textContent).toBe('Agent a')

    // Mutate the state signal — `selectedId.set('b')` from outside
    // through the rendered tree's effect graph. Easiest hook: the
    // template scope exposed via render. Instead, drive via a button
    // in a richer template:
    handle.dispose()

    const container2 = mkContainer()
    const tracker2 = makeTrackingSource<{ id: string; name: string }>()
    const ship2 = {
      agents: {
        get: (args: { id: string }) =>
          tracker2.build(args, { id: args.id, name: `Agent ${args.id}` }),
      },
    }
    const template2 = `
      <state name="selectedId" value="'a'" />
      <data name="agent" source="ship.agents.get({id: $scope.selectedId()})" live />

      <button data-test="next" on:click={() => selectedId.set('b')}>next</button>
      {#if agent().kind === 'ok'}<p data-test="name">{agent().value.name}</p>{/if}
    `
    const h2 = hydrate(template2, undefined, { container: container2, ship: ship2 })
    await flush()
    expect(tracker2.subscribeCalls).toBe(1)
    expect(tracker2.argsPerCall.at(-1)).toEqual({ id: 'a' })

    ;(container2.querySelector('[data-test="next"]') as HTMLButtonElement).click()
    await flush()

    // Source re-fired; old subscription disposed; new one opened.
    expect(tracker2.disposeCalls).toBe(1)
    expect(tracker2.subscribeCalls).toBe(2)
    expect(tracker2.argsPerCall.at(-1)).toEqual({ id: 'b' })
    expect(container2.querySelector('[data-test="name"]')!.textContent).toBe('Agent b')

    h2.dispose()
  })

  it('source can read state declared in <state> via $scope', async () => {
    const container = mkContainer()
    const calls: Array<unknown> = []
    const ship = {
      x: { call: (args: unknown) => {
        calls.push(args)
        return {
          subscribe: (sig: LiveSignal<unknown>) => {
            sig.set({ kind: 'ok', value: args })
            return () => {}
          },
        }
      } },
    }
    const template = `
      <state name="q" value="'init'" />
      <data name="r" source="ship.x.call({q: $scope.q()})" live />
      <p data-test="r">{r().kind === 'ok' ? JSON.stringify(r().value) : 'pending'}</p>
    `
    const handle = hydrate(template, undefined, { container, ship })
    await flush()
    expect(calls).toEqual([{ q: 'init' }])
    expect(container.querySelector('[data-test="r"]')!.textContent).toBe('{"q":"init"}')
    handle.dispose()
  })
})
