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

import type { AgentsSummariesResponse, GitDiff, GitDiffArgs, GitOverview } from './generated'
import type { LiveSource } from './live'

/** Live SDK handle exposed to controllers, hooks, and `<data>` sources. */
export interface ShipSDK {
  agents: {
    /** Agent list summaries — full payload as first patch, deltas after. */
    summaries(): LiveSource<AgentsSummariesResponse>
    /** Agent-create wizard ctx (templates + presets + providers). */
    create_ctx(): LiveSource<unknown>
    /** Graph payload (nodes + edges) for the agent-wire canvas. */
    wire(args: { id: string }): LiveSource<unknown>
    /** Full view-shaped agent detail. */
    detail(args: { id: string }): LiveSource<unknown>
    /** Tier list + catalog rows for the per-agent settings pane. */
    settings(args: { id: string }): LiveSource<unknown>
    /** Catalog rows for the agents list (providers, skills, etc.). */
    catalogs(): LiveSource<unknown>
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
    detail(args: { id: string }): LiveSource<unknown>
    /** Rows within a collection. */
    items(args: { collection: string }): LiveSource<unknown>
    /** All collection summaries. */
    list(): LiveSource<unknown>
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
  }
  mesh: {
    /** Live registry of mesh-connected agents. */
    snapshot(): LiveSource<unknown>
  }
  settings: {
    /** Settings view ctx — providers, defaults, terminal, about. */
    state(): LiveSource<unknown>
  }
}
