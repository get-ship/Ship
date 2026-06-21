// @ship/sdk-client — browser SDK shim for ship-set views.
//
// Each domain method returns either a `LiveSource` (read — opens an SSE
// channel to `/_deck/data?expr=ship.<domain>.<op>(<args>)`) or a Promise
// (write — POST/PUT/PATCH/DELETE against the daemon's REST surface).
// The runtime's `live()` detects the source shape, calls `subscribe(sig)`
// to open the patch stream, and the first SSE event flips the signal
// from `loading` to `ok`. The disposer returned by `subscribe` closes
// the EventSource on tear-down.
//
// HTTP plumbing lives in `./http.ts`; the SSE bridge in `./sse.ts`.
// Keeping `index.ts` to the surface declaration only respects the 300-
// line file cap and makes the SDK surface readable at a glance.
import type { LiveSource } from '@ship/sui-runtime'
import type { AgentsSummariesResponse } from './generated'
import type { GitDiff, GitDiffArgs, GitOverview } from './ship-data'
import type { ArtifactRecord, ShipSDK } from './ship-sdk'
import { fetchBody, mutateForm, mutateJson } from './http'
import { sharedSource } from './shared-source'
import { readSpaceConfig } from './spaces'
import { MuxClient } from './mux-client'
import { liveStream } from './live-stream'
import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'
// Re-export the URL constants from `./constants` so the existing surface
// (`import { DECK_PREFIX, DECK_API_ASSETS } from '@ship/sdk-client'`) keeps
// working unchanged. The mint point lives in `./constants`; that module's
// source is what `ts_constants_pinned_to_rust` (in
// `crates/deck/deck-core/src/routes.rs`) reads to assert Rust↔TS parity.
export { DECK_API_ASSETS, DECK_PREFIX } from './constants'

// Bind the SUI runtime's host-agnostic `ShipHost` slot to the concrete
// `ShipSDK` via declaration merging. The runtime types `ViewScope.ship` as
// the empty `ShipHost`; this augmentation — owned by the SDK provider, not
// the framework — makes `ship.<domain>.<op>()` fully typed in controllers
// while the runtime stays free of any Ship concept. (When the SDK surface
// moves to the generated connect client, this augmentation moves with it.)
declare module '@ship/sui-runtime' {
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  interface ShipHost extends ShipSDK {}
}

export { liveStream } from './live-stream'
export { sharedSource } from './shared-source'
export { readSpaceConfig } from './spaces'
export type { SpaceConfig, SpaceRuntimeRoute } from './spaces'
export {
  displayName as accountDisplayName,
  initials as accountInitials,
  openBilling as openAccountBilling,
  planLabel as accountPlanLabel,
  readIdentity as readAccountIdentity,
  signIn as accountSignIn,
  signOut as accountSignOut,
  type AccountIdentity,
  type AccountStatus,
} from './account'

// Generated daemon data types — sourced from the SDK layer's own generated.ts
// (gen-types output), NOT the framework. @ship/sui-runtime names no Ship type.
export type {
  AgentSummary,
  AgentsSummariesResponse,
} from './generated'
// Hand-owned SSE/mux payload shapes — their Rust structs left the specta
// surface when the registered wire moved to proto (see ship-data.ts).
export type {
  BranchSummary as GitBranch,
  CommitSummary as GitCommit,
  GitDiff,
  GitDiffArgs,
  GitOverview,
  StatusFile as GitFileEntry,
} from './ship-data'
// The Ship SDK surface + its hand-authored data types — owned here, not the framework.
export type {
  AccountState,
  ArtifactRecord,
  CollectionFieldRecord,
  CollectionSummary,
  ShipSDK,
  SkillIndexEntry,
} from './ship-sdk'

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
  /**
   * Connect client injection for tests — defaults to real client.
   * Tests pass a fake to control RPC responses and live streams.
   */
  connectClient?: unknown
}

/**
 * Build a Ship SDK handle. The returned object carries one method per
 * domain; each call returns either a `LiveSource<T>` ready to be passed
 * to `live()` or a Promise for mutation ops.
 *
 * @example
 * ```ts
 * const ship = createClient()
 * const agents = live(() => ship.agents.summaries())
 * await ship.agents.create({ id: 'foo', doc: { ... } })
 * ```
 */
// ── Shared singleton (zero-config) ─────────────────────────────────────────
//
// `ship-*` packs (e.g. @ship/agents, @ship/pipelines) reach for the SDK from
// inside a web component — below the view tier where ViewScope supplies `ship`.
// They just `import { ship }` and call it: the singleton lazily binds a
// same-origin client on first use, so there is NO host bootstrap step. Using
// our own SDK is one import, no config, no "remember to initialize" — the daemon
// serves both the SPA and the API, so same-origin is always right.
//
// A host that needs a non-default transport (custom baseUrl, an injected
// EventSource, a test fake) calls `setShipClient(...)` before first use — an
// optional override, not a required ceremony. deck names no ship concept and
// emits no init; the default lives here, in the SDK. Pure `sui-*` packs never
// reach for `ship` and never bind a client.

let _shipClient: ShipSDK | null = null

/** Override the SDK client (optional). Tests inject a fake; a host on a
 *  non-same-origin transport sets a custom-`baseUrl` client. Call before the
 *  first `ship.*` access. Left unset, `getShipClient` lazily binds a same-origin
 *  client — the common case needs no call at all. */
export function setShipClient(client: ShipSDK): void {
  _shipClient = client
}

/** Read the SDK client (rarely needed directly — prefer `ship`). Lazily binds a
 *  same-origin client on first use, so `import { ship }` works with zero
 *  ceremony and no host bootstrap. */
export function getShipClient(): ShipSDK {
  if (!_shipClient) {
    // Zero-config default: same-origin. The daemon serves the SPA and the API
    // together, so `baseUrl: ''` is correct for every first-party host (hub,
    // shell, spaces). No setShipClient ceremony, no deck coupling.
    _shipClient = createClient({ baseUrl: '' })
  }
  return _shipClient
}

/** The SDK singleton for `ship-*` packs. Zero ceremony — `import { ship }` and
 *  call it; the client lazily binds same-origin on first property access.
 *
 * ```ts
 * import { ship } from '@ship/sdk-client'
 * const agents = live(() => ship.agents.summaries())
 * ```
 *
 * Safe to import anytime. A host overrides the transport via `setShipClient`
 * before first use; otherwise the same-origin default just works. */
export const ship: ShipSDK = new Proxy({} as ShipSDK, {
  get(_target, prop, _receiver) {
    return Reflect.get(getShipClient() as object, prop)
  },
})

export function createClient(opts?: CreateClientOpts): ShipSDK {
  const baseUrl = (opts?.baseUrl ?? '').replace(/\/$/, '')
  const ESCtor = opts?.EventSource ?? globalThis.EventSource

  // Per-domain Connect clients (#72). Mutations route here; the daemon's
  // auth middleware reads the session cookie (same-origin transport), so no
  // token plumbing. Opaque `JsonResponse` ops decode back to the existing
  // ShipSDK shape via `okJson` — a pure transport swap, callers unchanged.
  const connect = (opts?.connectClient ?? createShipConnectClient({ baseUrl })) as any
  const okJson = async <T>(p: Promise<{ json: string }>): Promise<T> =>
    JSON.parse((await p).json) as T
  // Typed mutations (ScopedMutation / CollectionItem) carry the id directly;
  // adapt to the ShipSDK `{ ok, id }` envelope the REST path returned.
  const okId = async (p: Promise<{ id: string }>): Promise<{ ok: true; id: string }> => ({
    ok: true,
    id: (await p).id,
  })

  // Live reads flow through the multiplex transport (#214 fix). One MuxClient
  // per createClient instance: lazily opens a window + SSE stream on first
  // subscription; each (domain, op, args) tuple becomes a POST /mux/subscribe;
  // full-value frames arrive over the shared EventSource and are routed to the
  // matching LiveSignal. The sharedSource wrapper provides per-key dedup and
  // refcounted teardown (#198) on top of the mux subscription.
  const mux = new MuxClient(baseUrl, ESCtor)

  const sse = <T>(domain: string, op: string, args?: unknown): LiveSource<T> => {
    // Dedup key: identical (domain, op, args) reuse one upstream subscription
    // (#198). Multiple `live(() => ship.<d>.<o>(a))` callers fan out from a
    // single mux subscription with refcounted teardown.
    const key = `${domain}|${op}|${args === undefined ? '' : JSON.stringify(args)}`
    return sharedSource<T>(key, () => ({
      subscribe: (sig) => mux.subscribe<T>(domain, op, args, sig),
    }))
  }

  // Curried HTTP helpers — bind baseUrl once so the per-op factories
  // below stay terse.
  const json = <T>(method: 'POST' | 'PUT' | 'PATCH' | 'DELETE', path: string, body?: unknown) =>
    mutateJson<T>(baseUrl, method, path, body)
  const form = <T>(path: string, body: FormData) => mutateForm<T>(baseUrl, path, body)
  const body = <T>(path: string, as: 'text' | 'blob' | 'json') =>
    fetchBody<T>(baseUrl, path, as)

  return {
    account: {
      state: () => sse('account', 'state'),
    },
    agents: {
      summaries:   () => sse<AgentsSummariesResponse>('agents', 'summaries'),
      create_ctx:  () => sse('agents', 'create_ctx'),
      wire:      (args) => sse('agents', 'wire', args),
      detail:    (args) => sse('agents', 'detail', args),
      settings:  (args) => sse('agents', 'settings', args),
      catalogs:    () => sse('agents', 'catalogs'),
      create:    (args) =>
        okId(connect.agents.create({ id: args.id, docJson: JSON.stringify(args.doc), scope: '' })),
      update:    (args) =>
        okId(connect.agents.put({ id: args.id, docJson: JSON.stringify(args.doc), scope: '' })),
    },
    canvas: {
      doc:  (args) => sse('canvas', 'doc', args),
      list:    () => sse('canvas', 'list'),
    },
    collections: {
      state:  (args) => sse('collections', 'state', args),
      detail: (args) => sse('collections', 'detail', args),
      items:  (args) => sse('collections', 'items', args),
      list:      () => sse('collections', 'list'),
      create: (args) =>
        okJson<{ ok: true; id: string }>(
          connect.collections.createSchema({
            schemaJson: JSON.stringify({
              id: args.id,
              name: args.name ?? args.id,
              fields: args.fields ?? {},
            }),
          }),
        ),
      update_schema: (args) =>
        okJson<{ ok: true; id: string }>(
          connect.collections.updateSchema({
            collection: args.id,
            schemaJson: JSON.stringify({ id: args.id, fields: args.fields }),
          }),
        ),
      get_item: (args) =>
        okJson<{ ok: true; data: { id: string; data: Record<string, unknown> } }>(
          connect.collections.getItem({ collection: args.collection, id: args.item_id }),
        ),
      create_item: (args) =>
        okId(connect.collections.createItem({
          collection: args.collection,
          dataJson: JSON.stringify(args.data),
          id: args.id ?? '',
        })),
      update_item: (args) =>
        okId(connect.collections.updateItem({
          collection: args.collection,
          id: args.item_id,
          dataJson: JSON.stringify(args.data),
        })),
      delete_item: (args) =>
        connect.collections
          .deleteItem({ collection: args.collection, id: args.item_id })
          .then(() => {}),
    },
    events: {
      list: (args) => sse('events', 'list', args),
    },
    // git is parked: read-only. overview/diff/annotations route through the
    // mux resolver (`git.overview` registry op); the write path went out with
    // GitService.
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
      // Typed server-stream: the wire's `SettingsService.Watch` is
      // `server_streaming` and returns the shared `WatchEvent`
      // (patches / err / heartbeat). The settings chrome treats the
      // stream as a render trigger — re-emits on any frame, error
      // surfaces cleanly via the existing `LiveSignal<error>` path.
      // Scope is RUNTIME (shell preferences live in the daemon's
      // RUNTIME settings table — see apps/shell/src/ops.ts).
      state: () =>
        liveStream((signal) =>
          connect.settings.watch({ scope: SettingScope.RUNTIME }, { signal }),
        ),
    },
    sessions: {
      list: () => sse('sessions', 'list'),
      start: (args) =>
        json<{ session_id: string; attached: boolean }>(
          'POST',
          '/api/v1/sessions/spawn',
          args,
        ),
      end: (args) => json<void>('DELETE', `/api/v1/sessions/${encodeURIComponent(args.id)}`),
    },
    artifacts: {
      list: (args) => sse('artifacts', 'list', args),
      get: ((args: { id: string; as?: 'text' | 'blob' | 'json' }) =>
        body(`/api/v1/artifacts/${encodeURIComponent(args.id)}`, args.as ?? 'text')
      ) as ShipSDK['artifacts']['get'],
      upload: (args) => {
        const fd = new FormData()
        const name = args.filename
          ?? (args.file instanceof File ? args.file.name : 'upload.bin')
        fd.append('file', args.file, name)
        if (args.collection_id) fd.append('collection_id', args.collection_id)
        if (args.item_id) fd.append('item_id', args.item_id)
        return form<{ ok: true; data: ArtifactRecord }>('/api/v1/artifacts', fd)
      },
    },
    skills: {
      list:        () => sse('skills', 'list'),
      summaries: (args) => sse('skills', 'summaries', args),
      detail:    (args) => sse('skills', 'detail', args),
    },
    pipelines: {
      summaries: () => sse('pipelines', 'summaries'),
      run:    (args) => sse('pipelines', 'run', args),
      focus:  (args) => sse('pipelines', 'focus', args),
      history: (args) => sse('pipelines', 'history', args ?? {}),
      dispatch: (args) =>
        okJson(connect.pipelines.dispatch({
          pipeline: args.pipeline,
          spec: args.spec ?? '',
          harness: args.harness ?? '',
          auto: args.auto ?? false,
        })),
    },
    spaces: {
      config: (args) => readSpaceConfig(baseUrl, args),
    },
   }
 }
