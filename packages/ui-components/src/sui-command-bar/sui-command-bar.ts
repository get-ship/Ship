import { controller, signal, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-command-bar.sui'
import {
  type BarMode, type SessionSummary, type AgentSummary, type ThreadLine, type CreateSurface,
  CREATE_SURFACES, SLASH_COMMANDS, NAV_DESTINATIONS,
  detectMode, isKebab,
  renderInputRow, renderSessionPopover, renderFindBody, renderCommandBody,
  renderMentionBody, renderCreateBody, renderSpawnCard, renderThread, renderChatLayout,
} from './render'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // ── Signals ────────────────────────────────────────────────────
    const mode = signal<BarMode>('ai')
    const value = signal('')
    const session = signal<SessionSummary | null>(null)
    const agents = signal<AgentSummary[]>([])
    const providers = signal<string[]>([])
    const allSessions = signal<SessionSummary[]>([])
    const views = signal<string[]>([])
    const selectedIdx = signal(0)
    const sessionPickerOpen = signal(false)
    const spawnAgent = signal('')
    const spawnProvider = signal('')
    const spawning = signal(false)
    const spawnError = signal<string | null>(null)
    const thread = signal<ThreadLine[]>([])
    const workspaceId = signal('main')
    const createSurface = signal<CreateSurface>('view')
    const createName = signal('')
    const createPurpose = signal('')
    const createShape = signal('list')
    const createSubmitting = signal(false)
    const createError = signal<string | null>(null)
    let streamAbort: AbortController | null = null
    let lastSpaceAt = 0

    // ── Dialog + element references ────────────────────────────────
    const dialogRef = scope.dialogRef() as HTMLElement & { show(): void; close(): void } | null
    const triggerBtn = scope.triggerBtn() as HTMLButtonElement | null
    const triggerLabel = scope.triggerLabel() as HTMLElement | null
    const triggerDot = scope.triggerDot() as HTMLElement | null
    const wrapEl = scope.wrapEl() as HTMLElement | null

    // Active search input and textarea — refreshed after each render
    let searchEl: HTMLInputElement | null = null

    // ── Imperative API ─────────────────────────────────────────────
    type CB = HTMLElement & { show(m?: string, q?: string): void; hide(): void; toggle(m?: string): void; setMode(m: string): void }
    ;(host as CB).show = (m?: string, q?: string) => {
      if (m) mode.set(m as BarMode)
      if (typeof q === 'string') value.set(q)
      scope.open.set(true)
    }
    ;(host as CB).hide = () => scope.open.set(false)
    ;(host as CB).toggle = (m?: string) => {
      if (scope.open()) { scope.open.set(false) } else {
        if (m) mode.set(m as BarMode)
        scope.open.set(true)
      }
    }
    ;(host as CB).setMode = (m: string) => mode.set(m as BarMode)

    // ── Open/close effect ──────────────────────────────────────────
    const stopOpenSync = effect(() => {
      const isOpen = scope.open()
      host.setAttribute('data-state', isOpen ? 'open' : 'closed')
      if (isOpen) {
        dialogRef?.show()
        queueMicrotask(() => { searchEl?.focus() })
      } else {
        dialogRef?.close()
        host.dispatchEvent(new CustomEvent('sui-command:cancel', {
          detail: { mode: mode() }, bubbles: true, composed: true,
        }))
      }
    })

    // ── Reflect mode to attribute ──────────────────────────────────
    const stopModeSync = effect(() => {
      scope.mode.set(mode())
    })

    // ── Trigger label sync ─────────────────────────────────────────
    const stopTrigger = effect(() => {
      const s = session()
      if (triggerLabel) triggerLabel.textContent = s ? `Ask ${s.agent_id ?? 'Ship'} or jump to…` : 'Ask Ship or jump to…'
      if (triggerDot) {
        const cls = s?.status === 'thinking' ? 'thinking' : s ? 'active' : ''
        triggerDot.className = `cb-dot${cls ? ' ' + cls : ''}`
        triggerDot.style.background = cls === 'thinking' ? 'var(--warning,#d97706)' : cls === 'active' ? 'var(--success,#16a34a)' : 'var(--muted-foreground)'
      }
    })

    // ── Main render ────────────────────────────────────────────────
    const stopRender = effect(() => {
      const isOpen = scope.open()
      const m = mode()
      const v = value()
      const s = session()
      const si = selectedIdx()
      const spo = sessionPickerOpen()
      const sp = spawnError()
      const t = thread()
      const ag = agents()
      const pv = providers()
      const vs = views()
      const as_ = allSessions()
      const sa = spawnAgent()
      const spv = spawnProvider()
      const sw = spawning()
      const cs = createSurface()
      const cn = createName()
      const cp = createPurpose()
      const csh = createShape()
      const csub = createSubmitting()
      const ce = createError()

      if (!wrapEl || !isOpen) return

      const inChat = m === 'ai' && s !== null

      if (inChat) {
        wrapEl.innerHTML = renderChatLayout({ session: s!, thread: t, value: v, sessionPickerOpen: spo, allSessions: as_ })
        bindChatEvents(wrapEl, s!)
      } else {
        const bodyHtml = renderModeBody(m, v, si, { views: vs, agents: ag, sessions: as_, spawnAgent: sa, spawnProvider: spv, spawning: sw, spawnError: sp, createSurface: cs, createName: cn, createPurpose: cp, createShape: csh, createSubmitting: csub, createError: ce, thread: t })
        wrapEl.innerHTML = renderInputRow({ mode: m, value: v, session: s }) + `<div class="cb-body">${bodyHtml}</div>`
        searchEl = wrapEl.querySelector('.cb-search')
        if (searchEl) {
          searchEl.addEventListener('input', onSearchInput)
          searchEl.addEventListener('keydown', onInputKeydown)
        }
        bindPaletteEvents(wrapEl, m, v, s, si, as_, ag, vs, pv, sa, spv, cs, cn, csh)
      }
    })

    function renderModeBody(
      m: BarMode, v: string, si: number,
      ctx: { views: string[]; agents: AgentSummary[]; sessions: SessionSummary[]; spawnAgent: string; spawnProvider: string; spawning: boolean; spawnError: string | null; createSurface: CreateSurface; createName: string; createPurpose: string; createShape: string; createSubmitting: boolean; createError: string | null; thread: ThreadLine[] }
    ): string {
      switch (m) {
        case 'find': return renderFindBody(v, si)
        case 'command': return renderCommandBody(v, si)
        case 'mention': return renderMentionBody(v, ctx.views, ctx.agents, ctx.sessions, si)
        case 'create': return renderCreateBody({ surface: ctx.createSurface, name: ctx.createName, purpose: ctx.createPurpose, shape: ctx.createShape, submitting: ctx.createSubmitting, error: ctx.createError })
        case 'ai': return ctx.sessions.length > 0 || session() !== null ? renderThread(ctx.thread) : renderSpawnCard({ agents: ctx.agents, providers: ctx.providers ?? [], spawnAgent: ctx.spawnAgent, spawnProvider: ctx.spawnProvider, spawning: ctx.spawning, spawnError: ctx.spawnError })
        default: return ''
      }
    }

    function bindPaletteEvents(el: HTMLElement, m: BarMode, v: string, s: SessionSummary | null, si: number, as_: SessionSummary[], ag: AgentSummary[], vs: string[], _pv: string[], sa: string, spv: string, cs: CreateSurface, cn: string, csh: string): void {
      // Session chip
      const chip = el.querySelector('.cb-session-chip') as HTMLElement | null
      chip?.addEventListener('click', (e) => { e.stopPropagation(); toggleSessionPicker() })

      // Session popover events (if open)
      const popover = el.querySelector('.cb-session-popover') as HTMLElement | null
      if (popover) {
        popover.querySelectorAll('.cb-session-row').forEach((row) => {
          const id = (row as HTMLElement).dataset.sessionId
          if (id) row.addEventListener('click', () => switchSession(id, as_))
        })
        popover.querySelectorAll('.cb-session-end').forEach((btn) => {
          const id = (btn as HTMLElement).dataset.endId
          if (id) btn.addEventListener('click', (e) => { e.stopPropagation(); void endSession(id) })
        })
        popover.querySelectorAll('[data-action]').forEach((el) => {
          const action = (el as HTMLElement).dataset.action
          if (action === 'new-session') el.addEventListener('click', () => { sessionPickerOpen.set(false); session.set(null); mode.set('ai') })
          if (action === 'view-chats') el.addEventListener('click', () => { sessionPickerOpen.set(false); void navigate('/chats'); scope.open.set(false) })
        })
      }

      // List items
      el.querySelectorAll('.cb-item').forEach((item) => {
        item.addEventListener('click', () => activateItem(item as HTMLElement, m, v, ag, vs, as_))
      })

      // Create form
      const nameIn = el.querySelector('.cb-create-name') as HTMLInputElement | null
      if (nameIn) {
        nameIn.addEventListener('input', () => createName.set(nameIn.value))
        nameIn.addEventListener('keydown', (e) => { if ((e as KeyboardEvent).key === 'Enter' && isKebab(createName())) void submitCreate() })
      }
      const shapeIn = el.querySelector('.cb-create-shape') as HTMLSelectElement | null
      if (shapeIn) shapeIn.addEventListener('change', () => createShape.set(shapeIn.value))
      const purposeIn = el.querySelector('.cb-create-purpose') as HTMLInputElement | null
      if (purposeIn) purposeIn.addEventListener('input', () => createPurpose.set(purposeIn.value))
      const submitBtn = el.querySelector('.cb-create-submit') as HTMLButtonElement | null
      if (submitBtn) submitBtn.addEventListener('click', () => void submitCreate())
      el.querySelectorAll('.cb-surface-btn').forEach((btn) => {
        const surf = (btn as HTMLElement).dataset.surface as CreateSurface | undefined
        if (surf) btn.addEventListener('click', () => pickSurface(surf))
      })

      // Spawn card
      const agentSel = el.querySelector('.cb-spawn-agent') as HTMLSelectElement | null
      if (agentSel) agentSel.addEventListener('change', () => spawnAgent.set(agentSel.value))
      const provSel = el.querySelector('.cb-spawn-provider') as HTMLSelectElement | null
      if (provSel) provSel.addEventListener('change', () => spawnProvider.set(provSel.value))
      const startBtn = el.querySelector('.cb-spawn-start') as HTMLButtonElement | null
      if (startBtn) startBtn.addEventListener('click', () => void startSession())
    }

    function bindChatEvents(el: HTMLElement, s: SessionSummary): void {
      const textarea = el.querySelector('.cb-chat-input') as HTMLTextAreaElement | null
      const sendBtn = el.querySelector('.cb-chat-send') as HTMLButtonElement | null
      searchEl = null

      if (textarea) {
        textarea.addEventListener('input', () => {
          value.set(textarea.value)
          if (sendBtn) sendBtn.disabled = !textarea.value.trim()
        })
        textarea.addEventListener('keydown', (e) => {
          if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); void sendMessage(s) }
          else if (e.key === 'Escape') scope.open.set(false)
        })
      }
      if (sendBtn) sendBtn.addEventListener('click', () => void sendMessage(s))

      // Session chip / switcher
      const chip = el.querySelector('.cb-session-chip') as HTMLElement | null
      chip?.addEventListener('click', (e) => { e.stopPropagation(); toggleSessionPicker() })
      const popover = el.querySelector('.cb-session-popover') as HTMLElement | null
      if (popover) {
        popover.querySelectorAll('.cb-session-row').forEach((row) => {
          const id = (row as HTMLElement).dataset.sessionId
          if (id) row.addEventListener('click', () => switchSession(id, allSessions()))
        })
        popover.querySelectorAll('.cb-session-end').forEach((btn) => {
          const id = (btn as HTMLElement).dataset.endId
          if (id) btn.addEventListener('click', (e) => { e.stopPropagation(); void endSession(id) })
        })
        popover.querySelectorAll('[data-action]').forEach((el) => {
          const action = (el as HTMLElement).dataset.action
          if (action === 'new-session') el.addEventListener('click', () => { sessionPickerOpen.set(false); session.set(null); mode.set('ai') })
          if (action === 'view-chats') el.addEventListener('click', () => { sessionPickerOpen.set(false); void navigate('/chats'); scope.open.set(false) })
        })
      }
    }

    // ── Input event handlers ───────────────────────────────────────
    function onSearchInput(this: HTMLInputElement): void {
      value.set(this.value)
      mode.set(detectMode(this.value, mode()))
      selectedIdx.set(0)
    }

    function onInputKeydown(e: KeyboardEvent): void {
      if (e.key === 'ArrowDown') { e.preventDefault(); moveSelection(1) }
      else if (e.key === 'ArrowUp') { e.preventDefault(); moveSelection(-1) }
      else if (e.key === 'Enter') { e.preventDefault(); confirmSelection() }
      else if (e.key === 'Escape') { e.preventDefault(); scope.open.set(false) }
    }

    function moveSelection(dir: number): void {
      const items = wrapEl?.querySelectorAll('.cb-item')
      if (!items?.length) return
      selectedIdx.set(Math.max(0, Math.min(items.length - 1, selectedIdx() + dir)))
    }

    function confirmSelection(): void {
      const m = mode()
      const v = value()
      const items = wrapEl?.querySelectorAll('.cb-item')
      const active = items?.[selectedIdx()] as HTMLElement | undefined
      if (active) { activateItem(active, m, v, agents(), views(), allSessions()); return }
      // Enter with no selection: try to submit
      if (m === 'command') runSlashCommand(v)
      else if (m === 'find') navigateFirst(v)
    }

    function activateItem(el: HTMLElement, m: BarMode, v: string, ag: AgentSummary[], vs: string[], as_: SessionSummary[]): void {
      const path = el.dataset.path
      if (path) { void navigate(path); value.set(''); scope.open.set(false) }

      const cmd = el.dataset.cmd
      if (cmd) { value.set(''); runSlashCommand('/' + cmd) }

      const mKind = el.dataset.mentionKind as 'view' | 'agent' | 'session' | undefined
      const mId = el.dataset.mentionId
      if (mKind && mId) {
        const ref = mKind === 'view' ? `@${mId}` : `@${mKind}:${mId}`
        const updated = v.replace(/@[^@\s]*$/, `${ref} `)
        value.set(updated)
        mode.set(detectMode(updated, m))
        searchEl = wrapEl?.querySelector('.cb-search') ?? null
        searchEl?.focus()
      }

      host.dispatchEvent(new CustomEvent('sui-command:select', {
        detail: { mode: m, item: el.dataset }, bubbles: true, composed: true,
      }))
    }

    function runSlashCommand(v: string): void {
      const needle = v.replace(/^\/+\s*/, '').toLowerCase()
      const cmd = SLASH_COMMANDS.find((c) => c.key === needle || c.key.startsWith(needle))
      if (!cmd) return

      switch (cmd.key) {
        case 'new': mode.set('create'); value.set(''); break
        case 'chats': void navigate('/chats'); scope.open.set(false); break
        case 'settings': void navigate('/settings'); scope.open.set(false); break
        case 'end': if (session()) void endSession(session()!.id); scope.open.set(false); break
        case 'close': scope.open.set(false); break
        case 'lint': void runLint(); break
      }
    }

    function navigateFirst(v: string): void {
      const needle = v.replace(/^>+\s*/, '').toLowerCase().trim()
      const dest = NAV_DESTINATIONS.find((d) =>
        d.label.toLowerCase().includes(needle) ||
        d.keywords.toLowerCase().includes(needle) ||
        d.path.toLowerCase().includes(needle)
      )
      if (dest) { void navigate(dest.path); value.set(''); scope.open.set(false) }
    }

    function toggleSessionPicker(): void {
      sessionPickerOpen.set(!sessionPickerOpen())
      if (sessionPickerOpen()) void loadCurrentSession()
    }

    function pickSurface(surf: CreateSurface): void {
      createSurface.set(surf)
      const def = CREATE_SURFACES.find((s) => s.id === surf)!
      if (!def.shapes.includes(createShape())) createShape.set(def.shapes[0]!)
    }

    function switchSession(id: string, sessions: SessionSummary[]): void {
      const found = sessions.find((s) => s.id === id)
      if (!found || found.id === session()?.id) { sessionPickerOpen.set(false); return }
      session.set(found)
      thread.set([])
      attachStream(found.id)
      sessionPickerOpen.set(false)
      mode.set('ai')
    }

    // ── Global keyboard handler ────────────────────────────────────
    function onDocKeydown(e: KeyboardEvent): void {
      const meta = e.metaKey || e.ctrlKey
      if (meta && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        if (scope.open()) scope.open.set(false)
        else { mode.set('find'); value.set(''); selectedIdx.set(0); scope.open.set(true) }
        return
      }
      if (e.key === 'Escape' && scope.open()) { scope.open.set(false); return }

      // Space+Space doubletap → open in AI mode (outside inputs)
      if (e.key === ' ' && !scope.open()) {
        const t = e.target as HTMLElement | null
        if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
        const now = performance.now()
        if (now - lastSpaceAt < 350) {
          lastSpaceAt = 0
          e.preventDefault()
          mode.set('ai')
          value.set('')
          scope.open.set(true)
        } else { lastSpaceAt = now }
      }
    }

    document.addEventListener('keydown', onDocKeydown)

    // ── Trigger click ──────────────────────────────────────────────
    triggerBtn?.addEventListener('click', () => {
      mode.set('find')
      value.set('')
      selectedIdx.set(0)
      scope.open.set(true)
    })

    // ── Dialog closed externally (ESC from native dialog) ──────────
    const dialogEl = scope.dialogRef() as EventTarget | null
    const onDialogClose = () => { if (scope.open()) scope.open.set(false) }
    dialogEl?.addEventListener('sui-close', onDialogClose)

    // ── Async data loading ─────────────────────────────────────────
    async function loadCurrentSession(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
        if (resp.ok) {
          const body = await resp.json() as { sessions: SessionSummary[] }
          allSessions.set(body.sessions)
          const active = body.sessions.find((s) => s.status !== 'ended' && s.agent_id)
          if (active && active.id !== session()?.id) { session.set(active); thread.set([]); attachStream(active.id) }
          else if (!active) { session.set(null); detachStream() }
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

    // ── SSE session stream ─────────────────────────────────────────
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

    // ── Session CRUD ───────────────────────────────────────────────
    async function startSession(): Promise<void> {
      if (!spawnAgent() || !spawnProvider()) { spawnError.set('Pick an agent and a provider first.'); return }
      spawning.set(true)
      spawnError.set(null)
      try {
        const resp = await fetch('/api/v1/sessions/spawn', {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workspace_id: workspaceId(), kind: 'agent', agent_id: spawnAgent(), provider: spawnProvider() }),
        })
        if (!resp.ok) { const t = await resp.text().catch(() => ''); spawnError.set(`spawn failed: ${resp.status} ${t}`); return }
        const body = await resp.json() as { session_id: string }
        session.set({ id: body.session_id, workspace_id: workspaceId(), agent_id: spawnAgent(), provider: spawnProvider(), status: 'idle' })
        thread.set([])
        attachStream(body.session_id)
        void loadCurrentSession()
        window.dispatchEvent(new CustomEvent('ship:session:changed'))
      } catch (e) { spawnError.set(e instanceof Error ? e.message : String(e)) }
      finally { spawning.set(false) }
    }

    async function endSession(id: string): Promise<void> {
      try { await fetch(`/api/v1/sessions/${id}`, { method: 'DELETE', credentials: 'same-origin' }) } catch { /* best-effort */ }
      if (session()?.id === id) { session.set(null); detachStream() }
      await loadCurrentSession()
    }

    async function sendMessage(s: SessionSummary): Promise<void> {
      const text = value().trim()
      if (!text) return
      thread.set([...thread(), { id: String(Date.now()), sender: 'user', text }])
      value.set('')
      try {
        await fetch(`/api/v1/sessions/${s.id}/message`, {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text }),
        })
      } catch (e) {
        thread.set([...thread(), { id: String(Date.now()), sender: 'system', text: `send failed: ${e instanceof Error ? e.message : String(e)}` }])
      }
    }

    async function submitCreate(): Promise<void> {
      if (!isKebab(createName())) { createError.set('Name must be kebab-case.'); return }
      createSubmitting.set(true)
      createError.set(null)
      try {
        const resp = await fetch(`/api/v1/create/${encodeURIComponent(createSurface())}`, {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: createName(), purpose: createPurpose(), shape: createShape() }),
        })
        if (!resp.ok) { const t = await resp.text().catch(() => ''); createError.set(`create failed: ${resp.status} ${t}`); return }
        const body = await resp.json() as { ok?: boolean; data?: { route?: string }; error?: string }
        if (body.ok === false) { createError.set(body.error ?? 'create failed'); return }
        const route = body.data?.route
        if (route) void navigate(route)
        resetCreate()
        scope.open.set(false)
      } catch (e) { createError.set(e instanceof Error ? e.message : String(e)) }
      finally { createSubmitting.set(false) }
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
      scope.open.set(false)
    }

    function resetCreate(): void {
      createName.set(''); createPurpose.set(''); createShape.set('list'); createSurface.set('view')
    }

    async function navigate(path: string): Promise<void> {
      const w = window as Record<string, unknown> & { ship?: { navigate?: (p: string) => void } }
      if (w.ship?.navigate) { w.ship.navigate(path); return }
      history.pushState({}, '', path)
      document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
    }

    // ── Session change listener ────────────────────────────────────
    function onSessionChanged(): void { void loadCurrentSession() }
    function onBarOpen(e: Event): void {
      const ce = e as CustomEvent<{ mode?: BarMode; prefill?: string }>
      if (ce.detail?.mode) mode.set(ce.detail.mode)
      if (typeof ce.detail?.prefill === 'string') value.set(ce.detail.prefill)
      scope.open.set(true)
    }
    function onCreateOpen(): void { mode.set('create'); scope.open.set(true) }

    window.addEventListener('ship:session:changed', onSessionChanged)
    const shell = document.querySelector('ship-shell') ?? document.querySelector('sui-shell')
    shell?.addEventListener('ship:bar:open', onBarOpen)
    shell?.addEventListener('ship:create:open', onCreateOpen)

    void loadCurrentSession()

    return () => {
      document.removeEventListener('keydown', onDocKeydown)
      window.removeEventListener('ship:session:changed', onSessionChanged)
      shell?.removeEventListener('ship:bar:open', onBarOpen)
      shell?.removeEventListener('ship:create:open', onCreateOpen)
      dialogEl?.removeEventListener('sui-close', onDialogClose)
      detachStream()
      stopOpenSync()
      stopModeSync()
      stopTrigger()
      stopRender()
    }
  },
})
