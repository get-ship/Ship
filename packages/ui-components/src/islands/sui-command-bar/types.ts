// Shared types, constants, and utilities for the sui-command-bar component family.

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
  provider?: string
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

export interface SlashCommand {
  key: string
  label: string
  hint: string
}

export const CREATE_SURFACES: CreateSurfaceDef[] = [
  { id: 'view',     label: 'View',     icon: 'file',           shapes: ['list', 'form', 'detail', 'dashboard', 'tabs', 'blank'],    shapeLabel: 'Starting shape' },
  { id: 'skill',    label: 'Skill',    icon: 'sparkles',       shapes: ['task', 'reference', 'workflow'],                           shapeLabel: 'Skill kind'     },
  { id: 'agent',    label: 'Agent',    icon: 'users',          shapes: ['specialist', 'generalist', 'reviewer'],                    shapeLabel: 'Agent kind'     },
  { id: 'pipeline', label: 'Pipeline', icon: 'play',           shapes: ['sequential', 'fanout', 'review-gated'],                    shapeLabel: 'Topology'       },
  { id: 'trigger',  label: 'Trigger',  icon: 'zap',            shapes: ['webhook', 'schedule', 'file-change'],                      shapeLabel: 'Kind'           },
  { id: 'theme',    label: 'Theme',    icon: 'palette',        shapes: ['light', 'dark', 'brand-accent', 'high-contrast'],          shapeLabel: 'Starting shape' },
]

export const NAV_DESTINATIONS: NavDest[] = [
  { label: 'Dashboard',             path: '/dashboard',  icon: 'sparkles',          keywords: 'home overview'           },
  { label: 'Pipelines',             path: '/pipelines',  icon: 'play',              keywords: 'workflows runs jobs'     },
  { label: 'Agents',                path: '/agents',     icon: 'users',             keywords: 'configure profiles'      },
  { label: 'Skills',                path: '/skills',     icon: 'zap',               keywords: 'browse edit files'       },
  { label: 'Git',                   path: '/git',        icon: 'git',               keywords: 'status diff commit branch' },
  { label: 'Collections',           path: '/collections',icon: 'folder',            keywords: 'data items query'        },
  { label: 'Canvas',                path: '/canvas',     icon: 'image',             keywords: 'docs visual feedback'    },
  { label: 'Events',                path: '/events',     icon: 'sparkles',          keywords: 'stream log kernel'       },
  { label: 'Settings',              path: '/settings',   icon: 'settings',          keywords: 'config theme provider'   },
  { label: 'Chat',                  path: '/chat',       icon: 'message-square-plus', keywords: 'agent conversation'  },
  { label: 'Specs',                 path: '/specs',      icon: 'file',              keywords: 'features design'         },
  { label: 'Theme Studio',          path: '/theme-studio', icon: 'palette',         keywords: 'design tokens'           },
  { label: 'Chats (Mission Control)', path: '/chats',   icon: 'messages-square',   keywords: 'sessions active'         },
]

export const SLASH_COMMANDS: SlashCommand[] = [
  { key: 'new',      label: '/new',      hint: 'Scaffold a view, skill, agent, …' },
  { key: 'chats',    label: '/chats',    hint: 'Open Mission Control'              },
  { key: 'settings', label: '/settings', hint: 'Jump to settings'                 },
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
  history.pushState({}, '', path)
  document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
}
