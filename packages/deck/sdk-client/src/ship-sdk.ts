// ship.ts — typed handle for the Ship SDK.
//
// The SDK is injected by the runtime (and the daemon's hydrate flow)
// into:
//   - the `source=` attribute of `<data>` declarations
//   - the function form of a controller (`({ ship }) => ...`)
//   - each controller section's destructure
//   - controller hook factories that accept scope
//
// Authors do not import the runtime value of `ship`; only this type.
//
// Return shapes use `@ship/sui-runtime`'s generated.ts when the daemon
// struct carries `#[derive(Type)]`; otherwise `unknown` until a real
// shape lands. Op names mirror the daemon's `schema.rs::OPS` catalog.
//
// Domain knowledge here mirrors what generated.ts already carries
// (AgentsSummariesResponse, GitOverview, etc.) — keeping ops + shapes
// co-located so controller code can read `ship.foo.bar()` without the
// indirection of a per-project SDK manifest.

import type {
  AccountState,
  AgentsSummariesResponse,
  ArtifactRecord,
  GitDiff,
  GitDiffArgs,
  GitOverview,
  SkillIndexEntry,
} from './generated'
import type { LiveSource } from '@ship/sui-runtime'

export type { ArtifactRecord, SkillIndexEntry }

/** Collection field record for view controllers (id+attrs). */
export interface CollectionFieldRecord {
  name: string
  type: string
  field_type: string
  required: boolean
  values: string[]
  default: unknown
}

/** Collection summary row exposed by `ship.collections.list()`. */
export interface CollectionSummary {
  id: string
  name: string
  field_count: number
  fields: CollectionFieldRecord[]
  builtin: boolean
}

/** Live SDK handle exposed to controllers, hooks, and `<data>` sources. */
export interface ShipSDK {
  account: {
    /** Ship Cloud account snapshot — logged_out / pending / logged_in.
     *  Re-emits on the next state-changing event; today the daemon only
     *  flips the value on explicit login/logout so the chip refreshes
     *  whenever those routes run. */
    state(): LiveSource<AccountState>
  }
  agents: {
    /** Agent list summaries — full payload as first patch, deltas after. */
    summaries(): LiveSource<AgentsSummariesResponse>
    /** Agent-create wizard ctx (templates + presets + harnesses). */
    create_ctx(): LiveSource<unknown>
    /** Graph payload (nodes + edges) for the agent-wire canvas. */
    wire(args: { id: string }): LiveSource<unknown>
    /** Full view-shaped agent detail. */
    detail(args: { id: string }): LiveSource<unknown>
    /** Tier list + catalog rows for the per-agent settings pane. */
    settings(args: { id: string }): LiveSource<unknown>
    /** Catalog rows for the agents list (harnesses, skills, etc.). */
    catalogs(): LiveSource<unknown>
    /** Create an agent. Body is the agent doc as written to disk; `id` is
     *  the agent slug. */
    create(args: { id: string; doc: Record<string, unknown> }): Promise<{
      ok: true
      id: string
    }>
    /** Replace an agent doc (PUT). Used by agent settings + wire panes. */
    update(args: { id: string; doc: Record<string, unknown> }): Promise<{
      ok: true
      id: string
    }>
  }
  canvas: {
    /** Canvas document body. */
    doc(args: { slug: string }): LiveSource<unknown>
    /** Available canvas documents. */
    list(): LiveSource<unknown>
  }
  chat: {
    /** Recent chat messages, optionally scoped to a workspace. */
    state(args?: { workspace_id?: string }): LiveSource<unknown>
  }
  collections: {
    /** Collections list + active schema + items. */
    state(args?: { active?: string }): LiveSource<unknown>
    /** Single collection's schema + items. */
    detail(args: { id: string }): LiveSource<{
      schema: {
        id: string
        name: string
        fields: CollectionFieldRecord[]
        builtin: boolean
      }
      items: unknown[]
      total_items: number
    }>
    /** Rows within a collection. */
    items(args: { collection: string }): LiveSource<{
      items: unknown[]
      total_items: number
    }>
    /** All collection summaries. */
    list(): LiveSource<{ items: CollectionSummary[] }>
    /** Create a new collection schema. */
    create(args: {
      id: string
      name?: string
      fields?: Record<string, unknown>
    }): Promise<{ ok: true; id: string }>
    /** Replace a collection's schema (field set). */
    update_schema(args: {
      id: string
      fields: Array<{
        name: string
        type: string
        required?: boolean
        values?: string[]
      }>
    }): Promise<{ ok: true; id: string }>
    /** Read a single item by id. */
    get_item(args: { collection: string; item_id: string }): Promise<{
      ok: true
      data: { id: string; data: Record<string, unknown> }
    }>
    /** Insert an item into a collection. */
    create_item(args: {
      collection: string
      data: Record<string, unknown>
      id?: string
    }): Promise<{ ok: true; id: string }>
    /** Partial-merge an item. */
    update_item(args: {
      collection: string
      item_id: string
      data: Record<string, unknown>
    }): Promise<{ ok: true; id: string }>
    /** Delete an item. */
    delete_item(args: { collection: string; item_id: string }): Promise<void>
  }
  dashboard: {
    /** Aggregate stats + recent runs + events + agents + collections. */
    state(): LiveSource<unknown>
  }
  events: {
    /** Kernel events with filters, grouped by entity. */
    list(args?: {
      limit?: number
      event_type?: string
      actor?: string
      entity?: string
      q?: string
    }): LiveSource<unknown>
  }
  git: {
    /** Repo overview — branch + head + status buckets + commits + branches. */
    overview(args?: { workspace_id?: string; limit?: number }): LiveSource<GitOverview>
    /** Unified diff for a commit (or `'WORKING'` for the working tree). */
    diff(args: GitDiffArgs): LiveSource<GitDiff>
    /** Per-file annotation overlays for a diff. */
    annotations(args: { file: string; workspace_id?: string }): LiveSource<unknown>
    /** Stage one or more paths (file-level). */
    stage(args: { paths: string[]; workspace_id?: string }): Promise<{
      ok: true
      staged: number
    }>
    /** Unstage one or more paths. */
    unstage(args: { paths: string[]; workspace_id?: string }): Promise<{
      ok: true
      unstaged: number
    }>
    /** Commit the current index. `author` is `"Name <email>"` or omitted to
     *  inherit from git config. */
    commit(args: {
      message: string
      author?: string
      workspace_id?: string
    }): Promise<{ ok: true; sha: string; branch: string | null }>
    /** Push current branch to `remote` (default `origin`). */
    push(args?: {
      remote?: string
      force?: boolean
      workspace_id?: string
    }): Promise<{ ok: true; remote: string; force: boolean }>
    /** Fetch refs from `remote` (default `origin`). */
    fetch(args?: { remote?: string; workspace_id?: string }): Promise<{
      ok: true
      remote: string
    }>
    /** Fast-forward-only pull. Surfaces `non_ff` 409 on divergence. */
    pull(args?: { remote?: string; workspace_id?: string }): Promise<{
      ok: true
      head: string
    }>
    /** Checkout a branch / tag / sha. */
    checkout(args: { ref: string; workspace_id?: string }): Promise<{
      ok: true
      ref: string
      resolved: string
    }>
    /** Create a local branch from HEAD. */
    branch_create(args: {
      name: string
      checkout?: boolean
      workspace_id?: string
    }): Promise<{ ok: true; name: string }>
    /** Delete a local branch (refuses current branch + unmerged unless `force`). */
    branch_delete(args: {
      name: string
      force?: boolean
      workspace_id?: string
    }): Promise<{ ok: true; name: string }>
    /** Initialize a git repository at the active workspace root. */
    init(args?: { workspace_id?: string }): Promise<{ ok: true }>
  }
  mesh: {
    /** Live registry of mesh-connected agents. */
    snapshot(): LiveSource<unknown>
  }
  settings: {
    /** Settings view ctx — harnesses, defaults, terminal, about. */
    state(): LiveSource<unknown>
  }
  sessions: {
    /** Live PTY-session registry (Mission Control feed). */
    list(): LiveSource<unknown>
    /** Spawn a new PTY session and attach the actor.
     *  `kind` is `'agent' | 'shell'`; agents require `agent_id` + `harness`. */
    start(args: {
      workspace_id: string
      kind: 'agent' | 'shell'
      agent_id?: string
      harness?: string
    }): Promise<{ session_id: string; attached: boolean }>
    /** Gracefully stop a live session. Resolves once the daemon acks. */
    end(args: { id: string }): Promise<void>
  }
  artifacts: {
    /** Artifact metadata records, filterable by scope/mime/collection/etc. */
    list(args?: {
      scope?: string
      mime?: string
      collection_id?: string
      item_id?: string
      owner_actor?: string
      content_hash?: string
      sort?: string
      limit?: number
      offset?: number
    }): LiveSource<{
      items: ArtifactRecord[]
      count: number
      scope: string
    }>
    /** Fetch raw artifact body. `as` picks the response shape — `text`
     *  (default) for preview panes, `blob` for binary downloads, `json`
     *  for structured payloads. */
    get(args: { id: string; as?: 'text' }): Promise<string>
    get(args: { id: string; as: 'blob' }): Promise<Blob>
    get(args: { id: string; as: 'json' }): Promise<unknown>
    /** Upload an artifact via multipart form. */
    upload(args: {
      file: File | Blob
      filename?: string
      collection_id?: string
      item_id?: string
    }): Promise<{ ok: true; data: ArtifactRecord }>
  }
  skills: {
    /** All skill IndexEntry rows across project + user scopes. */
    list(): LiveSource<{ items: SkillIndexEntry[] }>
    /** Skills tree (sources + groups) for the explorer. */
    summaries(args?: {
      q?: string
      kind?: string
      tag?: string
    }): LiveSource<unknown>
    /** Single skill detail (frontmatter + body + vars). */
    detail(args: { id: string }): LiveSource<unknown>
    /** Invoke a library skill — attach its reference content to a
     *  session and emit lifecycle events on the kernel bus.
     *
     *  Resolves the skill, mints a `run_id`, picks a `session_id`
     *  (explicit > active-agent-for-workspace > synthetic = run_id),
     *  and emits `skill.started` → `skill.attached_to_session` →
     *  `skill.completed`. Operator + mode skill types are not yet
     *  wired through this surface — see
     *  `.artifacts/sui-arc/skills-invoke-real.md`. */
    invoke(args: {
      id: string
      vars?: Record<string, unknown>
      workspace_id?: string
      session_id?: string
    }): Promise<{
      ok: true
      run_id: string
      skill_id: string
      session_id: string
      workspace_id: string | null
      mode: 'library'
      reused_session: boolean
    }>
  }
  pipelines: {
    /** Pipeline summaries for the studio list. Live SSE. */
    summaries(): LiveSource<unknown>
    /** Single run detail by slug. Live SSE. */
    run(args: { slug: string }): LiveSource<unknown>
    /** Focused pipeline (definition + runs) by name. Live SSE. */
    focus(args: { name: string }): LiveSource<unknown>
    /** Run history, filterable. Live SSE. */
    history(args?: {
      q?: string
      pipeline?: string
      offset?: number
      limit?: number
    }): LiveSource<unknown>
    /** Kick off a pipeline run (spawns jobs). */
    dispatch(args: {
      pipeline: string
      spec?: string
      harness?: string
      auto?: boolean
    }): Promise<unknown>
  }
}
