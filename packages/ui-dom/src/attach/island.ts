// Initial-state JSON island parser.
//
// Spec: docs/ship-set-framework/references/docs/ssr-hydration.md.
// Every SSR'd page emits a single `<script type="application/json"
// id="__set_init__">{...}</script>` after the rendered body. Keys are
// the names of `<state>` and `<data>` declarations: `<state>` values
// are JSON scalars/arrays/objects; `<data>` values are `LiveState`
// envelopes (`{kind: "loading" | "ok" | "error", …}`).
//
// `readInitIsland` parses that JSON, wraps every entry in a Signal
// (LiveSignal for envelopes), and reports which keys are state vs live
// so the resolver can pass `stateKeys` to a controller setup.

import { signal, type LiveSignal, type LiveState, type Signal } from '@ship/ui-runtime'

const INIT_ID = '__set_init__'

/** Result of reading the SSR JSON island. */
export interface InitIsland {
  /** Each entry wrapped in `signal()` (or `LiveSignal` for envelopes). */
  scope: Record<string, Signal<unknown>>
  /** Names of plain-state declarations (writable proxy targets). */
  stateKeys: Set<string>
  /** Names of `<data>` declarations whose initial value is a LiveState. */
  liveKeys: Set<string>
  /** Original LiveState envelopes — Phase E-2 SSE wiring reads this. */
  envelopes: Record<string, LiveState<unknown>>
  /**
   * SSE channel URL emitted by the compiler when the view has at least one
   * `<data live />` declaration. The runtime opens one EventSource on this
   * URL and routes `patch-signals` events by data name to the matching
   * LiveSignal. Absent when the view has no live data.
   */
  subscribe?: string
}

/**
 * Locate `<script id="__set_init__">` inside `root` (or its document)
 * and parse it. Throws a clear message when the island is missing —
 * `attach()` cannot proceed against an unrendered shell.
 */
export function readInitIsland(root: HTMLElement): InitIsland {
  const owner = root.ownerDocument ?? document
  let node: HTMLScriptElement | null = null
  if (typeof root.querySelector === 'function') {
    node = root.querySelector<HTMLScriptElement>(`script#${INIT_ID}`)
  }
  if (!node) {
    node = owner.getElementById(INIT_ID) as HTMLScriptElement | null
  }
  if (!node) {
    throw new Error(
      "attach() requires the SSR walker's __set_init__ JSON island; got an empty shell or stale HTML.",
    )
  }
  const raw = node.textContent ?? '{}'
  let parsed: Record<string, unknown>
  try {
    parsed = JSON.parse(raw) as Record<string, unknown>
  } catch (err) {
    throw new Error(
      '[ship-set/attach] failed to parse __set_init__ JSON: ' + (err as Error).message,
    )
  }

  const scope: Record<string, Signal<unknown>> = {}
  const stateKeys = new Set<string>()
  const liveKeys = new Set<string>()
  const envelopes: Record<string, LiveState<unknown>> = {}
  let subscribe: string | undefined

  for (const [k, v] of Object.entries(parsed)) {
    if (k === '__subscribe__') {
      if (typeof v === 'string') subscribe = v
      continue
    }
    if (isLiveStateEnvelope(v)) {
      const sig = signal<LiveState<unknown>>(v) as LiveSignal<unknown>
      scope[k] = sig as unknown as Signal<unknown>
      liveKeys.add(k)
      envelopes[k] = v
    } else {
      scope[k] = signal<unknown>(v)
      stateKeys.add(k)
    }
  }
  return { scope, stateKeys, liveKeys, envelopes, subscribe }
}

function isLiveStateEnvelope(v: unknown): v is LiveState<unknown> {
  if (!v || typeof v !== 'object') return false
  const k = (v as { kind?: unknown }).kind
  return k === 'loading' || k === 'ok' || k === 'error'
}
