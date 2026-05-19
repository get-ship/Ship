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
  type ArtifactRecord,
  type GitDiff,
  type GitDiffArgs,
  type GitOverview,
  type LiveSource,
  type ShipSDK,
} from '@ship/sui-runtime'
import { fetchBody, mutateForm, mutateJson } from './http'
import { makeSseSource } from './sse'

export { makeSseSource, liveExpr } from './sse'
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

export type {
  AgentSummary,
  AgentsSummariesResponse,
  ArtifactRecord,
  BranchSummary as GitBranch,
  CollectionFieldRecord,
  CollectionSummary,
  CommitSummary as GitCommit,
  GitDiff,
  GitDiffArgs,
  GitOverview,
  ShipSDK,
  SkillIndexEntry,
  StatusFile as GitFileEntry,
  WorkspaceSummary,
} from '@ship/sui-runtime'

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

  // Build the `/_sui/data?expr=...` SSE URL for a single op call. The
  // daemon parses the expr as `ship.<domain>.<op>(<json>?)`. Args are
  // emitted as a JSON object literal when present.
  const sse = <T>(domain: string, op: string, args?: unknown): LiveSource<T> => {
    const argsLit = args === undefined ? '' : JSON.stringify(args)
    const expr = `ship.${domain}.${op}(${argsLit})`
    return makeSseSource<T>(
      `${baseUrl}/_sui/data?expr=${encodeURIComponent(expr)}`,
      ESCtor,
    )
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
        json<{ ok: true; id: string }>('POST', '/api/v1/agents', {
          id: args.id,
          doc: args.doc,
        }),
      update:    (args) =>
        json<{ ok: true; id: string }>(
          'PUT',
          `/api/v1/agents/${encodeURIComponent(args.id)}`,
          args.doc,
        ),
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
        json<{ ok: true; id: string }>('POST', '/api/v1/collections', {
          id: args.id,
          name: args.name ?? args.id,
          fields: args.fields ?? {},
        }),
      update_schema: (args) =>
        json<{ ok: true; id: string }>(
          'PUT',
          `/api/v1/collections/${encodeURIComponent(args.id)}/schema`,
          { id: args.id, fields: args.fields },
        ),
      get_item: (args) =>
        body<{ ok: true; data: { id: string; data: Record<string, unknown> } }>(
          `/api/v1/collections/${encodeURIComponent(args.collection)}/items/${encodeURIComponent(args.item_id)}`,
          'json',
        ),
      create_item: (args) => {
        const b: Record<string, unknown> = { data: args.data }
        if (args.id !== undefined) b.id = args.id
        return json<{ ok: true; id: string }>(
          'POST',
          `/api/v1/collections/${encodeURIComponent(args.collection)}/items`,
          b,
        )
      },
      update_item: (args) =>
        json<{ ok: true; id: string }>(
          'PUT',
          `/api/v1/collections/${encodeURIComponent(args.collection)}/items/${encodeURIComponent(args.item_id)}`,
          { data: args.data },
        ),
      delete_item: (args) =>
        json<void>(
          'DELETE',
          `/api/v1/collections/${encodeURIComponent(args.collection)}/items/${encodeURIComponent(args.item_id)}`,
        ),
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
        json<{ ok: true; staged: number }>('POST', '/api/v1/git/stage', args),
      unstage:     (args) =>
        json<{ ok: true; unstaged: number }>('POST', '/api/v1/git/unstage', args),
      commit:      (args) =>
        json<{ ok: true; sha: string; branch: string | null }>(
          'POST', '/api/v1/git/commit', args,
        ),
      push:        (args) =>
        json<{ ok: true; remote: string; force: boolean }>(
          'POST', '/api/v1/git/push', args ?? {},
        ),
      fetch:       (args) =>
        json<{ ok: true; remote: string }>('POST', '/api/v1/git/fetch', args ?? {}),
      pull:        (args) =>
        json<{ ok: true; head: string }>('POST', '/api/v1/git/pull', args ?? {}),
      checkout:    (args) =>
        json<{ ok: true; ref: string; resolved: string }>(
          'POST', '/api/v1/git/checkout', { ref: args.ref, workspace_id: args.workspace_id },
        ),
      branch_create: (args) =>
        json<{ ok: true; name: string }>('POST', '/api/v1/git/branch', args),
      branch_delete: (args) =>
        json<{ ok: true; name: string }>('DELETE', '/api/v1/git/branch', args),
      init:        (args) =>
        json<{ ok: true }>('POST', '/api/v1/git/init', args ?? {}),
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
    workspaces: {
      list:    () => sse('workspaces', 'list'),
      active:  () => sse('workspaces', 'active'),
      activate: (args) =>
        json<{ ok: true; active: string }>(
          'POST',
          '/api/v1/projects/switch',
          { id: args.id },
        ),
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
  }
}
