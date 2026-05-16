import { controller, signal, derived, effect, ref, shared } from '@ship/sui-runtime'
import type { Scope } from './sui-command-bar.sui'
import type { Signal } from '@ship/sui-runtime'
import {
  type BarMode, type SessionSummary, type AgentSummary, type ThreadLine, type CreateSurface,
  type SlashCommand, type SearchHit,
  SLASH_COMMANDS, CREATE_SURFACES,
  loadSlashCommands, cachedSlashCommands, slashCacheAge, SLASH_CACHE_TTL_MS,
  searchEntities,
  detectMode, isKebab, navigateTo,
} from './types'

// Full scope shape for use in mount: casts.
type BarState = {
  open: Signal<boolean>; mode: Signal<BarMode>; value: Signal<string>
  session: Signal<SessionSummary | null>; agents: Signal<AgentSummary[]>
  providers: Signal<string[]>; allSessions: Signal<SessionSummary[]>
  views: Signal<string[]>; selectedIdx: Signal<number>
  sessionPickerOpen: Signal<boolean>; spawnAgent: Signal<string>
  spawnProvider: Signal<string>; spawning: Signal<boolean>
  spawnError: Signal<string | null>; thread: Signal<ThreadLine[]>
  workspaceId: Signal<string>; createSurface: Signal<CreateSurface>
  createName: Signal<string>; createPurpose: Signal<string>
  createShape: Signal<string>; createSubmitting: Signal<boolean>
  createError: Signal<string | null>; dialogRef: Signal<HTMLElement | null>
  slashCommands: Signal<SlashCommand[]>; sendKey: Signal<'enter' | 'cmd+enter'>
  searchHits: Signal<SearchHit[]>; searching: Signal<boolean>
}

export default controller<Scope>({
  scope: (init) => {
    const initAny = init as unknown as BarState
    // Wrap prop signals as shared so consumers can access them.
    const open  = shared(initAny.open)
    const mode  = shared(initAny.mode as Signal<BarMode>)

    const value          = shared(signal(''))
    const session        = shared(signal<SessionSummary | null>(null))
    const agents         = shared(signal<AgentSummary[]>([]))
    const providers      = shared(signal<string[]>([]))
    const allSessions    = shared(signal<SessionSummary[]>([]))
    const views          = shared(signal<string[]>([]))
    const selectedIdx    = shared(signal(0))
    const sessionPickerOpen = shared(signal(false))
    const spawnAgent     = shared(signal(''))
    const spawnProvider  = shared(signal(''))
    const spawning       = shared(signal(false))
    const spawnError     = shared(signal<string | null>(null))
    const thread         = shared(signal<ThreadLine[]>([]))
    const workspaceId    = shared(signal('main'))
    const createSurface  = shared(signal<CreateSurface>('view'))
    const createName     = shared(signal(''))
    const createPurpose  = shared(signal(''))
    const createShape    = shared(signal('list'))
    const createSubmitting = shared(signal(false))
    const createError    = shared(signal<string | null>(null))
    const slashCommands  = shared(signal<SlashCommand[]>(cachedSlashCommands()))
    const sendKey        = shared(signal<'enter' | 'cmd+enter'>('enter'))
    const searchHits     = shared(signal<SearchHit[]>([]))
    const searching      = shared(signal(false))

    const isChat = shared(derived(() => mode() === 'ai' && session() !== null))

    const dialogRef = ref<HTMLElement>(null)

    return {
      open, mode, value, session, agents, providers, allSessions, views,
      selectedIdx, sessionPickerOpen, spawnAgent, spawnProvider, spawning,
      spawnError, thread, workspaceId, createSurface, createName, createPurpose,
      createShape, createSubmitting, createError, isChat, dialogRef,
      slashCommands, sendKey, searchHits, searching,
    }
  },

  derived: {
    triggerLabel: (s: any) => {
      const sess = s.session()
      return sess ? `Ask ${sess.agent_id ?? 'Ship'} or jump to…` : 'Ask Ship or jump to…'
    },
    dotBg: (s: any) => {
      const sess = s.session()
      if (sess?.status === 'thinking') return 'var(--warning,#d97706)'
      if (sess) return 'var(--success,#16a34a)'
      return 'var(--muted-foreground)'
    },
  },

  on: {
    // Called by trigger button click.
    openFind: (s: any) => { s.mode = 'find'; s.value = ''; s.selectedIdx = 0; s.open = true },
  },

  mount: (state: any, host) => {
    const st = state as unknown as BarState
    const open  = st.open
    const mode  = st.mode
    const value = st.value
    const session      = st.session
    const agents       = st.agents
    const providers    = st.providers
    const allSessions  = st.allSessions
    const views        = st.views
    const selectedIdx  = st.selectedIdx
    const spawnAgent   = st.spawnAgent
    const spawnProvider = st.spawnProvider
    const spawning     = st.spawning
    const spawnError   = st.spawnError
    const thread       = st.thread
    const workspaceId  = st.workspaceId
    const createSurface    = st.createSurface
    const createName       = st.createName
    const createPurpose    = st.createPurpose
    const createShape      = st.createShape
    const createSubmitting = st.createSubmitting
    const createError      = st.createError
    const slashCommands    = st.slashCommands
    const sendKey          = st.sendKey
    const searchHits       = st.searchHits
    const searching        = st.searching

    const dialogRef = st.dialogRef?.() as (HTMLElement & { show(): void; close(): void }) | null

    let streamAbort: AbortController | null = null
    let lastSpaceAt = 0

    // ── Slash registry + send-key setting loaders ─────────────────
    async function refreshSlashCommands(force = false): Promise<void> {
      const list = await loadSlashCommands(force)
      slashCommands.set(list)
    }
    async function refreshSendKey(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/settings/cmdk.send_key', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = await resp.json() as { value?: unknown }
        const v = typeof body.value === 'string' ? body.value : (body as { ok?: boolean; data?: { value?: string } }).data?.value
        if (v === 'enter' || v === 'cmd+enter') sendKey.set(v)
      } catch { /* leave default */ }
    }

    // ── Open/close effect ──────────────────────────────────────────
    // The first effect run fires synchronously during this controller's
    // mount, before the child <sui-dialog>'s connectedCallback has had
    // a chance to attach `.show`/`.close` to its host. Calling close()
    // on the bare element throws "is not a function". Guard with
    // typeof checks so the initial closed → closed no-op is silent and
    // subsequent transitions go through cleanly once the dialog is
    // ready.
    const stopOpenSync = effect(() => {
      const isOpen = open()
      host.setAttribute('data-state', isOpen ? 'open' : 'closed')
      if (isOpen) {
        if (typeof dialogRef?.show === 'function') dialogRef.show()
        if (slashCacheAge() > SLASH_CACHE_TTL_MS) void refreshSlashCommands()
        queueMicrotask(() => {
          const pal = host.shadowRoot?.querySelector('sui-command-palette')
          const inp = pal?.shadowRoot?.querySelector('.cb-search') as HTMLInputElement | null
          inp?.focus()
        })
      } else {
        if (typeof dialogRef?.close === 'function') dialogRef.close()
        host.dispatchEvent(new CustomEvent('sui-command:cancel', {
          detail: { mode: mode() }, bubbles: true, composed: true,
        }))
      }
    })

    // ── Public API ─────────────────────────────────────────────────
    type CB = { show(m?: string, q?: string): void; hide(): void; toggle(m?: string): void; setMode(m: string): void }
    ;(host as unknown as CB).show = (m?: string, q?: string) => {
      if (m) mode.set(m as BarMode)
      if (typeof q === 'string') value.set(q)
      open.set(true)
    }
    ;(host as unknown as CB).hide = () => open.set(false)
    ;(host as unknown as CB).toggle = (m?: string) => {
      if (open()) { open.set(false) } else {
        if (m) mode.set(m as BarMode)
        open.set(true)
      }
    }
    ;(host as unknown as CB).setMode = (m: string) => mode.set(m as BarMode)

    // ── SSE stream management ──────────────────────────────────────
    function attachStream(sessionId: string): void {
      detachStream()
      const ac = new AbortController()
      streamAbort = ac
      const es = new EventSource(`/api/v1/sessions/${sessionId}/events`, { withCredentials: true })
      let currentAgentId: string | null = null
      es.addEventListener('provider', (ev) => {
        let parsed: { kind?: string; text?: string; tool?: string }
        try { parsed = JSON.parse((ev as MessageEvent).data) } catch { return }
        switch (parsed.kind) {
          case 'message': {
            const chunk = parsed.text ?? ''
            if (!chunk) return
            if (currentAgentId) {
              thread.set(thread().map((m) => m.id === currentAgentId ? { ...m, text: m.text + chunk } : m))
            } else {
              const id = String(Date.now() + Math.random())
              currentAgentId = id
              thread.set([...thread(), { id, sender: 'agent', text: chunk }])
            }
            break
          }
          case 'tool_use':
            thread.set([...thread(), { id: String(Date.now() + Math.random()), sender: 'system', text: `→ tool: ${parsed.tool ?? '?'}` }])
            break
          case 'turn_complete': currentAgentId = null; break
        }
      })
      es.onerror = () => { if (!ac.signal.aborted) es.close() }
      ac.signal.addEventListener('abort', () => es.close())
    }
    function detachStream(): void { streamAbort?.abort(); streamAbort = null }

    // Auto-attach stream when session changes.
    const stopStreamSync = effect(() => {
      const s = session()
      if (s?.id) attachStream(s.id)
      else detachStream()
    })

    // ── Find-mode unified search ───────────────────────────────────
    // Debounce 200ms so each keystroke doesn't fire a request. Cancel
    // any in-flight search on a new keystroke. Cleared when the bar
    // closes or the mode flips away from 'find'.
    let searchTimer: ReturnType<typeof setTimeout> | null = null
    let searchAbort: AbortController | null = null
    function cancelSearch(): void {
      if (searchTimer) { clearTimeout(searchTimer); searchTimer = null }
      if (searchAbort) { searchAbort.abort(); searchAbort = null }
    }
    async function runSearch(query: string): Promise<void> {
      cancelSearch()
      const ac = new AbortController()
      searchAbort = ac
      searching.set(true)
      const resp = await searchEntities(query, { limit: 30, signal: ac.signal })
      if (ac.signal.aborted) return
      searching.set(false)
      searchAbort = null
      searchHits.set(resp?.data?.hits ?? [])
    }
    const stopSearchSync = effect(() => {
      const isOpen = open()
      const currentMode = mode()
      const q = value().replace(/^>+\s*/, '').trim()
      if (!isOpen || currentMode !== 'find') {
        cancelSearch()
        return
      }
      if (searchTimer) clearTimeout(searchTimer)
      searchTimer = setTimeout(() => { void runSearch(q) }, 200)
    })

    // ── Async data loading ─────────────────────────────────────────
    async function loadCurrentSession(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
        if (resp.ok) {
          const body = await resp.json() as { sessions: SessionSummary[] }
          allSessions.set(body.sessions)
          const active = body.sessions.find((s) => s.status !== 'ended' && s.agent_id)
          if (active && active.id !== session()?.id) { session.set(active); thread.set([]) }
          else if (!active) { session.set(null) }
        }
      } catch { /* leave null */ }
      if (agents().length === 0) void loadAgents()
      if (providers().length === 0) void loadProviders()
      if (views().length === 0) void loadViews()
    }

    async function loadAgents(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/agents', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = await resp.json() as { items?: Array<{ id: string }> }
        const list: AgentSummary[] = (body.items ?? []).map((a) => ({ id: a.id }))
        agents.set(list)
        if (!spawnAgent() && list.length > 0) spawnAgent.set(list[0]!.id)
      } catch { /* empty */ }
    }

    async function loadProviders(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/providers/installed', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = await resp.json() as { providers: string[] }
        providers.set(body.providers)
        if (!spawnProvider() && body.providers.length > 0) spawnProvider.set(body.providers[0]!)
      } catch { /* empty */ }
    }

    async function loadViews(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/views', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = await resp.json() as { data?: { views?: Array<{ name: string }> }; views?: Array<{ name: string }> }
        const list = body.data?.views ?? body.views ?? []
        views.set(list.map((v) => v.name).sort())
      } catch { /* empty */ }
    }

    // ── Keyboard navigation helpers ────────────────────────────────
    function getPaletteItems(): NodeListOf<Element> | undefined {
      const pal = host.shadowRoot?.querySelector('sui-command-palette')
      return pal?.shadowRoot?.querySelectorAll('.cb-item')
    }

    function moveSelection(dir: number): void {
      const items = getPaletteItems()
      if (!items?.length) return
      selectedIdx.set(Math.max(0, Math.min(items.length - 1, selectedIdx() + dir)))
    }

    function confirmSelection(): void {
      const m = mode()
      const v = value()
      const items = getPaletteItems()
      const active = items?.[selectedIdx()] as HTMLElement | undefined
      if (active) {
        const path = active.dataset['path']
        if (path) { void navigateTo(path); value.set(''); open.set(false) }
        const cmd = active.dataset['cmd']
        if (cmd) { value.set(''); runSlashCommand('/' + cmd) }
        const mKind = active.dataset['mentionKind'] as 'view' | 'agent' | 'session' | undefined
        const mId = active.dataset['mentionId']
        if (mKind && mId) {
          const ref = mKind === 'view' ? `@${mId}` : `@${mKind}:${mId}`
          value.set(v.replace(/@[^@\s]*$/, `${ref} `))
          mode.set(detectMode(value(), m))
        }
        host.dispatchEvent(new CustomEvent('sui-command:select', {
          detail: { mode: m, item: active.dataset }, bubbles: true, composed: true,
        }))
        return
      }
      if (m === 'command') runSlashCommand(v)
      else if (m === 'find') navigateFirstHit()
    }

    function runSlashCommand(v: string): void {
      const needle = v.replace(/^\/+\s*/, '').toLowerCase()
      const cmds = slashCommands()
      const cmd = cmds.find((c) => c.key === needle) ?? cmds.find((c) => c.key.startsWith(needle))
      if (!cmd) return
      switch (cmd.key) {
        case 'new': mode.set('create'); value.set(''); break
        case 'chats': void navigateTo('/chats'); open.set(false); break
        case 'settings': void navigateTo('/settings'); open.set(false); break
        case 'end': if (session()) void endSession(session()!.id); open.set(false); break
        case 'close': open.set(false); break
        case 'lint': void runLint(); break
      }
    }

    function navigateFirstHit(): void {
      const top = searchHits()[0]
      if (top) { void navigateTo(top.path); value.set(''); open.set(false) }
    }

    async function endSession(id: string): Promise<void> {
      try { await fetch(`/api/v1/sessions/${id}`, { method: 'DELETE', credentials: 'same-origin' }) } catch { /* best-effort */ }
      if (session()?.id === id) { session.set(null) }
      await loadCurrentSession()
    }

    async function runLint(): Promise<void> {
      const current = (window as Record<string, unknown> & { ship?: { current?: () => { path?: string } } }).ship?.current?.()
      const name = current?.path?.replace(/^\/+/, '').split('/')[0] ?? ''
      if (!name) return
      try {
        const resp = await fetch('/api/v1/lint/view', {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name }),
        })
        if (resp.ok) {
          const body = await resp.json() as { data?: { findings?: unknown[] } }
          const count = body.data?.findings?.length ?? 0
          console.log(count === 0 ? `${name}: no lint findings` : `${name}: ${count} lint findings`)
        }
      } catch { /* silent */ }
      open.set(false)
    }

    // ── Global keyboard listener ───────────────────────────────────
    // Cmd/Ctrl+K is owned by the shell (apps/shell/src/main.ts) which
    // calls `host.toggle('find')` on this element. Both listeners
    // running together raced — the cb listener flipped open=true,
    // then shell's listener (also seeing the same keypress) called
    // toggle() which flipped it straight back to false 23ms later.
    // Keep Escape (close) and double-space (open in AI mode) here
    // since those are intrinsic to the bar, not shell-wide bindings.
    const onDocKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && open()) { open.set(false); return }
      if (e.key === ' ' && !open()) {
        const t = e.target as HTMLElement | null
        if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
        const now = performance.now()
        if (now - lastSpaceAt < 350) {
          lastSpaceAt = 0; e.preventDefault()
          mode.set('ai'); value.set(''); open.set(true)
        } else { lastSpaceAt = now }
      }
    }

    // ── Palette input keyboard listener (ArrowDown/Up/Enter) ───────
    const onPaletteKeydown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') { e.preventDefault(); moveSelection(1) }
      else if (e.key === 'ArrowUp') { e.preventDefault(); moveSelection(-1) }
      else if (e.key === 'Enter') { e.preventDefault(); confirmSelection() }
    }
    // Listen on the host since palette keyboard events bubble through shadow.
    host.addEventListener('keydown', onPaletteKeydown)

    document.addEventListener('keydown', onDocKeydown)

    // ── Dialog close listener ──────────────────────────────────────
    const onDialogClose = () => { if (open()) open.set(false) }
    dialogRef?.addEventListener('sui-close', onDialogClose)

    // ── Session + shell event listeners ───────────────────────────
    const onSessionChanged = () => void loadCurrentSession()
    const onBarOpen = (e: Event) => {
      const ce = e as CustomEvent<{ mode?: BarMode; prefill?: string }>
      if (ce.detail?.mode) mode.set(ce.detail.mode)
      if (typeof ce.detail?.prefill === 'string') value.set(ce.detail.prefill)
      open.set(true)
    }
    const onCreateOpen = () => { mode.set('create'); open.set(true) }

    window.addEventListener('ship:session:changed', onSessionChanged)
    const shell = document.querySelector('ship-shell') ?? document.querySelector('sui-shell')
    shell?.addEventListener('ship:bar:open', onBarOpen)
    shell?.addEventListener('ship:create:open', onCreateOpen)

    // Expose endSession + loadCurrentSession on the shared context
    // so palette/chat controllers can call them.
    const ctxKey = '__sui_ctx_sui-command-bar'
    const ctx = (host as Record<string, unknown>)[ctxKey] as Record<string, unknown> | undefined
    if (ctx) {
      ctx['endSession'] = endSession
      ctx['loadCurrentSession'] = loadCurrentSession
    }

    void loadCurrentSession()
    void refreshSlashCommands()
    void refreshSendKey()

    // Reflect immediately on setting change. The daemon emits config.changed
    // events; the shell rebroadcasts them on the document. Cheap to re-poll.
    const onConfigChanged = (e: Event) => {
      const detail = (e as CustomEvent<{ subject?: string }>).detail
      if (!detail || detail.subject === 'cmdk.send_key') void refreshSendKey()
    }
    document.addEventListener('ship:config:changed', onConfigChanged)

    return () => {
      document.removeEventListener('ship:config:changed', onConfigChanged)
      document.removeEventListener('keydown', onDocKeydown)
      host.removeEventListener('keydown', onPaletteKeydown)
      window.removeEventListener('ship:session:changed', onSessionChanged)
      shell?.removeEventListener('ship:bar:open', onBarOpen)
      shell?.removeEventListener('ship:create:open', onCreateOpen)
      if (dialogRef) dialogRef.removeEventListener('sui-close', onDialogClose)
      cancelSearch()
      stopStreamSync()
      stopSearchSync()
      stopOpenSync()
    }
  },
})
