import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  defineRoutes,
  live,
  navigate,
  type LiveSignal,
  type RouteSignals,
} from '@ship-set/runtime'
import { attach } from '../../src/attach'
import { clearContainers, makeModule, mountSsr, flush } from './helpers'

beforeEach(() => {
  window.history.replaceState({}, '', '/')
})

afterEach(() => {
  window.history.replaceState({}, '', '/')
  clearContainers()
})

describe('attach/route — reactive params', () => {
  it('live() source thunk re-fires when route.params.id changes', async () => {
    defineRoutes([{ pattern: '/agents/:id' }])
    window.history.replaceState({}, '', '/agents/1')

    // SSR'd HTML is irrelevant for this test — we drive the live
    // signal via the controller's setup return rather than a real
    // <data live> directive (which the AOT compiler would otherwise
    // wire). The init island still has to be present so attach()
    // doesn't throw on the missing JSON script.
    const root = mountSsr('<p data-test="x"></p>', {})

    const calls: Array<string | undefined> = []
    let routeRef: RouteSignals | undefined
    let liveSig: LiveSignal<string> | undefined
    const fakeShip = {
      agents: {
        get: ({ id }: { id: string | undefined }) => {
          calls.push(id)
          return Promise.resolve(`agent:${id}`)
        },
      },
    }

    const mod = makeModule((ctx) => {
      routeRef = ctx.route
      const ship = ctx.ship as typeof fakeShip
      liveSig = live(() => ship.agents.get({ id: ctx.route!.params.id() }))
      return { agent: liveSig }
    }, [])

    const dispose = attach(root, { module: mod, ship: fakeShip })
    expect(routeRef).toBeDefined()
    expect(routeRef!.params.id()).toBe('1')
    await flush()
    await flush()
    expect(calls).toEqual(['1'])
    const v1 = liveSig!()
    expect(v1.kind).toBe('ok')
    if (v1.kind === 'ok') expect(v1.value).toBe('agent:1')

    navigate('/agents/2')
    expect(routeRef!.params.id()).toBe('2')
    expect(liveSig!().kind).toBe('loading')
    await flush()
    await flush()
    expect(calls).toEqual(['1', '2'])
    const v2 = liveSig!()
    expect(v2.kind).toBe('ok')
    if (v2.kind === 'ok') expect(v2.value).toBe('agent:2')

    dispose()
  })

  it('attach() with no route option seeds RouteSignals from window.location', () => {
    window.history.replaceState({}, '', '/dashboard?tab=overview')
    const root = mountSsr('<p></p>', {})
    let routeRef: RouteSignals | undefined
    const mod = makeModule((ctx) => {
      routeRef = ctx.route
      return {}
    }, [])
    const dispose = attach(root, { module: mod })
    expect(routeRef!.pathname()).toBe('/dashboard')
    expect(routeRef!.query.tab()).toBe('overview')
    dispose()
  })

  it('legacy { params, query } record is upgraded to RouteSignals', () => {
    const root = mountSsr('<p></p>', {})
    let routeRef: RouteSignals | undefined
    const mod = makeModule((ctx) => {
      routeRef = ctx.route
      return {}
    }, [])
    const dispose = attach(root, {
      module: mod,
      route: { params: { id: 'seeded' }, query: { q: 'hi' } },
    })
    // The lazy proxy materialises a Signal (callable) for each known key.
    expect(typeof routeRef!.params.id).toBe('function')
    // The legacy record-shape's values land on the lazy signals.
    expect(routeRef!.params.id()).toBe('seeded')
    expect(routeRef!.query.q()).toBe('hi')
    dispose()
  })
})
