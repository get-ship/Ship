# Sail runtime architecture

The browser-side library that compiled `.set` modules import from. Layer 1 of the stack — runs anywhere, no Ship dependency.

Target: ~12KB minified + gzipped. Solid-grade fine-grained reactivity. Patch-protocol native.

## Where it lives

```
packages/
└── @sail/runtime/                    # OSS, MIT-licensed, npm-published
    ├── src/
    │   ├── index.ts                  # public API surface
    │   ├── signal.ts                 # signal / derived / effect / batch
    │   ├── component.ts              # defineComponent + HTMLElement wrapper
    │   ├── bind.ts                   # bind directives (text/show/attr/class)
    │   ├── on.ts                     # event handler wiring
    │   ├── for.ts                    # keyed list reconciliation
    │   ├── slot.ts                   # light-DOM slot resolution
    │   ├── morph.ts                  # vendored idiomorph (or similar)
    │   ├── patch.ts                  # SSE patch protocol applicator
    │   ├── data.ts                   # data() resolver, SSE subscription
    │   └── decode.ts                 # prop attribute decoders
    ├── test/                         # vitest unit tests
    │   └── ...
    └── package.json
```

Single package, zero runtime dependencies. Vitest for unit; Playwright for integration (lives in the parent project, see [08-testing-benchmarks.md](08-testing-benchmarks.md)).

## Public API

What compiled component modules import from `@sail/runtime`:

```ts
// Reactivity primitives
export function signal<T>(initial: T): Signal<T>
export function derived<T>(fn: () => T): Derived<T>
export function effect(fn: () => void | (() => void), deps?: Signal<any>[]): () => void
export function batch(fn: () => void): void
export function untrack<T>(fn: () => T): T

// Component definition
export function defineComponent(tag: string, factory: ComponentFactory): void

// Bindings — used inside the compiled component class
export function bindText(el: Node, get: () => string, deps: Signal<any>[]): () => void
export function bindShow(el: Element, get: () => boolean, deps: Signal<any>[]): () => void
export function bindAttr(el: Element, name: string, get: () => any, deps: Signal<any>[]): () => void
export function bindClass(el: Element, name: string, get: () => boolean, deps: Signal<any>[]): () => void
export function bindValue(el: HTMLInputElement, sig: Signal<string>): () => void
export function bindFor<T>(anchor: Comment, items: () => T[], key: (item: T) => any, render: (item: T, i: number) => Node[], deps: Signal<any>[]): () => void
export function bindIf(anchor: Comment, cond: () => boolean, then_: () => Node[], else_: (() => Node[]) | null, deps: Signal<any>[]): () => void

// Events
export function on(el: Element, event: string, handler: EventListener, opts?: AddEventListenerOptions): () => void

// Slots (light-DOM)
export function resolveSlots(host: Element, fills: SlotFill[]): void

// Data (server-owned signals)
export function data<T>(opts: DataOpts<T>): Signal<T>
export interface DataOpts<T> {
  initial: T                         // SSR'd into HTML, hydrated here
  subscribe?: string                 // SSE channel for `live` data
  onPatch?: (patch: Patch) => void   // optional override
}

// Patch protocol
export function applyPatch(target: Signal<any>, patch: Patch): void
export interface Patch {
  op: 'set' | 'merge' | 'append' | 'prepend' | 'insert' | 'remove' | 'move' | 'splice'
  path?: string
  value?: any
  /* see 04-patch-protocol.md for full schema */
}

// Decoders (used by attribute → prop conversion)
export function decodeString(v: string | null, default_?: string): string
export function decodeNumber(v: string | null, default_?: number): number
export function decodeBoolean(v: string | null, default_?: boolean): boolean
export function decodeJson<T>(v: string | null, default_: T): T
```

That's the entire surface. ~25 exports.

## Signal engine

### Design

Solid-shaped fine-grained reactivity. Signals are the source of truth; derived are memoized; effects subscribe.

```ts
// signal.ts
type Subscriber = () => void

let CURRENT_OBSERVER: Subscriber | null = null
let BATCH_DEPTH = 0
const PENDING: Set<Subscriber> = new Set()

export interface Signal<T> {
  (): T                              // read
  set(value: T | ((prev: T) => T)): void
  __subs: Set<Subscriber>            // package-private
}

export function signal<T>(initial: T): Signal<T> {
  let value = initial
  const subs = new Set<Subscriber>()
  const sig = (() => {
    if (CURRENT_OBSERVER) subs.add(CURRENT_OBSERVER)
    return value
  }) as Signal<T>
  sig.set = (next) => {
    const v = typeof next === 'function' ? (next as Function)(value) : next
    if (Object.is(v, value)) return
    value = v
    if (BATCH_DEPTH > 0) {
      for (const s of subs) PENDING.add(s)
    } else {
      for (const s of subs) s()
    }
  }
  sig.__subs = subs
  return sig
}

export function derived<T>(fn: () => T): Derived<T> {
  let value: T
  let dirty = true
  const subs = new Set<Subscriber>()
  let trackedSigs: Set<Signal<any>> = new Set()

  const recompute = () => {
    const prev = CURRENT_OBSERVER
    CURRENT_OBSERVER = invalidate
    const oldSigs = trackedSigs
    trackedSigs = new Set()
    try {
      value = fn()
      dirty = false
    } finally {
      CURRENT_OBSERVER = prev
    }
    // Cleanup old subscriptions not needed anymore
    for (const s of oldSigs) if (!trackedSigs.has(s)) s.__subs.delete(invalidate)
  }

  const invalidate = () => {
    if (dirty) return
    dirty = true
    for (const s of subs) s()
  }

  const reader = (() => {
    if (CURRENT_OBSERVER) subs.add(CURRENT_OBSERVER)
    if (dirty) recompute()
    return value
  }) as Derived<T>
  reader.__subs = subs
  return reader
}

export function effect(fn: () => void | (() => void), deps?: Signal<any>[]): () => void {
  let cleanup: (() => void) | void
  const run = () => {
    if (cleanup) cleanup()
    const prev = CURRENT_OBSERVER
    CURRENT_OBSERVER = run
    try { cleanup = fn() } finally { CURRENT_OBSERVER = prev }
  }
  run()
  return () => { if (cleanup) cleanup() }
}

export function batch(fn: () => void): void {
  BATCH_DEPTH++
  try { fn() }
  finally {
    BATCH_DEPTH--
    if (BATCH_DEPTH === 0) {
      const subs = [...PENDING]
      PENDING.clear()
      for (const s of subs) s()
    }
  }
}

export function untrack<T>(fn: () => T): T {
  const prev = CURRENT_OBSERVER
  CURRENT_OBSERVER = null
  try { return fn() } finally { CURRENT_OBSERVER = prev }
}
```

~150 lines including types. Identical model to Solid's runtime. Tested with the same scenarios:
- Read tracks
- Write notifies
- Derived memoizes
- Cleanup on re-run
- Batched updates fire once
- Untrack escapes tracking

## Component wrapper

```ts
// component.ts
export type ComponentFactory = (host: HTMLElement, props: Record<string, Signal<any>>) => () => void

export function defineComponent(tag: string, factory: ComponentFactory): void {
  if (customElements.get(tag)) return
  class SailComponent extends HTMLElement {
    #cleanup?: () => void
    #propSigs: Record<string, Signal<any>> = {}

    static get observedAttributes() { /* generated per-component */ return [] }

    connectedCallback() {
      this.dataset.component = tag
      // Compiler-emitted code creates prop signals from attributes,
      // wires up bindings, runs lifecycle. The factory does this.
      this.#cleanup = factory(this, this.#propSigs)
    }

    disconnectedCallback() {
      this.#cleanup?.()
    }

    attributeChangedCallback(name: string, _: string, value: string) {
      const sig = this.#propSigs[camelize(name)]
      if (sig) sig.set(decodeAttr(name, value))
    }
  }
  customElements.define(tag, SailComponent)
}
```

The factory function is what the compiler emits per `.set` file. It gets the host element + a place to attach prop signals, returns a cleanup function.

Key design: **factory closures hold all user-script state.** No module-level state per component. Multiple instances of the same component each get their own closure.

## Bindings

Each binding type is a small function that:
1. Reads the current value
2. Applies it to the DOM
3. Subscribes to changes
4. Returns a cleanup

```ts
// bind.ts (excerpt)
export function bindText(el: Node, get: () => string, deps: Signal<any>[]): () => void {
  const apply = () => { el.textContent = get() }
  const dispose = effect(apply, deps)
  return dispose
}

export function bindShow(el: Element, get: () => boolean, deps: Signal<any>[]): () => void {
  const apply = () => { (el as HTMLElement).style.display = get() ? '' : 'none' }
  const dispose = effect(apply, deps)
  return dispose
}

export function bindAttr(el: Element, name: string, get: () => any, deps: Signal<any>[]): () => void {
  const apply = () => {
    const v = get()
    if (v == null || v === false) el.removeAttribute(name)
    else el.setAttribute(name, v === true ? '' : String(v))
  }
  return effect(apply, deps)
}
```

Compiler emits one `bind*` call per reactive binding in the template. Each subscribes individually, so `state x` updating only re-evaluates expressions that reference `x`.

## Keyed list reconciliation

The non-trivial one. `bindFor` handles keyed list updates with O(n) reconciliation:

```ts
// for.ts
interface ItemRecord<T> { key: any; item: T; nodes: Node[]; cleanup: () => void }

export function bindFor<T>(
  anchor: Comment,
  items: () => T[],
  keyFn: (item: T) => any,
  render: (item: T, i: number) => { nodes: Node[]; cleanup: () => void },
  deps: Signal<any>[],
): () => void {
  let prev: ItemRecord<T>[] = []

  const apply = () => {
    const next = items()
    const nextKeys = new Map(next.map((it, i) => [keyFn(it), { item: it, i }]))
    const prevKeys = new Map(prev.map(r => [r.key, r]))

    // 1. Remove items not in next
    for (const r of prev) {
      if (!nextKeys.has(r.key)) {
        for (const n of r.nodes) n.parentNode?.removeChild(n)
        r.cleanup()
      }
    }

    // 2. Build the new list, reusing existing items where possible
    const newRecords: ItemRecord<T>[] = []
    let cursor = anchor.nextSibling
    for (let i = 0; i < next.length; i++) {
      const item = next[i]
      const key = keyFn(item)
      const existing = prevKeys.get(key)
      if (existing) {
        // Reuse — move into position if needed
        for (const n of existing.nodes) {
          if (n !== cursor) anchor.parentNode!.insertBefore(n, cursor)
          else cursor = n.nextSibling
        }
        existing.item = item
        newRecords.push(existing)
      } else {
        // New — render and insert
        const r = render(item, i)
        const record = { key, item, nodes: r.nodes, cleanup: r.cleanup }
        for (const n of r.nodes) anchor.parentNode!.insertBefore(n, cursor)
        newRecords.push(record)
      }
    }

    prev = newRecords
  }

  const dispose = effect(apply, deps)
  return () => {
    dispose()
    for (const r of prev) {
      for (const n of r.nodes) n.parentNode?.removeChild(n)
      r.cleanup()
    }
  }
}
```

~50 lines. O(n) reconciliation. Stable nodes for unchanged keys (preserves focus, scroll, animation state).

## Slots (light-DOM)

```ts
// slot.ts
export interface SlotDecl {
  name: string
  anchor: Comment                    // placeholder where slot content goes
  fallback: () => Node[]             // rendered if no fill provided
}

export function resolveSlots(host: Element, slots: SlotDecl[]) {
  // Find children with `slot="X"` attribute, distribute them.
  const fills = new Map<string, Element[]>()
  for (const child of Array.from(host.children)) {
    const slotName = child.getAttribute('slot') || 'default'
    const list = fills.get(slotName) ?? []
    list.push(child)
    fills.set(slotName, list)
  }

  for (const slot of slots) {
    const filled = fills.get(slot.name) ?? []
    if (filled.length > 0) {
      for (const el of filled) {
        slot.anchor.parentNode!.insertBefore(el, slot.anchor)
      }
    } else {
      const fallbackNodes = slot.fallback()
      for (const n of fallbackNodes) slot.anchor.parentNode!.insertBefore(n, slot.anchor)
    }
  }

  // Remove any children that didn't match a slot (cleanup unattached fills)
  for (const child of Array.from(host.children)) {
    if (child.hasAttribute('slot')) {
      // Was distributed; remove the slot attribute now that it's placed
      child.removeAttribute('slot')
    }
  }
}
```

Light-DOM slot resolution is more permissive than shadow DOM — slot content lives in the host's actual subtree, accessible via querySelector, theme tokens cascade naturally.

## Data primitives

```ts
// data.ts
export function data<T>(opts: DataOpts<T>): Signal<T> {
  const sig = signal(opts.initial)
  if (opts.subscribe) {
    const sse = new EventSource(opts.subscribe)
    sse.addEventListener('patch', (ev) => {
      const patch = JSON.parse(ev.data) as Patch
      if (opts.onPatch) opts.onPatch(patch)
      else applyPatch(sig, patch)
    })
    // Cleanup on disconnect: handled by component cleanup chain
  }
  return sig
}
```

The compiler emits `data agents = ship.agents.list(...)` as a `data({ initial: SSR_VALUE, subscribe: '/api/v1/events?path=ship.agents.list&...' })` call.

## Patch applicator

See [04-patch-protocol.md](04-patch-protocol.md) for the protocol; the runtime side:

```ts
// patch.ts
export function applyPatch<T>(sig: Signal<T>, patch: Patch): void {
  const current = sig()
  const next = applyOp(current, patch)
  sig.set(next)
}

function applyOp(value: any, p: Patch): any {
  switch (p.op) {
    case 'set': return atPath(value, p.path, p.value, 'replace')
    case 'merge': return atPath(value, p.path, p.value, 'merge')
    case 'append': return atPath(value, p.path, p.value, 'append')
    case 'prepend': return atPath(value, p.path, p.value, 'prepend')
    case 'insert': return atPath(value, p.path, p.value, 'insert', p.index)
    case 'remove': return atPath(value, p.path, undefined, 'remove')
    case 'move': return atPath(value, p.path, undefined, 'move', p.from, p.to)
    case 'splice': return atPath(value, p.path, p.value, 'splice', p.index, p.count)
    default: throw new Error(`unknown patch op: ${(p as any).op}`)
  }
}

// atPath walks a JSON path (e.g., "shapes[7].color") and applies
// the operation immutably. For arrays, returns a new array; for
// objects, returns a new object. Sail signals always replace value
// (Object.is comparison) so subscribers fire on real change.
```

~80 lines. JSON-path resolution is the meaty bit; everything else is straightforward.

## Morph engine

For full HTML element patches (the `element:patch` SSE event, less common than signal patches but supported for hypermedia views).

Vendor [idiomorph](https://github.com/bigskysoftware/idiomorph) (~500 lines, MIT). Use it for `applyElementPatch(target, htmlString, opts)` — replaces `target`'s subtree with the morphed result of parsing `htmlString`.

```ts
// morph.ts
import { Idiomorph } from './idiomorph.vendored'

export function applyElementPatch(target: Element, html: string, opts?: { focus?: 'preserve' | 'reset' }): void {
  // Idiomorph diffs old vs new, applies minimal mutations
  Idiomorph.morph(target, html, {
    morphStyle: 'innerHTML',
    callbacks: {
      beforeNodeRemoved: (oldNode) => {
        // Trigger Sail cleanup hooks if oldNode is a Sail component
      },
    },
  })
}
```

## SSE wiring

The runtime opens one EventSource per `data live` declaration (or one shared one if optimization warrants — TBD). Patches arrive as named events:

```
event: signal:patch
data: {"signal":"_my_component_3_agents","op":"append","value":{"id":"x"}}

event: element:patch
data: {"target":"#main","html":"..."}
```

The runtime dispatches based on event name to either `applyPatch` (for signal patches) or `applyElementPatch` (for element patches).

## Performance characteristics

- **Signal read:** ~50ns (function call + observer dependency tracking)
- **Signal write that triggers 1 subscriber:** ~200ns (notification + apply)
- **Effect fan-out 1→N subscribers:** ~50ns per subscriber + the work each does
- **Patch parse + apply (single signal):** ~100μs for typical patch (1KB JSON, simple path)
- **Keyed list reconciliation, 1000 items, 1 changed:** ~2ms (mostly key comparison + 1 DOM mutation)
- **Component mount:** ~5ms typical, ~20ms for complex components with many bindings (one-time cost)

These are rough; real numbers come from the benchmark in [08-testing-benchmarks.md](08-testing-benchmarks.md).

## Bundle size

Target 12KB minified + gzipped for the runtime. Breakdown:

- signal.ts: ~3KB
- bind.ts + on.ts: ~2KB
- for.ts: ~1KB
- slot.ts: ~0.5KB
- patch.ts + atPath: ~2KB
- morph.ts (vendored idiomorph minified): ~3KB
- data.ts + decode.ts + component.ts: ~1.5KB

Total: ~13KB. Tighten if needed.

For comparison: Datastar v1 is ~30KB. Lit is ~5KB but doesn't include reactivity primitives. Solid is ~7KB. Svelte 5 runtime is ~12KB.

We're in the right neighborhood.

## Test plan

Vitest unit tests in `packages/@sail/runtime/test/`:

- `signal.test.ts` — read tracks, write notifies, derived memoizes, batch, untrack, cycle detection
- `bind.test.ts` — each binding directive's apply behavior + cleanup on unsubscribe
- `for.test.ts` — keyed reconciliation with insert/remove/move/swap; node identity preserved across reorder
- `slot.test.ts` — slot fill, fallback, distribution
- `patch.test.ts` — every op type, JSON path edge cases, immutability
- `component.test.ts` — defineComponent, attribute → signal, lifecycle, cleanup chain

Each test runs in jsdom + a small Sail-runtime harness. <50 tests total at the runtime level. Integration tests (browser-rendered) live in the Sail-using project (Ship, in our case) and exercise the whole stack.

## Versioning

Semver. v1 is the public API surface above. Breaking changes go through deprecation cycles + migration tool support.

## Implementation phasing

- **Day 1-2:** signal.ts (the core engine). Heavy unit tests.
- **Day 3:** bind.ts + on.ts (basic bindings). 
- **Day 4:** component.ts (custom-element wrapper) + decode.ts.
- **Day 5:** for.ts + slot.ts.
- **Day 6-7:** patch.ts + JSON path. data.ts + SSE wiring.
- **Day 8:** morph.ts (vendor + integrate idiomorph).
- **Day 9-10:** integration with the compiler emit, first end-to-end Sail component in a real browser.

~10 days for the runtime. Most of the time is testing.

## What this gives us

A small, fast, fine-grained reactive runtime — Solid-grade — that knows how to:
- Mount + lifecycle custom elements
- Handle keyed list reconciliation
- Resolve light-DOM slots
- Apply patches over SSE
- Hydrate from server-rendered HTML

It doesn't know what Ship is. That's intentional — Layer 3 (Ship adapter, [05-server-data-bridge.md](05-server-data-bridge.md)) plugs in via the `subscribe` URL on data declarations.

Open source, npm-published as `@sail/runtime`.
