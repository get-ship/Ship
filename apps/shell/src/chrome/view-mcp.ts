// chrome/view-mcp.ts — wires the view-authoring MCP surface from chrome
// to the daemon. Replaces the apps/shell2 mock that the VA components
// shipped against before the backend landed.
//
// Lifecycle:
//   1. enterEditMode(view)   POST /api/v1/views/<view>/edit-mode/enter
//                            grants the chrome's session_id access to
//                            the view.* tools for this view.
//   2. dispatch(view, op, args)
//                            POST /api/v1/views/<view>/edit/tool with
//                            { session_id, op, args }. Returns the
//                            response body verbatim.
//   3. exitEditMode(view)    POST /api/v1/views/<view>/edit-mode/exit
//
// The chrome's session_id is a one-per-tab UUID persisted to
// sessionStorage so the grant survives a soft reload. It is distinct
// from agent sessions spawned by cmdk.

const SESSION_STORAGE_KEY = 'ship.chrome.view-mcp.session_id'

interface ViewMcp {
  sessionId(): string
  enterEditMode(view: string): Promise<void>
  exitEditMode(view: string): Promise<void>
  readSource(view: string): Promise<{ source: string; content_hash: string }>
  writeSource(view: string, source: string, turn_label?: string): Promise<{
    snapshot_id?: string
    content_hash: string
    compile_ok: boolean
    errors?: Array<{ file: string; line: number; message: string }>
  }>
  compileCheck(view: string): Promise<{
    ok: boolean
    errors?: Array<{ file: string; line: number; message: string }>
  }>
  listSiblings(view: string): Promise<Array<{ name: string; path: string; summary?: string }>>
  themeTokens(view: string): Promise<Record<string, string>>
  listSnapshots(view: string, limit?: number, cursor?: string): Promise<{
    snapshots: Array<{
      snapshot_id: string
      view_name: string
      turn_id: string
      agent_id: string
      timestamp: string
      label?: string
      content_hash: string
    }>
    cursor?: string
  }>
  restoreSnapshot(view: string, snapshot_id: string): Promise<{
    content_hash: string
    compile_ok: boolean
    errors?: Array<{ file: string; line: number; message: string }>
  }>
}

declare global {
  interface Window { __viewMcp?: ViewMcp }
}

function ensureSessionId(): string {
  let id: string | null = null
  try { id = sessionStorage.getItem(SESSION_STORAGE_KEY) } catch { /* private mode */ }
  if (id && id.length > 0) return id
  id = (crypto?.randomUUID?.() ?? `chrome-${Date.now()}-${Math.random().toString(36).slice(2)}`)
  try { sessionStorage.setItem(SESSION_STORAGE_KEY, id) } catch { /* swallow */ }
  return id
}

async function dispatch<T = unknown>(view: string, op: string, args: Record<string, unknown> = {}): Promise<T> {
  const session_id = ensureSessionId()
  const resp = await fetch(
    `/api/v1/views/${encodeURIComponent(view)}/edit/tool`,
    {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_id, op, args }),
    },
  )
  if (!resp.ok) {
    const text = await resp.text().catch(() => '')
    throw new Error(`view.${op} failed: ${resp.status} ${text}`)
  }
  return await resp.json() as T
}

async function postLifecycle(view: string, kind: 'enter' | 'exit'): Promise<void> {
  const session_id = ensureSessionId()
  const resp = await fetch(
    `/api/v1/views/${encodeURIComponent(view)}/edit-mode/${kind}`,
    {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_id }),
    },
  )
  if (!resp.ok) {
    const text = await resp.text().catch(() => '')
    throw new Error(`edit-mode/${kind} failed: ${resp.status} ${text}`)
  }
}

const mcp: ViewMcp = {
  sessionId: ensureSessionId,
  enterEditMode: (view) => postLifecycle(view, 'enter'),
  exitEditMode: (view) => postLifecycle(view, 'exit'),
  readSource: (view) => dispatch(view, 'read_source'),
  writeSource: (view, source, turn_label) => dispatch(view, 'write_source', { source, turn_label }),
  compileCheck: (view) => dispatch(view, 'compile_check'),
  listSiblings: (view) => dispatch(view, 'list_siblings').then((r) => (r as { siblings?: unknown[] }).siblings as Array<{ name: string; path: string; summary?: string }> ?? []),
  themeTokens: (view) => dispatch(view, 'theme_tokens').then((r) => (r as { tokens?: Record<string,string> }).tokens ?? {}),
  listSnapshots: (view, limit, cursor) => {
    const args: Record<string, unknown> = {}
    if (typeof limit === 'number') args['limit'] = limit
    if (typeof cursor === 'string') args['cursor'] = cursor
    return dispatch(view, 'list_snapshots', args)
  },
  restoreSnapshot: (view, snapshot_id) => dispatch(view, 'restore_snapshot', { snapshot_id }),
}

export function installViewMcp(): void {
  if (typeof window === 'undefined') return
  window.__viewMcp = mcp
}

export { mcp as viewMcp }
