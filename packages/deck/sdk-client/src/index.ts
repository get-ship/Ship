// @ship/sdk-client — browser SDK shim for ship-set views.
//
// Each domain method returns either a `LiveSource` (read — opens an SSE
// channel to `/_sui/data?expr=ship.<domain>.<op>(<args>)`) or a Promise
// (write — POST/PUT/PATCH/DELETE against the daemon's REST surface).
// The runtime's `live()` detects the source shape, calls `subscribe(sig)`
// to open the patch stream, and the first SSE event flips the signal
// from `loading` to `ok`. The disposer returned by `subscribe` closes
// the EventSource on tear-down.
//
// HTTP plumbing lives in `./http.ts`; the SSE bridge in `./sse.ts`.
// Keeping `index.ts` to the surface declaration only respects the 300-
// line file cap and makes the SDK surface readable at a glance.

import {
  type AgentsSummariesResponse,
  type GitDiff,
  type GitDiffArgs,
  type GitOverview,
  type LiveSource,
} from '@ship/sui-runtime'
import type { ArtifactRecord, ShipSDK } from './ship-sdk'
import { fetchBody, mutateForm, mutateJson } from './http'
import { makeSseSource } from './sse'
import { liveStream } from './live-stream'
import { createShipConnectClient } from '@ship/sdk/connect'

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

export { makeSseSource } from './sse'
export { liveStream } from './live-stream'
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

// Generated daemon data types (still sourced from the framework's generated.ts
// for now — relocating the gen output out of the framework is the next step).
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
// The Ship SDK surface + its hand-authored data types — owned here, not the framework.
export type {
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
export function createClient(opts?: CreateClientOpts): ShipSDK {
  const baseUrl = (opts?.baseUrl ?? '').replace(/\/$/, '')
  const ESCtor = opts?.EventSource ?? globalThis.EventSource

  // Per-domain Connect clients (#72). Mutations route here; the daemon's
  // auth middleware reads the session cookie (same-origin transport), so no
  // token plumbing. Opaque `JsonResponse` ops decode back to the existing
  // ShipSDK shape via `okJson` — a pure transport swap, callers unchanged.
  const connect = createShipConnectClient({ baseUrl })
  const okJson = async <T>(p: Promise<{ json: string }>): Promise<T> =>
    JSON.parse((await p).json) as T
  // Typed mutations (ScopedMutation / CollectionItem) carry the id directly;
  // adapt to the ShipSDK `{ ok, id }` envelope the REST path returned.
  const okId = async (p: Promise<{ id: string }>): Promise<{ ok: true; id: string }> => ({
    ok: true,
    id: (await p).id,
  })

  // Live reads stream over the per-domain Connect `<Op>Live` rpcs (#138):
  // `ship.<domain>.<op>()` opens `ops.<domain>.<op>Live` and flips the signal on
  // each full-value frame — real typed RPC on the wire, no expr string, no SSE
  // eval. The (domain, op) → client-method lookup is the one internal bridge;
  // the `ShipSDK` interface keeps every caller fully typed. Snake_case args are
  // camelCased to match the generated message fields.
  type LiveMethod = (req: unknown, opts: { signal: AbortSignal }) => AsyncIterable<{ json: string }>
  const camelKeys = (v: unknown): unknown => {
    if (Array.isArray(v)) return v.map(camelKeys)
    if (v && typeof v === 'object') {
      return Object.fromEntries(
        Object.entries(v as Record<string, unknown>).map(([k, val]) => [
          k.replace(/_([a-z])/g, (_m, c: string) => c.toUpperCase()),
          camelKeys(val),
        ]),
      )
    }
    return v
  }
  const sse = <T>(domain: string, op: string, args?: unknown): LiveSource<T> => {
    const method = op.replace(/_([a-z])/g, (_m, c: string) => c.toUpperCase()) + 'Live'
    const svc = (connect as unknown as Record<string, Record<string, LiveMethod>>)[domain]
    const fn = svc?.[method]
    // mesh + canvas have no per-domain Connect service yet (operational / canvas
    // #99 in-flux) — those alone fall back to the legacy expr-SSE channel.
    if (typeof fn !== 'function') {
      const argsLit = args === undefined ? '' : JSON.stringify(args)
      const expr = `ship.${domain}.${op}(${argsLit})`
      return makeSseSource<T>(`${baseUrl}/_sui/data?expr=${encodeURIComponent(expr)}`, ESCtor)
    }
    return liveStream<T>((signal) => fn(camelKeys(args ?? {}), { signal }))
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
    chat: {
      state: (args) => sse('chat', 'state', args),
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
      stage:       (args) =>
        okJson<{ ok: true; staged: number }>(
          connect.git.stage({ workspaceId: args.workspace_id ?? '', paths: args.paths }),
        ),
      unstage:     (args) =>
        okJson<{ ok: true; unstaged: number }>(
          connect.git.unstage({ workspaceId: args.workspace_id ?? '', paths: args.paths }),
        ),
      commit:      (args) =>
        okJson<{ ok: true; sha: string; branch: string | null }>(
          connect.git.commit({
            workspaceId: args.workspace_id ?? '',
            message: args.message,
            author: args.author ?? '',
          }),
        ),
      push:        (args) =>
        okJson<{ ok: true; remote: string; force: boolean }>(
          connect.git.push({
            workspaceId: args?.workspace_id ?? '',
            remote: args?.remote ?? '',
            force: args?.force ?? false,
          }),
        ),
      fetch:       (args) =>
        okJson<{ ok: true; remote: string }>(
          connect.git.fetch({ workspaceId: args?.workspace_id ?? '', remote: args?.remote ?? '' }),
        ),
      pull:        (args) =>
        okJson<{ ok: true; head: string }>(
          connect.git.pull({ workspaceId: args?.workspace_id ?? '', remote: args?.remote ?? '' }),
        ),
      checkout:    (args) =>
        okJson<{ ok: true; ref: string; resolved: string }>(
          connect.git.checkout({ workspaceId: args.workspace_id ?? '', refname: args.ref }),
        ),
      branch_create: (args) =>
        okJson<{ ok: true; name: string }>(
          connect.git.branchCreate({
            workspaceId: args.workspace_id ?? '',
            name: args.name,
            checkout: args.checkout ?? false,
          }),
        ),
      branch_delete: (args) =>
        okJson<{ ok: true; name: string }>(
          connect.git.branchDelete({
            workspaceId: args.workspace_id ?? '',
            name: args.name,
            force: args.force ?? false,
          }),
        ),
      init:        (args) =>
        okJson<{ ok: true }>(connect.git.init({ workspaceId: args?.workspace_id ?? '' })),
    },
    mesh: {
      snapshot: () => sse('mesh', 'snapshot'),
    },
    settings: {
      state: () => sse('settings', 'state'),
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
      invoke:    (args) =>
        json<{
          ok: true
          run_id: string
          skill_id: string
          session_id: string
          workspace_id: string | null
          mode: 'library'
          reused_session: boolean
        }>(
          'POST',
          `/api/v1/skills/${encodeURIComponent(args.id)}/invoke`,
          {
            vars: args.vars,
            workspace_id: args.workspace_id,
            session_id: args.session_id,
          },
        ),
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
  }
}
