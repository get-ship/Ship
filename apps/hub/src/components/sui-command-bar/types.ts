// Shared types, constants, and utilities for the sui-command-bar component family.
//
// Hub port of apps/shell/src/components/sui-command-bar/types.ts.
// Seam changes from shell:
//   - `ops` sourced from '../../lib/client' (hub's shared Connect client)
//   - `navigateTo` removed — callers import `navigate` from '@ship/deck/router' directly
//   - `chrome` import removed (no chrome IPC in the hub; navigate() is the single path)

import type { Signal } from '@ship/sui-runtime'
import { EntityKind, type SearchHit as ProtoSearchHit } from '@ship/sdk/connect'
import { ops } from '../../lib/client'

export type BarMode = 'ai' | 'command' | 'find' | 'mention' | 'create'

export type CreateSurface = 'view' | 'skill' | 'agent' | 'pipeline' | 'trigger' | 'theme'

export interface CreateSurfaceDef {
  id: CreateSurface
  label: string
  icon: string
  shapes: string[]
  shapeLabel: string
}

export interface SessionSummary {
  /** PTY session id (the runtime `SessionMeta.id` from `agents.sessions`). */
  id: string
  workspace_id: string
  agent_id?: string
  harness?: string
  status: string
  /** Dispatch handle for `agents.stop` / `agents.steer` (NOT the same as
   *  `id` — slug is `<workspace>-<ulid-suffix>` and only the cmdk that
   *  dispatched has it; sessions listed from `agents.sessions` won't carry
   *  one until the daemon enriches `SessionMeta` with the slug field). */
  slug?: string
}

export interface AgentSummary {
  id: string
  name?: string
  label?: string
}

export interface ThreadLine {
  id: string
  sender: 'user' | 'agent' | 'system'
  text: string
}

export interface NavDest {
  label: string
  path: string
  icon: string
  keywords: string
}

// ── Unified search ─────────────────────────────────────────────────

export type SearchEntityKind =
  | 'view'
  | 'collection'
  | 'collection_item'
  | 'agent'
  | 'skill'
  | 'session'
  | 'artifact'

export interface SearchHit {
  kind: SearchEntityKind
  /** Present only when kind === 'collection_item'. */
  collection_id?: string
  id: string
  label: string
  path: string
  icon?: string
  snippet?: string
  score: number
}

export interface SearchResponse {
  ok: boolean
  data: {
    hits: SearchHit[]
    total: number
    cursor?: string
  }
}

/** Friendly section header for each kind. */
export const SEARCH_KIND_LABELS: Record<SearchEntityKind, string> = {
  view: 'Views',
  collection: 'Collections',
  collection_item: 'Items',
  agent: 'Agents',
  skill: 'Skills',
  session: 'Sessions',
  artifact: 'Artifacts',
}

/** Call the unified search endpoint. Returns `null` on network error.
 *  The hub's ops client fans out across spaces server-side
 *  (run_global_search), so the palette search is multi-space for free. */
export async function searchEntities(
  q: string,
  opts: { limit?: number; types?: SearchEntityKind[]; cursor?: string; signal?: AbortSignal } = {},
): Promise<SearchResponse | null> {
  try {
    const resp = await ops.search.search(
      {
        q,
        limit: opts.limit ?? 0,
        cursor: opts.cursor ?? '',
        kinds: opts.types?.map(toEntityKind) ?? [],
      },
      opts.signal ? { signal: opts.signal } : {},
    )
    return {
      ok: true,
      data: {
        hits: resp.hits.map(toSearchHit),
        total: resp.hits.length,
        ...(resp.nextCursor ? { cursor: resp.nextCursor } : {}),
      },
    }
  } catch {
    return null
  }
}

/** View string → wire enum. */
function toEntityKind(kind: SearchEntityKind): EntityKind {
  switch (kind) {
    case 'view': return EntityKind.VIEW
    case 'collection': return EntityKind.COLLECTION
    case 'collection_item': return EntityKind.COLLECTION_ITEM
    case 'agent': return EntityKind.AGENT
    case 'skill': return EntityKind.SKILL
    case 'session': return EntityKind.SESSION
    case 'artifact': return EntityKind.ARTIFACT
  }
}

/** Wire hit → view hit. */
function toSearchHit(hit: ProtoSearchHit): SearchHit {
  return {
    kind: fromEntityKind(hit.kind),
    id: hit.id,
    label: hit.label,
    path: hit.path,
    score: hit.score,
    ...(hit.icon ? { icon: hit.icon } : {}),
    ...(hit.snippet ? { snippet: hit.snippet } : {}),
    ...(hit.collectionId ? { collection_id: hit.collectionId } : {}),
  }
}

/** Wire enum → view string. UNSPECIFIED defaults to 'view'. */
function fromEntityKind(kind: EntityKind): SearchEntityKind {
  switch (kind) {
    case EntityKind.VIEW: return 'view'
    case EntityKind.COLLECTION: return 'collection'
    case EntityKind.COLLECTION_ITEM: return 'collection_item'
    case EntityKind.AGENT: return 'agent'
    case EntityKind.SKILL: return 'skill'
    case EntityKind.SESSION: return 'session'
    case EntityKind.ARTIFACT: return 'artifact'
    default: return 'view'
  }
}

export interface SlashCommand {
  key: string
  label: string
  hint: string
  handler_kind?: 'mode' | 'navigate' | 'endpoint' | 'local'
}

// Session-level cache for the slash registry. Static client-side list.
const SLASH_CACHE_MAX_AGE_MS = 60_000
let slashCache: { at: number; commands: SlashCommand[] } | null = null

export async function loadSlashCommands(_force = false): Promise<SlashCommand[]> {
  slashCache = { at: performance.now(), commands: SLASH_COMMANDS }
  return slashCache.commands
}

export function cachedSlashCommands(): SlashCommand[] {
  return slashCache?.commands ?? SLASH_COMMANDS
}

export function slashCacheAge(): number {
  return slashCache ? performance.now() - slashCache.at : Number.POSITIVE_INFINITY
}

export const SLASH_CACHE_TTL_MS = SLASH_CACHE_MAX_AGE_MS

export const CREATE_SURFACES: CreateSurfaceDef[] = [
  { id: 'view',     label: 'View',     icon: 'file',     shapes: ['list', 'form', 'detail', 'dashboard', 'tabs', 'blank'], shapeLabel: 'Starting shape' },
  { id: 'skill',    label: 'Skill',    icon: 'sparkles', shapes: ['task', 'reference', 'workflow'],                        shapeLabel: 'Skill kind'     },
  { id: 'agent',    label: 'Agent',    icon: 'users',    shapes: ['specialist', 'generalist', 'reviewer'],                 shapeLabel: 'Agent kind'     },
  { id: 'pipeline', label: 'Pipeline', icon: 'play',     shapes: ['sequential', 'fanout', 'review-gated'],                 shapeLabel: 'Topology'       },
  { id: 'trigger',  label: 'Trigger',  icon: 'zap',      shapes: ['webhook', 'schedule', 'file-change'],                   shapeLabel: 'Kind'           },
  { id: 'theme',    label: 'Theme',    icon: 'palette',  shapes: ['light', 'dark', 'brand-accent', 'high-contrast'],        shapeLabel: 'Starting shape' },
]

export const SLASH_COMMANDS: SlashCommand[] = [
  { key: 'new',      label: '/new',      hint: 'Scaffold a view, skill, agent, …' },
  { key: 'chats',    label: '/chats',    hint: 'Open Mission Control'              },
  { key: 'settings', label: '/settings', hint: 'Jump to settings'                 },
  { key: 'theme',    label: '/theme',    hint: 'Open Theme Studio',          handler_kind: 'navigate' },
  { key: 'lint',     label: '/lint',     hint: 'Run lint on the current view'      },
  { key: 'end',      label: '/end',      hint: 'End the current session'           },
  { key: 'close',    label: '/close',    hint: 'Close the command bar'             },
]

const KEBAB_RE = /^[a-z][a-z0-9-]{0,48}$/
export function isKebab(s: string): boolean { return KEBAB_RE.test(s) }

export function detectMode(value: string, fallback: BarMode): BarMode {
  if (value.startsWith('/')) return 'command'
  if (value.startsWith('>')) return 'find'
  if (value.startsWith('+')) return 'create'
  if (value.trim().length === 0) return fallback
  if (/(^|\s)@[^\s@]*$/.test(value)) return 'mention'
  return 'ai'
}

// The shared reactive scope sui-command-bar provisions (via `shared(...)`) for
// its sibling panels (palette, chat) to consume through
// `useShared<CommandBarScope>(host, 'sui-command-bar')`.
export interface CommandBarScope {
  open: Signal<boolean>
  mode: Signal<BarMode>
  value: Signal<string>
  session: Signal<SessionSummary | null>
  agents: Signal<AgentSummary[]>
  harnesses: Signal<string[]>
  allSessions: Signal<SessionSummary[]>
  views: Signal<string[]>
  selectedIdx: Signal<number>
  sessionPickerOpen: Signal<boolean>
  spawnAgent: Signal<string>
  spawnHarness: Signal<string>
  spawning: Signal<boolean>
  spawnError: Signal<string | null>
  thread: Signal<ThreadLine[]>
  workspaceId: Signal<string>
  createSurface: Signal<CreateSurface>
  createName: Signal<string>
  createPurpose: Signal<string>
  createShape: Signal<string>
  createSubmitting: Signal<boolean>
  createError: Signal<string | null>
  isChat: Signal<boolean>
  dialogRef: Signal<HTMLElement | null>
  slashCommands: Signal<SlashCommand[]>
  sendKey: Signal<'enter' | 'cmd+enter'>
  searchHits: Signal<SearchHit[]>
  searching: Signal<boolean>
  daemonOnline: Signal<boolean>
  recentSends: Signal<string[]>
  historyIdx: Signal<number>
  sendError: Signal<string | null>
}
