<scope>
  props: {
    open = false
    mode = 'ai'
  }
</scope>
<controller>
// FILE_CAP_OVERRIDE: cmdk keystone consolidates input row + palette body +
// chat body + keyboard/search/session stream in one tag.
import { controller, signal, derived, effect, mount } from '@ship/sail-runtime'
import type { Action, Signal } from '@ship/sail-runtime'
import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'
import { navigate } from '@ship/deck-router'
import type { Scope } from './ship-command-bar.sail'
import {
  type BarMode, type SessionSummary, type AgentSummary, type ThreadLine,
  type CreateSurface, type SlashCommand, type SearchHit, type SearchEntityKind,
  type CommandBarScope,
  CREATE_SURFACES, SEARCH_KIND_LABELS,
  loadSlashCommands, cachedSlashCommands, slashCacheAge, SLASH_CACHE_TTL_MS,
  searchEntities,
  detectMode, isKebab, publishCommandBar, BAR_CTX,
} from './types'

const ops = createShipConnectClient({ baseUrl: '' })
const RECENT_MAX = 20
const KIND_ORDER: SearchEntityKind[] = [
  'view', 'collection', 'agent', 'skill', 'session', 'collection_item', 'artifact',
]
const LAUNCH_TIER: ReadonlySet<CreateSurface> = new Set<CreateSurface>([
  'view', 'skill', 'agent', 'theme',
])

function asHtml(node: Element | null): HTMLElement | null {
  return node instanceof HTMLElement ? node : null
}

function rootOf(el: Element): Element {
  return el.shadowRoot ?? el
}

export default controller((ctx: Scope, host?: Element) => {
  const h = host as HTMLElement | undefined
  const open = ctx.open
  const mode = ctx.mode as unknown as Signal<BarMode>

  const value          = signal('')
  const session        = signal<SessionSummary | null>(null)
  const agents         = signal<AgentSummary[]>([])
  const harnesses      = signal<string[]>([])
  const allSessions    = signal<SessionSummary[]>([])
  const views          = signal<string[]>([])
  const selectedIdx    = signal(0)
  const sessionPickerOpen = signal(false)
  const spawnAgent     = signal('')
  const spawnHarness   = signal('')
  const spawning       = signal(false)
  const spawnError     = signal<string | null>(null)
  const thread         = signal<ThreadLine[]>([])
  const workspaceId    = signal('main')
  const createSurface  = signal<CreateSurface>('view')
  const createName     = signal('')
  const createPurpose  = signal('')
  const createShape    = signal('list')
  const createSubmitting = signal(false)
  const createError    = signal<string | null>(null)
  const slashCommands  = signal<SlashCommand[]>(cachedSlashCommands())
  const sendKey        = signal<'enter' | 'cmd+enter'>('enter')
  const searchHits     = signal<SearchHit[]>([])
  const searching      = signal(false)
  const daemonOnline   = signal(true)
  const recentSends    = signal<string[]>([])
  const historyIdx     = signal(-1)
  const sendError      = signal<string | null>(null)
  const dialogEl       = signal<Element | null>(null)
  const isChat = derived(() => mode() === 'ai')

  const captureDialog: Action<void> = (node) => {
    dialogEl.set(node)
    return { destroy: () => dialogEl.set(null) }
  }

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
  const modeGlyph = derived(() => {
    const glyphs: Record<string, string> = {
      command: '/', find: '>', mention: '@', create: '+', ai: 'ai',
    }
    return glyphs[mode()] ?? '>'
  })
  const searchPlaceholder = derived(() => {
    const placeholders: Record<string, string> = {
      command: 'Run a Ship action…',
      find:    'Jump to a view…',
      mention: 'Reference an agent, view, file…',
      create:  'Create a view, skill, agent…',
      ai:      'Ask Ship or type / for actions',
    }
    return placeholders[mode()] ?? 'Search…'
  })
  const chipLabel = derived(() => session()?.agent_id ?? 'No session')
  const chipDotBg = derived(() => {
    const s = session()
    if (s?.status === 'thinking') return 'var(--warning,#d97706)'
    if (s) return 'var(--success,#16a34a)'
    return 'var(--muted-foreground)'
  })
  const activeSessions = derived(() =>
    allSessions().filter((s) => s.status !== 'ended' && s.agent_id)
  )
  const searchGroups = derived(() => {
    const groups: Record<SearchEntityKind, SearchHit[]> = {
      view: [], collection: [], collection_item: [],
      agent: [], skill: [], session: [], artifact: [],
    }
    for (const hit of searchHits()) groups[hit.kind].push(hit)
    let nextIdx = 0
    return KIND_ORDER
      .filter((k) => groups[k].length > 0)
      .map((k) => ({
        kind: k,
        label: SEARCH_KIND_LABELS[k],
        items: groups[k].map((hit) => ({ ...hit, idx: nextIdx++ })),
      }))
  })
  const findHasResults = derived(() => searchHits().length > 0)
  const findSearching = derived(() => searching())
  const cmdItems = derived(() => {
    const needle = value().replace(/^\/+\s*/, '').toLowerCase().trim()
    return slashCommands()
      .filter((c) =>
        !needle ||
        c.key.includes(needle) ||
        c.label.toLowerCase().includes(needle) ||
        c.hint.toLowerCase().includes(needle)
      )
      .map((c, i) => ({ ...c, idx: i }))
  })
  const mentionItems = derived(() => {
    const needle = value().replace(/^@+/, '').toLowerCase().trim()
    const refs = [
      ...views().map((v) => ({ kind: 'view' as const, id: v, label: v })),
      ...agents().map((a) => ({ kind: 'agent' as const, id: a.id, label: a.id })),
      ...allSessions()
        .filter((s) => s.status !== 'ended' && s.agent_id)
        .map((s) => ({ kind: 'session' as const, id: s.id, label: `${s.agent_id} (session)` })),
    ]
    const iconFor = (k: 'view' | 'agent' | 'session'): string => {
      if (k === 'view') return 'app-window'
      if (k === 'agent') return 'bot'
      return 'message-square-plus'
    }
    return refs
      .filter((r) => !needle || r.id.toLowerCase().includes(needle))
      .slice(0, 40)
      .map((r, i) => ({ ...r, idx: i, icon: iconFor(r.kind) }))
  })
  const createSurfaces = derived(() =>
    CREATE_SURFACES.map((s) => ({ ...s, disabled: !LAUNCH_TIER.has(s.id) }))
  )
  const currentSurfaceDef = derived(() =>
    CREATE_SURFACES.find((s) => s.id === createSurface())!
  )
  const createNameValid = derived(() => isKebab(createName()))
  const hasSession = derived(() => session() !== null)
  const hasAgents = derived(() => agents().length > 0)
  const hasProviders = derived(() => harnesses().length > 0)
  const daemonOffline = derived(() => daemonOnline() === false)
  const activeDescendantId = derived(() => {
    const idx = selectedIdx()
    if (idx < 0) return ''
    return `cb-item-${idx}`
  })

  function emit(type: string, detail?: unknown): void {
    h?.dispatchEvent(new CustomEvent(type, {
      detail, bubbles: true, composed: true,
    }))
  }

  function openFind(): void {
    mode.set('find')
    value.set('')
    selectedIdx.set(0)
    open.set(true)
    emit('ship-open', { mode: 'find' })
  }
  function closeBar(): void {
    open.set(false)
  }

  function pickModeTab(m: string): void {
    if (m !== 'find' && m !== 'search' && m !== 'create' && m !== 'ai') return
    if (m === mode()) return
    mode.set(m as BarMode)
    emit('ship-mode-change', { mode: m })
  }

  function onInput(e: Event): void {
    const v = (e.target as HTMLInputElement).value
    value.set(v)
    mode.set(detectMode(v, mode()))
    selectedIdx.set(0)
  }

  function onInputKeydown(e: KeyboardEvent): void {
    if (e.key === 'ArrowDown') { e.preventDefault(); moveSelection(1) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); moveSelection(-1) }
    else if (e.key === 'Enter') { e.preventDefault(); confirmSelection() }
  }

  function onComposerKeydown(e: KeyboardEvent): void {
    const ta = e.target as HTMLTextAreaElement
    const recent = recentSends() as string[]
    const idx = historyIdx() as number
    const composerEmptyOrAtHistory =
      ta.value.trim() === '' || (idx >= 0 && ta.value === (recent[idx] ?? ''))

    if (e.key === 'ArrowUp' && composerEmptyOrAtHistory && recent.length > 0) {
      e.preventDefault()
      const next = Math.min(recent.length - 1, idx + 1)
      historyIdx.set(next)
      const v = recent[next] ?? ''
      value.set(v)
      queueMicrotask(() => { ta.setSelectionRange(v.length, v.length) })
      return
    }
    if (e.key === 'ArrowDown' && idx >= 0) {
      e.preventDefault()
      const next = idx - 1
      historyIdx.set(next)
      const v = next < 0 ? '' : (recent[next] ?? '')
      value.set(v)
      queueMicrotask(() => { ta.setSelectionRange(v.length, v.length) })
      return
    }

    const sk = sendKey?.() ?? 'enter'
    const isSubmit = sk === 'cmd+enter'
      ? (e.key === 'Enter' && (e.metaKey || e.ctrlKey))
      : (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey)
    if (!isSubmit) return
    e.preventDefault()
    void sendCurrent()
  }

  function toggleSessionPicker(): void {
    sessionPickerOpen.set(!sessionPickerOpen())
  }

  function selectSurface(id: CreateSurface): void {
    if (!LAUNCH_TIER.has(id)) return
    const def = CREATE_SURFACES.find((x) => x.id === id)
    createSurface.set(id)
    if (def?.shapes[0]) createShape.set(def.shapes[0])
  }

  async function submitCreate(): Promise<void> {
    if (!createNameValid() || createSubmitting()) return
    createError.set('Create is not available yet — backend op pending.')
  }

  async function endSession(id: string): Promise<void> {
    try { await fetch(`/api/v1/sessions/${id}`, { method: 'DELETE', credentials: 'same-origin' }) }
    catch { /* best-effort */ }
    if (session()?.id === id) { session.set(null) }
    await loadCurrentSession()
  }
  function selectSession(id: string): void {
    const s = allSessions().find((x) => x.id === id)
    if (s) { session.set(s); thread.set([]) }
    sessionPickerOpen.set(false)
  }
  function startNewSession(): void {
    sessionPickerOpen.set(false)
    mode.set('ai')
    session.set(null)
  }
  function viewChats(): void {
    sessionPickerOpen.set(false)
    navigate('/chats')
    open.set(false)
  }
  function goToAgents(): void {
    navigate('/studio/agents')
    open.set(false)
  }
  function dismissSendError(): void { sendError.set(null) }

  function getPaletteItems(): NodeListOf<Element> | undefined {
    return h ? rootOf(h).querySelectorAll('[role="option"]') : undefined
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
      if (path) { navigate(path); value.set(''); open.set(false) }
      const cmd = active.dataset['cmd']
      if (cmd) { value.set(''); runSlashCommand('/' + cmd) }
      const mKind = active.dataset['mentionKind'] as 'view' | 'agent' | 'session' | undefined
      const mId = active.dataset['mentionId']
      if (mKind && mId) {
        const ref = mKind === 'view' ? `@${mId}` : `@${mKind}:${mId}`
        value.set(v.replace(/@[^@\s]*$/, `${ref} `))
        mode.set(detectMode(value(), m))
      }
      emit('ship-select', { id: active.dataset['id'], kind: active.dataset['kind'], path: active.dataset['path'] })
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
    switch (cmd.key) {
      case 'new': mode.set('create'); break
      case 'chats': navigate('/chats'); open.set(false); break
      case 'settings': navigate('/settings'); open.set(false); break
      case 'theme': navigate('/theme-studio'); open.set(false); break
      case 'lint': navigate('/lint'); open.set(false); break
      case 'end': if (session()) void endSession(session()!.id); open.set(false); break
      case 'close': open.set(false); break
    }
  }
  function navigateFirstHit(): void {
    const top = searchHits()[0]
    if (top) { navigate(top.path); value.set(''); open.set(false) }
  }
  function activateNavItem(path: string): void {
    emit('ship-select', { id: path, kind: 'find', path })
    navigate(path)
    value.set('')
    open.set(false)
  }
  function activateCmd(key: string): void {
    value.set('')
    switch (key) {
      case 'new': mode.set('create'); break
      case 'chats': navigate('/chats'); open.set(false); break
      case 'settings': navigate('/settings'); open.set(false); break
      case 'theme': navigate('/theme-studio'); open.set(false); break
      case 'end':
        if (session()) void endSession(session()!.id)
        open.set(false)
        break
      case 'close': open.set(false); break
    }
  }
  function activateMention(item: { kind: string; id: string }): void {
    const ref = item.kind === 'view' ? `@${item.id}` : `@${item.kind}:${item.id}`
    const v = value()
    value.set(v.replace(/@[^@\s]*$/, `${ref} `))
    mode.set(detectMode(value(), mode()))
  }

  async function sendCurrent(): Promise<void> {
    const text = value().trim()
    if (!text || !session()) return
    const sessionId = session()!.id
    thread.set([
      ...thread(),
      { id: String(Date.now() + Math.random()), sender: 'user', text },
    ])
    value.set('')
    historyIdx.set(-1)
    sendError.set(null)
    try {
      const resp = await fetch(`/api/v1/sessions/${sessionId}/message`, {
        method: 'POST', credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ text }),
      })
      if (!resp.ok) {
        restoreOnFailure(text, `Send failed: HTTP ${resp.status}`)
        return
      }
      const recent = recentSends()
      const next = [text, ...recent.filter((t) => t !== text)].slice(0, RECENT_MAX)
      recentSends.set(next)
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Network error'
      restoreOnFailure(text, `Send failed: ${msg}`)
    }
  }
  function restoreOnFailure(text: string, message: string): void {
    const t = thread()
    const popped = t[t.length - 1]
    if (popped && popped.sender === 'user' && popped.text === text) {
      thread.set(t.slice(0, -1))
    }
    if (!value()) value.set(text)
    sendError.set(message)
  }
  function sendMessage(): void { void sendCurrent() }
  async function spawnSession(): Promise<void> {
    const agent = spawnAgent()
    const harness = spawnHarness()
    if (!agent || !harness) return
    spawning.set(true)
    spawnError.set(null)
    try {
      const wid = await ops.space.active({}).then((r) => r.active?.id).catch(() => undefined)
      if (!wid) {
        spawnError.set('No active project — cannot spawn session.')
        return
      }
      const resp = await fetch('/api/v1/sessions/spawn', {
        method: 'POST', credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workspace_id: wid, kind: 'agent', agent_id: agent, harness }),
      })
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({})) as { error?: string }
        spawnError.set(err.error ?? `HTTP ${resp.status}`)
      }
    } catch (err) {
      spawnError.set(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      spawning.set(false)
    }
  }

  async function refreshSlashCommands(force = false): Promise<void> {
    const list = await loadSlashCommands(force)
    slashCommands.set(list)
  }
  async function refreshSendKey(): Promise<void> {
    try {
      const setting = await ops.settings.get({ key: 'cmdk.send_key', scope: SettingScope.RUNTIME })
      if (setting.valueJson.length === 0) return
      const v = JSON.parse(new TextDecoder().decode(setting.valueJson)) as unknown
      if (v === 'enter' || v === 'cmd+enter') sendKey.set(v)
    } catch { /* leave default */ }
  }

  async function loadCurrentSession(): Promise<void> {
    try {
      const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
      if (resp.ok) {
        if (!daemonOnline()) daemonOnline.set(true)
        const body = await resp.json() as { sessions: SessionSummary[] }
        allSessions.set(body.sessions)
        const active = body.sessions.find((s) => s.status !== 'ended' && s.agent_id)
        if (active && active.id !== session()?.id) { session.set(active); thread.set([]) }
        else if (!active) { session.set(null) }
      } else if (daemonOnline()) {
        daemonOnline.set(false)
      }
    } catch { if (daemonOnline()) daemonOnline.set(false) }
    if (agents().length === 0) void loadAgents()
    if (harnesses().length === 0) void loadProviders()
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

  let streamAbort: AbortController | null = null
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
            const t = thread()
            thread.set(t.map((m) => m.id === currentAgentId ? { ...m, text: m.text + chunk } : m))
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

  function onCreateNameInput(e: Event): void {
    createName.set((e.currentTarget as HTMLInputElement).value)
  }
  function onCreateShapeInput(e: Event): void {
    createShape.set((e.currentTarget as HTMLSelectElement).value)
  }
  function onCreatePurposeInput(e: Event): void {
    createPurpose.set((e.currentTarget as HTMLInputElement).value)
  }
  function onSpawnAgentInput(e: Event): void {
    spawnAgent.set((e.currentTarget as HTMLSelectElement).value)
  }
  function onSpawnHarnessInput(e: Event): void {
    spawnHarness.set((e.currentTarget as HTMLSelectElement).value)
  }
  function onComposerInput(e: Event): void {
    value.set((e.currentTarget as HTMLTextAreaElement).value)
  }

  const barScope: CommandBarScope = {
    open, mode, value, session, agents, harnesses, allSessions, views,
    selectedIdx, sessionPickerOpen, spawnAgent, spawnHarness, spawning,
    spawnError, thread, workspaceId, createSurface, createName, createPurpose,
    createShape, createSubmitting, createError, isChat,
    dialogRef: dialogEl as unknown as Signal<HTMLElement | null>,
    slashCommands, sendKey, searchHits, searching,
    daemonOnline, recentSends, historyIdx, sendError,
  }
  publishCommandBar(host, barScope)

  mount((mounted) => {
    publishCommandBar(mounted, barScope)
    const dialogOf = (): (HTMLElement & { show(): void; close(): void }) | null =>
      asHtml(dialogEl()) as (HTMLElement & { show(): void; close(): void }) | null
    let lastSpaceAt = 0
    let searchTimer: ReturnType<typeof setTimeout> | null = null
    let searchAbort: AbortController | null = null
    function cancelSearch(): void {
      clearTimeout(searchTimer ?? undefined)
      searchTimer = null
      searchAbort?.abort()
      searchAbort = null
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

    const stopOpenSync = effect(() => {
      const isOpen = open()
      const dialog = dialogOf()
      mounted.setAttribute('data-state', isOpen ? 'open' : 'closed')
      if (isOpen) {
        if (typeof dialog?.show === 'function') dialog.show()
        if (slashCacheAge() > SLASH_CACHE_TTL_MS) void refreshSlashCommands(true)
        queueMicrotask(() => {
          if (mode() === 'ai') {
            const ta = rootOf(mounted).querySelector('textarea') as HTMLTextAreaElement | null
            if (ta) {
              ta.focus()
              if (ta.value.length > 0) ta.select()
            }
            return
          }
          const inp = rootOf(mounted).querySelector('input') as HTMLInputElement | null
          inp?.focus()
        })
      } else {
        if (typeof dialog?.close === 'function') dialog.close()
        mounted.dispatchEvent(new CustomEvent('sui-command:cancel', {
          detail: { mode: mode() }, bubbles: true, composed: true,
        }))
      }
    })

    const stopStreamSync = effect(() => {
      const s = session()
      if (s?.id) attachStream(s.id)
      else detachStream()
    })

    const stopModeFocusSync = effect(() => {
      const m = mode()
      if (open()) {
        queueMicrotask(() => {
          if (m === 'ai') {
            const ta = rootOf(mounted).querySelector('textarea') as HTMLTextAreaElement | null
            ta?.focus()
            return
          }
          const inp = rootOf(mounted).querySelector('input') as HTMLInputElement | null
          inp?.focus()
        })
      }
      void m
    })

    const stopSearchSync = effect(() => {
      const isOpen = open()
      const currentMode = mode()
      const q = value().replace(/^>+\s*/, '').trim()
      if (!isOpen || currentMode !== 'find') {
        cancelSearch()
        return
      }
      searchTimer = setTimeout(() => { void runSearch(q) }, 200)
    })

    type CB = {
      show(m?: string, q?: string): void
      hide(): void
      toggle(m?: string): void
      setMode(m: string): void
    }
    const cb = mounted as unknown as CB
    cb.show = (m, q) => {
      if (m) mode.set(m as BarMode)
      if (typeof q === 'string') value.set(q)
      open.set(true)
    }
    cb.hide = () => open.set(false)
    cb.toggle = (m) => {
      if (open()) { open.set(false) } else {
        if (m) mode.set(m as BarMode)
        open.set(true)
      }
    }
    cb.setMode = (m) => mode.set(m as BarMode)

    function onGlobalKey(e: KeyboardEvent): void {
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault()
        const next = !open()
        open.set(next)
        mounted.dispatchEvent(new CustomEvent(next ? 'ship-open' : 'ship-close', {
          bubbles: true, composed: true,
        }))
        return
      }
      if (e.key === 'Escape' && open()) { open.set(false); return }
      if (e.key === ' ' && !open()) {
        const t = e.target as HTMLElement | null
        if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
        const now = performance.now()
        if (now - lastSpaceAt < 350) {
          lastSpaceAt = 0
          e.preventDefault()
          mode.set('ai')
          value.set('')
          open.set(true)
        } else { lastSpaceAt = now }
      }
    }

    function onSessionChanged(): void { void loadCurrentSession() }
    function onBarOpen(e: Event): void {
      const ce = e as CustomEvent<{ mode?: BarMode; prefill?: string }>
      if (ce.detail?.mode) mode.set(ce.detail.mode)
      if (typeof ce.detail?.prefill === 'string') value.set(ce.detail.prefill)
      open.set(true)
    }
    function onCreateOpen(): void { mode.set('create'); open.set(true) }
    function onConfigChanged(e: Event): void {
      const detail = (e as CustomEvent<{ subject?: string }>).detail
      if (!detail || detail.subject === 'cmdk.send_key') void refreshSendKey()
    }
    function onDialogClose(): void { if (open()) open.set(false) }
    const stopDialogBind = effect(() => {
      const dialog = dialogOf()
      if (!dialog) return
      dialog.addEventListener('sail-close', onDialogClose)
      dialog.addEventListener('close', onDialogClose)
      return () => {
        dialog.removeEventListener('sail-close', onDialogClose)
        dialog.removeEventListener('close', onDialogClose)
      }
    })

    window.addEventListener('keydown', onGlobalKey)
    window.addEventListener('ship:session:changed', onSessionChanged)
    document.addEventListener('ship:bar:open', onBarOpen)
    document.addEventListener('ship:create:open', onCreateOpen)
    document.addEventListener('ship:config:changed', onConfigChanged)

    void loadCurrentSession()
    void refreshSlashCommands()
    void refreshSendKey()

    const bag = (mounted as unknown as Record<string, unknown>)[BAR_CTX] as Record<string, unknown> | undefined
    if (bag) {
      bag['endSession'] = endSession
      bag['loadCurrentSession'] = loadCurrentSession
    }

    return () => {
      window.removeEventListener('keydown', onGlobalKey)
      window.removeEventListener('ship:session:changed', onSessionChanged)
      document.removeEventListener('ship:bar:open', onBarOpen)
      document.removeEventListener('ship:create:open', onCreateOpen)
      document.removeEventListener('ship:config:changed', onConfigChanged)
      stopDialogBind()
      cancelSearch()
      detachStream()
      stopStreamSync()
      stopSearchSync()
      stopOpenSync()
      stopModeFocusSync()
    }
  })

  return {
    open, mode, value, session, agents, harnesses, allSessions, views,
    selectedIdx, sessionPickerOpen, spawnAgent, spawnHarness, spawning,
    spawnError, thread, workspaceId, createSurface, createName, createPurpose,
    createShape, createSubmitting, createError, isChat, captureDialog,
    slashCommands, sendKey, searchHits, searching,
    daemonOnline, recentSends, historyIdx, sendError,
    triggerLabel, dotBg, modeGlyph, searchPlaceholder,
    chipLabel, chipDotBg, activeSessions,
    searchGroups, findHasResults, findSearching,
    cmdItems, mentionItems, activeDescendantId,
    createSurfaces, currentSurfaceDef, createNameValid,
    hasSession, hasAgents, hasProviders, daemonOffline,
    openFind, pickModeTab, onInput, onInputKeydown, onComposerKeydown,
    toggleSessionPicker, selectSurface, submitCreate,
    selectSession, startNewSession, viewChats, goToAgents, dismissSendError,
    moveSelection, confirmSelection, runSlashCommand, navigateFirstHit,
    activateNavItem, activateCmd, activateMention,
    endSession, sendMessage, spawnSession,
    onCreateNameInput, onCreateShapeInput, onCreatePurposeInput,
    onSpawnAgentInput, onSpawnHarnessInput, onComposerInput,
    closeBar,
  }
})

export type { CommandBarScope }
</controller>

<!--
  ship-command-bar.sail — @ship/chrome · the cmdk keystone. ★ the one chrome locus.

  Consolidated from the shell's three-component command surface (palette +
  chat + bar). Surface (4-mode keystone):
    Modes      : find · search · create · ai (reflected `mode` attribute)
    Props      : open (bool, reflect), mode (string, reflect)
    Methods    : show(mode?, query?) · hide() · toggle(mode?) · setMode(mode)
    Events     : ship-open / ship-close (⌘K · esc) · ship-mode-change {mode} ·
                 ship-submit {query, mode} (↵) · ship-select {id, kind, path}
    Document   : ship:bar:open {mode?, prefill?} · ship:create:open
-->

<button
  type="button"
  aria-label="Open command bar"
  aria-haspopup="dialog"
  aria-controls="cb-dialog-root"
  aria-expanded={open() ? 'true' : 'false'}
  on:click={openFind}
>
  <icon name="search" size="sm"></icon>
  <span>{triggerLabel()}</span>
  <span style:background={dotBg()}></span>
  <kbd>⌘K</kbd>
</button>

<dialog
  id="cb-dialog-root"
  no-header
  no-footer
  no-close-button
  backdrop="blur"
  size="full"
  open={open()}
  on-close={closeBar}
  use:captureDialog
>
  <div>
    <div>
      <span data-ai={isChat()}>{modeGlyph()}</span>
      <input
        type="text"
        autocomplete="off"
        spellcheck="false"
        placeholder={searchPlaceholder()}
        value={value()}
        aria-label="Command bar search"
        aria-autocomplete="list"
        aria-activedescendant={activeDescendantId()}
        on:input={onInput}
        on:keydown={onInputKeydown}
      />
      <kbd aria-label="Escape to close">esc</kbd>
      <div>
        <span role="button" aria-haspopup="listbox" on:click={toggleSessionPicker}>
          <span style:background={chipDotBg()}></span>
          <span>{chipLabel()}</span>
        </span>
        <if {sessionPickerOpen()}>
          <div role="listbox" aria-label="Sessions">
            <for {activeSessions() as s} key={s.id}>
              <div
                data-current={session()?.id === s.id}
                role="option"
                on:click={() => selectSession(s.id)}
              >
                <span style:background={s.status === 'thinking' ? 'var(--warning)' : 'var(--success)'}></span>
                <span><strong>{s.agent_id ?? ''}</strong><span> · {s.harness ?? ''} · {s.workspace_id}</span></span>
                <button type="button" aria-label="End session" on:click={(e) => { e.stopPropagation(); endSession(s.id) }}>×</button>
              </div>
            </for>
            <if {activeSessions().length === 0}>
              <div><span></span><span>No active sessions</span><span></span></div>
            </if>
            <div role="separator"></div>
            <div on:click={startNewSession}>
              <icon name="plus" size="sm" aria-hidden="true"></icon>
              <span>Start new session</span>
            </div>
            <div on:click={viewChats}>
              <icon name="columns" size="sm" aria-hidden="true"></icon>
              <span>View all chats</span>
            </div>
          </div>
        </if>
      </div>
    </div>

    <if {isChat()}>
      <div>
        <if {daemonOffline()}>
          <div role="status">
            <icon name="alert-triangle" size="sm" aria-hidden="true"></icon>
            <span>Daemon unreachable. Reconnect to send messages.</span>
          </div>
        </if>
        <if {sendError()}>
          <div role="alert">
            <icon name="alert-circle" size="sm" aria-hidden="true"></icon>
            <span>{sendError()}</span>
            <button type="button" aria-label="Dismiss" on:click={dismissSendError}>×</button>
          </div>
        </if>
        <div>
          <if {hasSession()}>
            <if {thread().length === 0}>
              <div>
                <div>Connected to {session()?.agent_id ?? 'agent'}</div>
                <div>Type below and press Enter. ArrowUp recalls recent messages.</div>
              </div>
            {else =>
              <for {thread() as msg} key={msg.id}>
                <div data-sender={msg.sender}>
                  <div>{msg.text}</div>
                </div>
              </for>
            }</if>
          {else-if {!hasAgents()} =>
            <div>
              <div>No agents configured</div>
              <div>Open <button type="button" on:click={goToAgents}>/studio/agents</button> to add one, then start a session.</div>
            </div>
          }{else =>
            <div>
              <div>Start a session</div>
              <div>
                <label>Agent</label>
                <select value={spawnAgent()} on:input={onSpawnAgentInput}>
                  <for {agents() as a} key={a.id}>
                    <option value={a.id}>{a.name ?? a.label ?? a.id}</option>
                  </for>
                </select>
              </div>
              <div>
                <label>Harness</label>
                <if {hasProviders()}>
                  <select value={spawnHarness()} on:input={onSpawnHarnessInput}>
                    <for {harnesses() as p} key={p}>
                      <option value={p}>{p}</option>
                    </for>
                  </select>
                {else =>
                  <span>No harnesses installed</span>
                }</if>
              </div>
              <if {spawnError()}>
                <div>{spawnError()}</div>
              </if>
              <button
                type="button"
                disabled={spawning() || !spawnAgent() || !spawnHarness()}
                on:click={spawnSession}
              >{spawning() ? 'Starting…' : 'Start session'}</button>
            </div>
          }</if>
        </div>
        <div>
          <textarea
            rows="2"
            placeholder={hasSession() ? 'Message the agent…' : 'Draft a message — start a session above to send'}
            value={value()}
            on:input={onComposerInput}
            on:keydown={onComposerKeydown}
          ></textarea>
          <button
            type="button"
            disabled={!value().trim() || !hasSession()}
            on:click={sendMessage}
          >Send</button>
        </div>
      </div>
    {else =>
      <div>
        <if {mode() === 'find'}>
          <for {searchGroups() as group} key={group.kind}>
            <div>{group.label}</div>
            <div role="listbox" aria-label={group.label}>
              <for {group.items as item} key={item.kind + ':' + item.id}>
                <div
                  role="option"
                  id={'cb-item-' + item.idx}
                  aria-selected={selectedIdx() === item.idx ? 'true' : 'false'}
                  data-id={item.id}
                  data-kind={item.kind}
                  data-path={item.path}
                  on:click={() => activateNavItem(item.path)}
                >
                  <icon name={item.icon ?? 'app-window'} size="sm" aria-hidden="true"></icon>
                  <span>{item.label}</span>
                  <span>{group.label}</span>
                </div>
              </for>
            </div>
          </for>
          <if {!findHasResults() && findSearching()}>
            <div>Searching…</div>
          </if>
          <if {!findHasResults() && !findSearching()}>
            <div>No matches.</div>
          </if>
        {else-if {mode() === 'command'} =>
          <if {cmdItems().length > 0}>
            <div>Actions</div>
            <div role="listbox">
              <for {cmdItems() as item} key={item.key}>
                <div
                  role="option"
                  id={'cb-item-' + item.idx}
                  aria-selected={selectedIdx() === item.idx ? 'true' : 'false'}
                  data-id={item.key}
                  data-cmd={item.key}
                  on:click={() => activateCmd(item.key)}
                >
                  <span>{item.label}</span>
                  <span></span>
                  <span>{item.hint}</span>
                </div>
              </for>
            </div>
          {else =>
            <div>No commands match.</div>
          }</if>
        }{else-if {mode() === 'mention'} =>
          <if {mentionItems().length > 0}>
            <div>Reference</div>
            <div role="listbox">
              <for {mentionItems() as item} key={item.id + item.kind}>
                <div
                  role="option"
                  id={'cb-item-' + item.idx}
                  aria-selected={selectedIdx() === item.idx ? 'true' : 'false'}
                  data-id={item.kind + ':' + item.id}
                  data-mention-kind={item.kind}
                  data-mention-id={item.id}
                  on:click={() => activateMention(item)}
                >
                  <icon name={item.icon} size="sm" aria-hidden="true"></icon>
                  <span>{item.label}</span>
                  <span>{item.kind}</span>
                </div>
              </for>
            </div>
          {else =>
            <div>No matching views or agents.</div>
          }</if>
        }{else-if {mode() === 'create'} =>
          <div>Create</div>
          <div role="group" aria-label="Surface type">
            <for {createSurfaces() as surf} key={surf.id}>
              <button
                type="button"
                data-surface={surf.id}
                aria-pressed={createSurface() === surf.id ? 'true' : 'false'}
                aria-disabled={surf.disabled ? 'true' : 'false'}
                disabled={surf.disabled}
                title={surf.disabled ? 'post-launch' : surf.label}
                on:click={() => selectSurface(surf.id)}
              >
                <icon name={surf.icon} aria-hidden="true"></icon>
                <span>{surf.label}</span>
                <if {surf.disabled}>
                  <span>post-launch</span>
                </if>
              </button>
            </for>
          </div>
          <div>
            <label>
              <span>Name (kebab-case)</span>
              <input type="text" placeholder={'my-' + createSurface()} value={createName()} on:input={onCreateNameInput} autocomplete="off" />
            </label>
            <label>
              <span>{currentSurfaceDef().shapeLabel}</span>
              <select value={createShape()} on:input={onCreateShapeInput}>
                <for {currentSurfaceDef().shapes as sh} key={sh}>
                  <option value={sh}>{sh}</option>
                </for>
              </select>
            </label>
            <label>
              <span>Purpose (optional)</span>
              <input type="text" placeholder="One-line description" value={createPurpose()} on:input={onCreatePurposeInput} autocomplete="off" />
            </label>
          </div>
          <if {createError()}>
            <div>{createError()}</div>
          </if>
          <div>
            <button
              type="button"
              disabled={createSubmitting() || !createNameValid()}
              on:click={submitCreate}
            >{createSubmitting() ? 'Creating…' : 'Create ' + currentSurfaceDef().label}</button>
          </div>
        }{else =>
          <div>…</div>
        }</if>
      </div>
    }</if>
  </div>
</dialog>
