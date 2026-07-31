// Pure filter predicates for the agents list view. Kept dependency-free
// (no @ship/sail-runtime import) so the controller and its unit tests can
// both consume them without the runtime in scope — same split as the
// detail view's hue.ts.

export type AgentScope = 'all' | 'project' | 'user'

export interface FilterableAgent {
  id: string
  name: string
  description?: string
  source: string
  tags?: string[]
}

export function matchesKind(a: FilterableAgent, kind: AgentScope): boolean {
  if (kind === 'all') return true
  if (kind === 'project') return a.source === 'PROJECT AGENTS'
  if (kind === 'user') return a.source === 'USER AGENTS'
  return true
}

export function matchesQuery(a: FilterableAgent, q: string): boolean {
  if (!q) return true
  const needle = q.toLowerCase().trim()
  if (!needle) return true
  if (a.name.toLowerCase().includes(needle)) return true
  if (a.id.toLowerCase().includes(needle)) return true
  if ((a.description || '').toLowerCase().includes(needle)) return true
  for (const t of a.tags ?? []) {
    if (t.toLowerCase().includes(needle)) return true
  }
  return false
}
