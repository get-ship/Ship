// @ship/sdk-client — browser SDK shim for ship-set views.
//
// Each domain method returns a `LiveSource` (see @ship/sui-runtime/live):
// the runtime's `live()` detects the shape, calls `subscribe(sig)` to
// open the patch stream, and the first SSE event flips the signal from
// `loading` to `ok`. The disposer returned by `subscribe` closes the
// EventSource on tear-down.
//
// Wire format: the daemon's `/_sui/data?expr=ship.<domain>.<op>()`
// endpoint streams `event: patch` frames whose data payload matches the
// runtime's Patch shape (currently always `{op: 'set', value: ...}`).
// Subsequent ops (`merge`, `append`, …) flow through `applyPatch` so a
// future server-side narrowing to per-row patches lights up without an
// SDK change.

import {
  type AgentsSummariesResponse,
  type GitDiff,
  type GitDiffArgs,
  type GitOverview,
  type LiveSignal,
  type LiveSource,
  type ShipSDK,
} from '@ship/sui-runtime'
// R3: applyPatch + Patch live at the /patch subpath now (the runtime
// barrel no longer re-exports them). sdk-client uses them to mutate
// the live signal in place when the daemon streams a partial patch
// (op !== 'set').
import { applyPatch, type Patch } from '@ship/sui-runtime/patch'

export type {
  AgentSummary,
  AgentsSummariesResponse,
  BranchSummary as GitBranch,
  CommitSummary as GitCommit,
  GitDiff,
  GitDiffArgs,
  GitOverview,
  StatusFile as GitFileEntry,
} from '@ship/sui-runtime'

/**
 * Public Ship SDK surface as exposed to view setups. Owned by
 * `@ship/sui-runtime` (so controllers see the full shape without an
 * extra import) and re-exported here for callers that prefer the
 * sdk-client entrypoint.
 */
export type { ShipSDK } from '@ship/sui-runtime'

/** Options accepted by `createClient`. */
export interface CreateClientOpts {
  /**
   * Daemon base URL — defaults to `''` (same-origin). Trailing slash is
   * trimmed so callers may pass either form.
   */
  baseUrl?: string
  /**
   * EventSource constructor injection — defaults to
   * `globalThis.EventSource`. Tests pass a fake.
   */
  EventSource?: typeof globalThis.EventSource
}

/**
 * Build a Ship SDK handle. The returned object carries one method per
 * domain; each call returns a `LiveSource<T>` ready to be passed to
 * `live()`.
 *
 * @example
 * ```ts
 * const ship = createClient()
 * const agents = live(() => ship.agents.summaries())
 * ```
 */
export function createClient(opts?: CreateClientOpts): ShipSDK {
  const baseUrl = (opts?.baseUrl ?? '').replace(/\/$/, '')
  const ESCtor = opts?.EventSource ?? globalThis.EventSource

  // Build the `/_sui/data?expr=...` SSE URL for a single op call. The
  // daemon parses the expr as `ship.<domain>.<op>(<json>?)`. Args are
  // emitted as a JSON object literal when present (same wire format
  // git.diff has used since the SDK landed).
  const sse = <T>(domain: string, op: string, args?: unknown): LiveSource<T> => {
    const argsLit = args === undefined ? '' : JSON.stringify(args)
    const expr = `ship.${domain}.${op}(${argsLit})`
    return makeSseSource<T>(
      `${baseUrl}/_sui/data?expr=${encodeURIComponent(expr)}`,
      ESCtor,
    )
  }

  return {
    agents: {
      summaries:   () => sse<AgentsSummariesResponse>('agents', 'summaries'),
      create_ctx:  () => sse('agents', 'create_ctx'),
      wire:      (args) => sse('agents', 'wire', args),
      detail:    (args) => sse('agents', 'detail', args),
      settings:  (args) => sse('agents', 'settings', args),
      catalogs:    () => sse('agents', 'catalogs'),
    },
    canvas: {
      doc:  (args) => sse('canvas', 'doc', args),
      list:    () => sse('canvas', 'list'),
    },
    chat: {
      state: (args) => sse('chat', 'state', args),
    },
    collections: {
      state:  (args) => sse('collections', 'state', args),
      detail: (args) => sse('collections', 'detail', args),
      items:  (args) => sse('collections', 'items', args),
      list:      () => sse('collections', 'list'),
    },
    dashboard: {
      state: () => sse('dashboard', 'state'),
    },
    events: {
      list: (args) => sse('events', 'list', args),
    },
    git: {
      overview:    (args?: { workspace_id?: string; limit?: number }) =>
        sse<GitOverview>('git', 'overview', args),
      diff:        (args: GitDiffArgs) => sse<GitDiff>('git', 'diff', args),
      annotations: (args: { file: string; workspace_id?: string }) =>
        sse('git', 'annotations', args),
    },
    mesh: {
      snapshot: () => sse('mesh', 'snapshot'),
    },
    settings: {
      state: () => sse('settings', 'state'),
    },
  }
}

/**
 * Wire one `_sui/data` SSE channel into a `LiveSource`. The first
 * `patch` event with `op === 'set'` flips the signal to ok; subsequent
 * events go through `applyPatch` against a façade that maps T <→ ok-state.
 *
 * Visible for tests via the package main barrel — production code uses
 * `createClient`.
 */
export function makeSseSource<T>(
  url: string,
  ESCtor: typeof globalThis.EventSource | undefined,
): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      if (!ESCtor) {
        sig.set({
          kind: 'error',
          error: new Error('[ship-set/sdk-client] no EventSource available'),
        })
        return () => {}
      }
      const facade = makeFacade<T>(sig)
      const es = new ESCtor(url)
      let firstSeen = false

      const onPatch = (ev: MessageEvent): void => {
        let patch: Patch
        try {
          patch = JSON.parse(ev.data) as Patch
        } catch (err) {
          sig.set({ kind: 'error', error: toError(err) })
          return
        }
        try {
          if (!firstSeen) {
            firstSeen = true
            // The first frame is always a full `set` from the resolver.
            // Use the value directly so the signal transitions cleanly
            // from loading → ok with the same patch flow as updates.
            if (patch.op === 'set' && (!patch.path || patch.path === '')) {
              sig.set({ kind: 'ok', value: patch.value as T })
            } else {
              applyPatch(facade, patch)
            }
          } else {
            applyPatch(facade, patch)
          }
        } catch (err) {
          sig.set({ kind: 'error', error: toError(err) })
        }
      }
      const onError = (): void => {
        if (!firstSeen) {
          sig.set({
            kind: 'error',
            error: new Error('[ship-set/sdk-client] SSE connection error'),
          })
        }
      }

      es.addEventListener('patch', onPatch as EventListener)
      es.addEventListener('error', onError)

      return () => {
        es.removeEventListener('patch', onPatch as EventListener)
        es.removeEventListener('error', onError)
        es.close()
      }
    },
  }
}

/**
 * Project a `LiveSignal<T>` into a plain `Signal<T>` view for `applyPatch`.
 * Reads return the inner ok value (undefined while loading/error); writes
 * transition to `{ kind: 'ok', value }`.
 */
function makeFacade<T>(sig: LiveSignal<T>): import('@ship/sui-runtime').Signal<T> {
  type Inner = import('@ship/sui-runtime').Signal<T>
  const reader = (() => {
    const v = sig()
    return v.kind === 'ok' ? v.value : (undefined as unknown as T)
  }) as Inner
  reader.set = (next) => {
    const cur = sig.__peek()
    const prev = cur.kind === 'ok' ? cur.value : (undefined as unknown as T)
    const v = typeof next === 'function' ? (next as (p: T) => T)(prev) : next
    sig.set({ kind: 'ok', value: v })
  }
  reader.__subs = sig.__subs
  reader.__peek = () => {
    const v = sig.__peek()
    return v.kind === 'ok' ? v.value : (undefined as unknown as T)
  }
  return reader
}

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))
