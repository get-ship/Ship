// chrome/terminal-bridge.ts — connects the dock's <sui-terminal> to a
// real shell PTY session on the daemon.
//
// First activation of the terminal tab spawns a shell session under
// the active workspace, opens an EventSource for its output, and
// forwards keystrokes back as input. Closes everything on exit.
//
// Backend contract (apps/shipd/src/sessions/api):
//   POST /api/v1/sessions/spawn  { workspace_id, kind:'shell' } → { session_id }
//   GET  /api/v1/sessions/{id}/stream                            ← SSE bytes
//   POST /api/v1/sessions/{id}/input  { data }
//   POST /api/v1/sessions/{id}/resize { cols, rows }
//   DELETE /api/v1/sessions/{id}

interface TerminalEl extends HTMLElement {
  write?: (data: string) => void
  writeln?: (data: string) => void
  clear?: () => void
  focus?: () => void
}

let bridged = false
let sessionId: string | null = null
let sse: EventSource | null = null
let dataListener: ((e: Event) => void) | null = null

async function getActiveWorkspaceId(): Promise<string | null> {
  try {
    const r = await fetch('/api/v1/projects/active', { credentials: 'same-origin' })
    if (!r.ok) return null
    const b = await r.json() as { active?: { id?: string } | null }
    return b.active?.id ?? null
  } catch { return null }
}

async function spawnShell(): Promise<string | null> {
  const workspace_id = await getActiveWorkspaceId()
  if (!workspace_id) return null
  try {
    const r = await fetch('/api/v1/sessions/spawn', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ workspace_id, kind: 'shell' }),
    })
    if (!r.ok) {
      const text = await r.text().catch(() => '')
      console.warn('[terminal] spawn failed:', r.status, text)
      return null
    }
    const b = await r.json() as { session_id?: string }
    return b.session_id ?? null
  } catch (e) {
    console.warn('[terminal] spawn threw:', e)
    return null
  }
}

function openStream(termEl: TerminalEl, id: string): EventSource {
  const es = new EventSource(`/api/v1/sessions/${encodeURIComponent(id)}/stream`)
  // Backend sends `event: bytes\ndata: <raw text>` per PTY output chunk.
  // EventSource ignores the trailing newline so multi-line escape
  // sequences arrive intact. Write straight into xterm.
  const writeChunk = (data: unknown): void => {
    if (typeof data !== 'string') return
    try { termEl.write?.(data) } catch { /* ignore */ }
  }
  es.addEventListener('bytes', (e) => writeChunk((e as MessageEvent).data))
  es.addEventListener('chunk', (e) => writeChunk((e as MessageEvent).data))
  es.addEventListener('output', (e) => writeChunk((e as MessageEvent).data))
  es.onmessage = (m) => writeChunk(m.data)
  es.onerror = () => {
    // EventSource auto-reconnects; nothing to do but note the dip.
    termEl.write?.('\r\n[stream disconnected — reconnecting]\r\n')
  }
  return es
}

async function sendInput(id: string, data: string): Promise<void> {
  try {
    await fetch(`/api/v1/sessions/${encodeURIComponent(id)}/input`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ data }),
    })
  } catch { /* fire-and-forget */ }
}

export async function bridgeDockTerminal(): Promise<void> {
  if (bridged) return
  const termEl = document.querySelector<TerminalEl>('sui-terminal[slot="terminal"]')
    ?? document.getElementById('ship-panel-dock')?.querySelector<TerminalEl>(':scope > sui-terminal')
  if (!termEl) return
  bridged = true

  const id = await spawnShell()
  if (!id) {
    termEl.write?.('\r\n[no shell — could not spawn PTY session]\r\n')
    bridged = false
    return
  }
  sessionId = id
  // Open output stream.
  sse = openStream(termEl, id)
  // Forward keystrokes.
  dataListener = (e: Event) => {
    const d = (e as CustomEvent<{ data?: string }>).detail?.data
    if (typeof d === 'string') void sendInput(id, d)
  }
  termEl.addEventListener('sui-terminal-data', dataListener)
}

export async function disposeDockTerminal(): Promise<void> {
  if (!bridged) return
  bridged = false
  const termEl = document.querySelector<TerminalEl>('sui-terminal[slot="terminal"]')
  if (termEl && dataListener) termEl.removeEventListener('sui-terminal-data', dataListener)
  dataListener = null
  sse?.close()
  sse = null
  if (sessionId) {
    try {
      await fetch(`/api/v1/sessions/${encodeURIComponent(sessionId)}`, {
        method: 'DELETE',
        credentials: 'same-origin',
      })
    } catch { /* best-effort */ }
  }
  sessionId = null
}
