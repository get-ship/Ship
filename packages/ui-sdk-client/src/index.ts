// @ship/ui-sdk-client — browser SDK shim for ship-set views.
//
// Each domain method returns a `LiveSource` (see @ship/ui-runtime/live):
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

import { applyPatch, type LiveSignal, type LiveSource, type Patch } from '@ship/ui-runtime'

/** Public Ship SDK surface as exposed to view setups. */
export interface ShipSDK {
  agents: {
    /**
     * Stream of agent summaries — list-view payload. Subscribers receive
     * the full list as the first patch and per-change patches afterward.
     */
    summaries(): LiveSource<AgentsSummariesResponse>
  }
  git: {
    /**
     * Repo overview — branch + head + clean/staged/modified/untracked
     * file lists + recent commits + branches + ahead/behind. Fetched
     * once on subscribe and re-fetched whenever the daemon emits a
     * relevant change event.
     */
    overview(): LiveSource<GitOverview>
    /**
     * Unified diff for a specific commit (or the special `'WORKING'`
     * value for the working-tree diff). Args are required.
     */
    diff(args: GitDiffArgs): LiveSource<GitDiff>
  }
}

export interface GitDiffArgs {
  /** Commit SHA, or `'WORKING'` for the working-tree diff. */
  commit: string
  /** Optional workspace scope (defaults to the daemon's project root). */
  workspace_id?: string
}

export interface GitDiff {
  sha: string
  short_sha: string
  is_working: boolean
  diff_text: string
  parsed: unknown
}

/** Shape of `agents.summaries()` payload — matches the daemon registry. */
export interface AgentsSummariesResponse {
  items: AgentSummary[]
}

/** Repo-overview payload. Mirrors `services::registry::"git.overview"`. */
export interface GitOverview {
  workspace_id: string | null
  branch: string
  head_short: string
  clean: boolean
  total_changed: number
  staged: GitFileEntry[]
  modified: GitFileEntry[]
  untracked: GitFileEntry[]
  commits: GitCommit[]
  branches: GitBranch[]
  ahead: number
  behind: number
}

export interface GitFileEntry {
  path: string
  status?: string
}

export interface GitCommit {
  sha: string
  short_sha: string
  subject: string
  author: string
  date: string
  additions: number
  deletions: number
}

export interface GitBranch {
  name: string
  active: boolean
  upstream?: string
  ahead: number
  behind: number
}

/** One agent card's worth of data. Mirrors `services::agents::view_state::AgentSummary`. */
export interface AgentSummary {
  id: string
  name: string
  description: string
  skills_count: number
  providers_count: number
  mcp_count: number
  rules_count: number
  letter: string
  source: string
  providers: string[]
  tags: string[]
  active: boolean
}

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

  return {
    agents: {
      summaries: () => makeSseSource<AgentsSummariesResponse>(
        `${baseUrl}/_sui/data?expr=${encodeURIComponent('ship.agents.summaries()')}`,
        ESCtor,
      ),
    },
    git: {
      overview: () => makeSseSource<GitOverview>(
        `${baseUrl}/_sui/data?expr=${encodeURIComponent('ship.git.overview()')}`,
        ESCtor,
      ),
      diff: (args) => makeSseSource<GitDiff>(
        `${baseUrl}/_sui/data?expr=${encodeURIComponent(`ship.git.diff(${JSON.stringify(args)})`)}`,
        ESCtor,
      ),
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
function makeFacade<T>(sig: LiveSignal<T>): import('@ship/ui-runtime').Signal<T> {
  type Inner = import('@ship/ui-runtime').Signal<T>
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
