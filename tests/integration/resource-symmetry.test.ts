// @vitest-environment jsdom
//
// Resource-symmetry integration test.
//
// Goal: every checked-in view, when mounted in jsdom and immediately
// disposed, leaks zero EventSources, zero MutationObservers, zero
// ResizeObservers, zero IntersectionObservers, and zero setIntervals.
//
// This catches the leak class fixed in two recent commits:
//   - 87cf3223 — `live()` discarded its effect's stop fn, so the
//     EventSource opened by a `LiveSource.subscribe(sig)` was never
//     closed; `<data live>` views OOM'd tabs after a few-dozen
//     navigations.
//   - 0efea6d7 — `mount()` was dropping the `() => …` cleanup that
//     controllers return from their `mount()` lifecycle hook.
//     Pre-fix, every controller setting up a MutationObserver /
//     ResizeObserver / setInterval / addEventListener tied to the
//     host leaked the resource on every navigation.
//
// Two halves:
//   1. Synthetic leak-class scenarios that directly target the bug
//      shapes named above. These are the most precise regression
//      gates — a future revert of either commit fails them.
//   2. A per-view smoke loop: every controller in `views/<name>/`
//      (and the named subviews) is mounted via the runtime's
//      `mount()` with a synthetic compiled-module wrapper, then
//      disposed. Catches the broader class of "some new view
//      allocates a resource and forgets to release it on dispose."
//
// See `.artifacts/job-spec.md` § Part B for the full design.

import { describe, expect, it, beforeEach, afterEach } from 'vitest'
import {
  mount,
  live,
  signal,
  type LiveSource,
  type LiveState,
  type ViewPayload,
} from '@ship/sui-runtime'
import {
  installWrappers,
  snapshot,
  openSince,
  type InstalledWrappers,
  type ResourceCounters,
} from './_fixtures/resource-counters'

let wrappers: InstalledWrappers
let counters: ResourceCounters

beforeEach(() => {
  document.body.innerHTML = ''
  wrappers = installWrappers()
  counters = wrappers.counters
})

afterEach(() => {
  wrappers.restore()
})

const tick = () => new Promise<void>((r) => setTimeout(r, 0))

function makeContainer(): HTMLElement {
  const el = document.createElement('main')
  document.body.appendChild(el)
  return el
}

interface SimpleModule {
  default: {
    setup: (ctx: unknown) => Record<string, unknown>
  }
}

/** Convenience: build a minimal payload with empty SSR + no scope. */
function emptyPayload(name = 'test'): ViewPayload {
  return { name, html: '<div></div>', css: '', scope: {}, moduleUrl: '/test' }
}

describe('resource-symmetry — synthetic leak-class scenarios', () => {
  it('live(LiveSource) closes its EventSource on dispose [87cf3223]', async () => {
    // Synthetic LiveSource that opens an EventSource on subscribe and
    // closes it on disposer. live() must register the disposer with
    // the mount's effect scope; a regression of 87cf3223 would skip
    // that registration and the EventSource would leak.
    const before = snapshot(counters)
    const target = makeContainer()
    const mod: SimpleModule = {
      default: {
        setup: () => {
          const sig = live<string>(() => ({
            subscribe: (out) => {
              const es = new EventSource('/test/sse')
              es.addEventListener('message', (e) => {
                out.set({ kind: 'ok', value: String((e as MessageEvent).data) } as LiveState<string>)
              })
              return () => es.close()
            },
          }))
          return { sig }
        },
      },
    }
    const handle = await mount(target, emptyPayload('synthetic-live'), { module: mod })
    // Verify the wrapping worked — at least one EventSource opened.
    expect(counters.EventSource.created - before.EventSource.created).toBeGreaterThanOrEqual(1)
    handle.dispose()
    await tick()
    const leaks = openSince(before, snapshot(counters))
    expect(leaks).toEqual({})
  })

  it('controller mount() returning () => void runs the disposer [0efea6d7 sync]', async () => {
    const before = snapshot(counters)
    const target = makeContainer()
    let intervalId: ReturnType<typeof setInterval> | undefined
    const mod: SimpleModule = {
      default: {
        setup: () => ({
          __shipSetLifecycle: {
            state: {},
            mount: () => {
              intervalId = setInterval(() => {}, 60_000)
              return () => clearInterval(intervalId)
            },
          },
        }),
      },
    }
    const handle = await mount(target, emptyPayload('synthetic-sync'), { module: mod })
    handle.dispose()
    await tick()
    const leaks = openSince(before, snapshot(counters))
    expect(leaks).toEqual({})
  })

  it('controller mount() returning Promise<() => void> runs the disposer [0efea6d7 async]', async () => {
    const before = snapshot(counters)
    const target = makeContainer()
    const mod: SimpleModule = {
      default: {
        setup: () => ({
          __shipSetLifecycle: {
            state: {},
            mount: async () => {
              // Yield once so the promise actually trips the async-mount branch.
              await Promise.resolve()
              const id = setInterval(() => {}, 60_000)
              return () => clearInterval(id)
            },
          },
        }),
      },
    }
    const handle = await mount(target, emptyPayload('synthetic-async'), { module: mod })
    // Wait for the async mount to resolve before disposing.
    await Promise.resolve()
    await tick()
    handle.dispose()
    await tick()
    const leaks = openSince(before, snapshot(counters))
    expect(leaks).toEqual({})
  })

  it('controller mount() that allocates a MutationObserver releases it on dispose', async () => {
    const before = snapshot(counters)
    const target = makeContainer()
    const mod: SimpleModule = {
      default: {
        setup: () => ({
          __shipSetLifecycle: {
            state: {},
            mount: (_state, el) => {
              const mo = new MutationObserver(() => {})
              mo.observe(el, { childList: true })
              return () => mo.disconnect()
            },
          },
        }),
      },
    }
    const handle = await mount(target, emptyPayload('synthetic-mo'), { module: mod })
    handle.dispose()
    await tick()
    const leaks = openSince(before, snapshot(counters))
    expect(leaks).toEqual({})
  })

  it('payload __subscribe__ opens then closes the live SSE channel', async () => {
    const before = snapshot(counters)
    const target = makeContainer()
    const payload: ViewPayload = {
      name: 'live-channel',
      html: '<div></div>',
      css: '',
      scope: {
        __subscribe__: '/test/sse-channel',
        agents: { kind: 'loading' },
      },
      moduleUrl: '/test',
    }
    const mod: SimpleModule = { default: { setup: () => ({}) } }
    const handle = await mount(target, payload, { module: mod })
    expect(counters.EventSource.created - before.EventSource.created).toBe(1)
    handle.dispose()
    await tick()
    const leaks = openSince(before, snapshot(counters))
    expect(leaks).toEqual({})
  })
})

// ── Per-view smoke loop ────────────────────────────────────────────────
//
// For every controller paired with a `.sui` template under `views/`, we
// build a synthetic compiled-module wrapper that mirrors what the real
// compiler emits — `setup(ctx)` runs the controller's resolveController,
// merges the resulting bindings into a flat scope, and exposes any
// `__shipSetLifecycle` so `mount()` finds it. We mount with a minimal
// payload + ship stub, dispose, and assert symmetry.
//
// This doesn't drive every code path each view uses (no template
// rendering, no derived signal exercise) — that's covered by per-view
// vitest suites elsewhere. Here we gate the lifecycle's resource
// hygiene specifically.

interface ViewFixture {
  name: string
  controller: () => Promise<unknown>
  /** Initial scope keys + values fed into the synthetic mount payload. */
  scope: Record<string, unknown>
  /** Optional ship stub. Provided when the controller derefs ship.* during setup. */
  ship?: unknown
}

const noopShip = new Proxy(
  {},
  {
    get(): unknown {
      // Recursive proxy: any access path returns another no-op proxy
      // until called as a function — controllers that reach into
      // ship.git.overview() during setup get a no-op LiveSource.
      return new Proxy(
        function fakeCallable() {
          return { subscribe: () => () => {} }
        },
        {
          get() {
            return noopShip
          },
        },
      )
    },
  },
)

const VIEW_FIXTURES: ViewFixture[] = [
  {
    name: 'counter',
    controller: () => import('../../views/counter/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    name: 'agents',
    controller: () => import('../../views/agents/controller'),
    scope: { agents: { kind: 'loading' } as LiveState<unknown> },
    ship: noopShip,
  },
  {
    name: 'git',
    controller: () => import('../../views/git/controller'),
    scope: { overview: { kind: 'loading' } as LiveState<unknown> },
    ship: noopShip,
  },
  {
    name: 'home',
    controller: () => import('../../views/home/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    name: 'demos/signals-and-derived',
    controller: () => import('../../views/demos/signals-and-derived/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    name: 'demos/match',
    controller: () => import('../../views/demos/match/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    name: 'demos/conditional-render',
    controller: () => import('../../views/demos/conditional-render/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    name: 'demos/iteration',
    controller: () => import('../../views/demos/iteration/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    // The lifecycle demo is the canonical reproducer for bug 0efea6d7:
    // its `mount: (state) => { const t = setInterval(...); return () =>
    // clearInterval(t) }` shape leaks the interval if the runtime drops
    // the returned disposer. Specifically gating it here makes sure the
    // sensitivity to that exact regression is covered.
    name: 'demos/lifecycle',
    controller: () => import('../../views/demos/lifecycle/controller'),
    scope: {},
    ship: noopShip,
  },
  {
    name: 'studio-components',
    controller: () => import('../../views/studio/components/controller'),
    scope: {},
    ship: noopShip,
  },
]

interface ControllerExportLike {
  setup: (ctx: unknown) => Record<string, unknown>
}

describe('resource-symmetry — every checked-in view mounts + disposes clean', () => {
  for (const fx of VIEW_FIXTURES) {
    it(`views/${fx.name} → mount + dispose leaks nothing`, async () => {
      const mod = (await fx.controller()) as { default: ControllerExportLike | null }
      const ctrlExport = mod.default
      const before = snapshot(counters)
      const target = makeContainer()
      const setup = (ctx: unknown): Record<string, unknown> => {
        const r = ctrlExport ? ctrlExport.setup(ctx) : {}
        // Mirror what the compiler emits: merge controller bindings
        // into a $scope and return it. mount() reads
        // __shipSetLifecycle directly off the return value.
        return { ...r }
      }
      const handle = await mount(
        target,
        {
          name: fx.name,
          html: '<div></div>',
          css: '',
          scope: fx.scope,
          moduleUrl: `/test/${fx.name}`,
        },
        { module: { default: { setup } }, ship: fx.ship },
      )
      // Let any async mount lifecycle settle before disposing.
      await tick()
      handle.dispose()
      await tick()
      const leaks = openSince(before, snapshot(counters))
      expect(leaks, `views/${fx.name} leaked: ${JSON.stringify(leaks)}`).toEqual({})
    })
  }
})
