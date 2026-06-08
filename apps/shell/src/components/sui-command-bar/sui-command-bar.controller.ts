import { controller, signal, derived, effect, ref, shared } from '@ship/sui-runtime'
import type { Signal, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-command-bar.sui'
import {
  type BarMode, type SessionSummary, type AgentSummary, type ThreadLine, type CreateSurface,
  type SlashCommand, type SearchHit,
  loadSlashCommands, cachedSlashCommands, slashCacheAge, SLASH_CACHE_TTL_MS,
  searchEntities,
  detectMode, navigateTo,
} from './types'
import { ops, readSetting } from '../../ops'

export default controller({
  // `open`/`mode` are `<prop>`s the runtime injects onto the scope before
  // this factory runs (component-factory: `$scope.open = props.open`). The
  // compiler omits them from the generated `Scope` because their names
  // collide with the `shared()` re-exports this factory returns, so declare
  // their injected signal types on the `init` parameter — `Ctx` is inferred
  // from this annotation, so the contract stays precise and cast-free.
  scope: (init: ViewScope<Scope> & { open: Signal<boolean>; mode: Signal<BarMode> }) => {
    // Wrap prop signals as shared so consumers can access them.
    const open  = shared(init.open)
    const mode  = shared(init.mode)

    const value          = shared(signal(''))
    const session        = shared(signal<SessionSummary | null>(null))
    const agents         = shared(signal<AgentSummary[]>([]))
    const harnesses      = shared(signal<string[]>([]))
    const allSessions    = shared(signal<SessionSummary[]>([]))
    const views          = shared(signal<string[]>([]))
    const selectedIdx    = shared(signal(0))
    const sessionPickerOpen = shared(signal(false))
    const spawnAgent     = shared(signal(''))
    const spawnHarness  = shared(signal(''))
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
    const daemonOnline   = shared(signal(true))
    const recentSends    = shared(signal<string[]>([]))
    const historyIdx     = shared(signal(-1))
    const sendError      = shared(signal<string | null>(null))

    // ai mode owns the chat surface even with no session. The chat panel
    // renders a spawn / no-agents empty state inline so a prefill from
    // `ship:bar:open` is always visible above the same composer.
    const isChat = shared(derived(() => mode() === 'ai'))

    const dialogRef = ref<HTMLElement>(null)

    const triggerLabel = derived<string>(() => {
      const sess = session()
      return sess ? `Ask ${sess.agent_id ?? 'Ship'} or jump to…` : 'Ask Ship or jump to…'
    })
    const dotBg = derived<string>(() => {
      const sess = session()
      if (sess?.status === 'thinking') return 'var(--warning,#d97706)'
      if (sess) return 'var(--success,#16a34a)'
      return 'var(--muted-foreground)'
    })

    return {
      open, mode, value, session, agents, harnesses, allSessions, views,
      selectedIdx, sessionPickerOpen, spawnAgent, spawnHarness, spawning,
      spawnError, thread, workspaceId, createSurface, createName, createPurpose,
      createShape, createSubmitting, createError, isChat, dialogRef,
      slashCommands, sendKey, searchHits, searching,
      daemonOnline, recentSends, historyIdx, sendError,
      triggerLabel, dotBg,
    }
  },

  on: {
    // Called by trigger button click.
    openFind: (s: any) => { s.mode = 'find'; s.value = ''; s.selectedIdx = 0; s.open = true },
  },

  mount: (state, host) => {
    const open         = state.$signals.open
    const mode         = state.$signals.mode
    const value        = state.$signals.value
    const session      = state.$signals.session
    const agents       = state.$signals.agents
    const harnesses    = state.$signals.harnesses
    const allSessions  = state.$signals.allSessions
    const views        = state.$signals.views
    const selectedIdx  = state.$signals.selectedIdx
    const spawnAgent   = state.$signals.spawnAgent
    const spawnHarness = state.$signals.spawnHarness
    const thread: Signal<ThreadLine[]>   = state.$signals.thread
    const slashCommands: Signal<SlashCommand[]> = state.$signals.slashCommands
    const sendKey          = state.$signals.sendKey
    const searchHits       = state.$signals.searchHits
    const searching        = state.$signals.searching
    const daemonOnline     = state.$signals.daemonOnline

    const dialogRef = state.$signals.dialogRef?.() as (HTMLElement & { show(): void; close(): void }) | null

    let streamAbort: AbortController | null = null
    let lastSpaceAt = 0

    // ── Slash registry + send-key setting loaders ─────────────────
    async function refreshSlashCommands(force = false): Promise<void> {
      const list = await loadSlashCommands(force)
      slashCommands.set(list)
    }
    async function refreshSendKey(): Promise<void> {
      const v = await readSetting<unknown>('cmdk.send_key')
      if (v === 'enter' || v === 'cmd+enter') sendKey.set(v)
    }

    // ── Open/close effect ──────────────────────────────────────────
    // The first effect run fires synchronously during this controller's
    // mount, before the child <sui-dialog>'s connectedCallback has had
    // a chance to attach `.show`/`.close` to its host. Calling close()
    // on the bare element throws "is not a function". Guard with
    // typeof checks so the initial closed → closed no-op is silent and
    // subsequent transitions go through cleanly once the dialog is
    // ready.
    function focusActivePane(): void {
      // Defer to next microtask so the {#if isChat()} branch has rendered
      // and the relevant child is in the DOM. ai mode focuses the chat
      // textarea (and selects any prefill so the user can immediately
      // type to replace, or arrow-key edit); all other modes focus the
      // palette search input.
      queueMicrotask(() => {
        if (mode() === 'ai') {
          const chat = host.shadowRoot?.querySelector('sui-command-chat')
          const ta = chat?.shadowRoot?.querySelector('.cb-chat-input') as HTMLTextAreaElement | null
          if (ta) {
            ta.focus()
            if (ta.value.length > 0) ta.select()
          }
          return
        }
        const pal = host.shadowRoot?.querySelector('sui-command-palette')
        const inp = pal?.shadowRoot?.querySelector('.cb-search') as HTMLInputElement | null
        inp?.focus()
      })
    }

    const stopOpenSync = effect(() => {
      const isOpen = open()
      host.setAttribute('data-state', isOpen ? 'open' : 'closed')
      if (isOpen) {
        if (typeof dialogRef?.show === 'function') dialogRef.show()
        if (slashCacheAge() > SLASH_CACHE_TTL_MS) void refreshSlashCommands()
        focusActivePane()
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
      es.addEventListener('harness', (ev) => {
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

    // Re-focus the right input whenever mode flips while the bar is open
    // (e.g., send-to-agent prefill flips mode→'ai' on an already-open bar).
    const stopModeFocusSync = effect(() => {
      const m = mode()
      if (open()) focusActivePane()
      // touch m to register dep
      void m
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

    // ── Daemon online indicator ────────────────────────────────────
    // Any successful fetch flips back to online; a network error on a
    // core endpoint flips offline. Used by the chat empty state to show
    // a "daemon offline" banner instead of failing silently.
    function markOnline(): void { if (!daemonOnline()) daemonOnline.set(true) }
    function markOffline(): void { if (daemonOnline()) daemonOnline.set(false) }

    // ── Async data loading ─────────────────────────────────────────
    async function loadCurrentSession(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
        if (resp.ok) {
          markOnline()
          const body = await resp.json() as { sessions: SessionSummary[] }
          allSessions.set(body.sessions)
          const active = body.sessions.find((s) => s.status !== 'ended' && s.agent_id)
          if (active && active.id !== session()?.id) { session.set(active); thread.set([]) }
          else if (!active) { session.set(null) }
        } else {
          markOffline()
        }
      } catch { markOffline() }
      if (agents().length === 0) void loadAgents()
      if (harnesses().length === 0) void loadProviders()
      if (views().length === 0) void loadViews()
    }

    async function loadAgents(): Promise<void> {
      try {
        const res = await ops.agents.list({})
        const list: AgentSummary[] = res.items.map((a) => ({ id: a.id }))
        agents.set(list)
        if (!spawnAgent() && list.length > 0) spawnAgent.set(list[0]!.id)
      } catch { /* empty */ }
    }

    async function loadProviders(): Promise<void> {
      try {
        const res = await ops.agents.harnesses({})
        harnesses.set(res.items)
        if (!spawnHarness() && res.items.length > 0) spawnHarness.set(res.items[0]!)
      } catch { /* empty */ }
    }

    async function loadViews(): Promise<void> {
      // Views are client-side routes; the daemon discovery catalog was quarantined
      // (06153c1a5). Cut the dead op — the palette's view list stays empty until
      // the router→view-list integration lands.
      views.set([])
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
      const needle = v.replace(/^\/+\s*/, '').toLowerCase().trim()
      const cmds = slashCommands()
      const cmd = cmds.find((c) => c.key === needle) ?? cmds.find((c) => c.key.startsWith(needle))
      if (!cmd) return
      value.set('')
      // Mirror sui-command-palette#activateCmd. Keep these in sync — Enter
      // (here) and click (palette) are the two activation paths into the
      // same slash registry; missing branches here silently no-op'd /theme
      // and /lint on Enter pre-fix.
      switch (cmd.key) {
        case 'new': mode.set('create'); break
        case 'chats': void navigateTo('/chats'); open.set(false); break
        case 'settings': void navigateTo('/settings'); open.set(false); break
        case 'theme': void navigateTo('/theme-studio'); open.set(false); break
        case 'lint': void navigateTo('/lint'); open.set(false); break
        case 'end': if (session()) void endSession(session()!.id); open.set(false); break
        case 'close': open.set(false); break
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
    // Bar-open + create-open events: bubble on document since the
    // legacy <ship-shell>/<sui-shell> host no longer exists in this
    // shell. Listening on the document means anything in the page
    // (corner edit pill, view-picker, etc.) reaches the cmdk.
    document.addEventListener('ship:bar:open', onBarOpen)
    document.addEventListener('ship:create:open', onCreateOpen)

    // Expose endSession + loadCurrentSession on the shared context
    // so palette/chat controllers can call them.
    const ctxKey = '__sui_ctx_sui-command-bar'
    const ctx = (host as unknown as Record<string, unknown>)[ctxKey] as Record<string, unknown> | undefined
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
      document.removeEventListener('ship:bar:open', onBarOpen)
      document.removeEventListener('ship:create:open', onCreateOpen)
      if (dialogRef) dialogRef.removeEventListener('sui-close', onDialogClose)
      cancelSearch()
      stopStreamSync()
      stopSearchSync()
      stopOpenSync()
      stopModeFocusSync()
    }
  },
})
