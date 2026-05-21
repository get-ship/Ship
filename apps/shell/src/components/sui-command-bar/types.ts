// Shared types, constants, and utilities for the sui-command-bar component family.

import { chrome } from '@ship/sui-runtime'

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
  id: string
  workspace_id: string
  agent_id?: string
  harness?: string
  status: string
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

/** Call the unified search endpoint. Returns `null` on network error. */
export async function searchEntities(
  q: string,
  opts: { limit?: number; types?: SearchEntityKind[]; cursor?: string; signal?: AbortSignal } = {},
): Promise<SearchResponse | null> {
  const params = new URLSearchParams()
  if (q) params.set('q', q)
  if (opts.limit != null) params.set('limit', String(opts.limit))
  if (opts.types?.length) params.set('types', opts.types.join(','))
  if (opts.cursor) params.set('cursor', opts.cursor)
  try {
    const resp = await fetch(`/api/v1/search?${params.toString()}`, {
      credentials: 'same-origin',
      signal: opts.signal,
    })
    if (!resp.ok) return null
    return await resp.json() as SearchResponse
  } catch {
    return null
  }
}

export interface SlashCommand {
  key: string
  label: string
  hint: string
  handler_kind?: 'mode' | 'navigate' | 'endpoint' | 'local'
}

// Session-level cache for the slash registry. The bar refreshes when the
// cached snapshot crosses MAX_AGE_MS at the moment the user opens cmdk —
// no background polling. Falls back to the seeded constant on fetch error
// so the palette is never empty.
const SLASH_CACHE_MAX_AGE_MS = 60_000
let slashCache: { at: number; commands: SlashCommand[] } | null = null
let slashInFlight: Promise<SlashCommand[]> | null = null

export async function loadSlashCommands(force = false): Promise<SlashCommand[]> {
  const now = performance.now()
  if (!force && slashCache && now - slashCache.at < SLASH_CACHE_MAX_AGE_MS) {
    return slashCache.commands
  }
  if (slashInFlight) return slashInFlight
  slashInFlight = (async () => {
    try {
      const resp = await fetch('/api/v1/commands', { credentials: 'same-origin' })
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
      const body = await resp.json() as { commands?: SlashCommand[] }
      const commands = body.commands ?? []
      slashCache = { at: performance.now(), commands }
      return commands
    } catch {
      // Hold previous cache if any, else fall back to the seeded constant
      // so the bar is usable when the daemon is unreachable.
      if (slashCache) return slashCache.commands
      slashCache = { at: performance.now(), commands: SLASH_COMMANDS }
      return SLASH_COMMANDS
    } finally {
      slashInFlight = null
    }
  })()
  return slashInFlight
}

export function cachedSlashCommands(): SlashCommand[] {
  return slashCache?.commands ?? SLASH_COMMANDS
}

export function slashCacheAge(): number {
  return slashCache ? performance.now() - slashCache.at : Number.POSITIVE_INFINITY
}

export const SLASH_CACHE_TTL_MS = SLASH_CACHE_MAX_AGE_MS

export const CREATE_SURFACES: CreateSurfaceDef[] = [
  { id: 'view',     label: 'View',     icon: 'file',           shapes: ['list', 'form', 'detail', 'dashboard', 'tabs', 'blank'],    shapeLabel: 'Starting shape' },
  { id: 'skill',    label: 'Skill',    icon: 'sparkles',       shapes: ['task', 'reference', 'workflow'],                           shapeLabel: 'Skill kind'     },
  { id: 'agent',    label: 'Agent',    icon: 'users',          shapes: ['specialist', 'generalist', 'reviewer'],                    shapeLabel: 'Agent kind'     },
  { id: 'pipeline', label: 'Pipeline', icon: 'play',           shapes: ['sequential', 'fanout', 'review-gated'],                    shapeLabel: 'Topology'       },
  { id: 'trigger',  label: 'Trigger',  icon: 'zap',            shapes: ['webhook', 'schedule', 'file-change'],                      shapeLabel: 'Kind'           },
  { id: 'theme',    label: 'Theme',    icon: 'palette',        shapes: ['light', 'dark', 'brand-accent', 'high-contrast'],          shapeLabel: 'Starting shape' },
]

export const NAV_DESTINATIONS: NavDest[] = [
  { label: 'Home',                  path: '/home',         icon: 'home',     keywords: 'overview dashboard start'  },
  { label: 'Canvas',                path: '/canvas',       icon: 'image',    keywords: 'docs visual feedback'      },
  { label: 'Collections',           path: '/collections',  icon: 'folder',   keywords: 'data items query'          },
  { label: 'Artifacts',             path: '/artifacts',    icon: 'file',     keywords: 'outputs files exports'     },
  { label: 'Agents',                path: '/studio/agents', icon: 'users',   keywords: 'configure profiles studio' },
  { label: 'Skills',                path: '/studio/skills', icon: 'zap',     keywords: 'browse edit files'         },
  { label: 'Git',                   path: '/git',          icon: 'git',      keywords: 'status diff commit branch' },
  { label: 'Components',            path: '/studio/components', icon: 'sparkles', keywords: 'components design tokens ui studio catalog' },
  { label: 'Settings',              path: '/settings',     icon: 'settings', keywords: 'config theme harness'     },
  { label: 'Theme Studio',          path: '/theme-studio', icon: 'palette',  keywords: 'design tokens'             },
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

export async function navigateTo(path: string): Promise<void> {
  const w = window as Record<string, unknown> & { ship?: { navigate?: (p: string) => void } }
  if (w.ship?.navigate) { w.ship.navigate(path); return }
  chrome.navigate(path)
}
