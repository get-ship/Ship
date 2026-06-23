// ship-command-bar.controller.ts — @ship/chrome · the cmdk keystone.
//
// Migrated from apps/shell/src/components/sui-command-bar/{sui-command-bar,
// sui-command-palette, sui-command-chat}.controller.ts. The shell version
// was a three-controller tree sharing state via `useShared`; the pack
// consolidates the three into one — input row, palette body, chat body
// all live in the same shadow root (`ship-command-bar.sui`).
//
// Audit refactors applied:
//   1. `chrome.navigate` and `window.ship?.navigate` are gone. Navigation
//      routes through `navigate()` from `@ship/deck/router` (unified router).
//   2. The shell's `navigateTo` helper is gone — the pack calls `navigate`
//      directly at each call site.
//   3. The synthetic `sui:navigate` re-fire in the devtools controller was
//      a recovery hack; the audit noted it for removal. See
//      `apps/shell/src/components/sui-devtools/sui-devtools-source.controller.ts`.
//
// Surface (4-mode keystone, preserved from the launch pack):
//   ship-open / ship-close (⌘K · esc) · ship-mode-change {mode} ·
//   ship-submit {query, mode} (↵) · ship-select {id, kind, path} (slot
//   pick with [data-id]).
//
// `CommandBarScope` is still published on the host (see types.ts) so any
// slotted custom child can `useShared<CommandBarScope>(host,
// 'ship-command-bar')`. The bar's own helpers consume the scope through
// the same `useShared` boundary to mirror the shell's split and keep
// reactivity observable.

import { controller, signal, derived, effect, ref, shared, type Signal, type ViewScope } from '@ship/sui-runtime'
import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'
import { navigate } from '@ship/deck/router'
import type { Scope } from './ship-command-bar.sui'
import {
  type BarMode, type SessionSummary, type AgentSummary, type ThreadLine,
  type CreateSurface, type SlashCommand, type SearchHit, type SearchEntityKind,
  type CommandBarScope,
  CREATE_SURFACES, SEARCH_KIND_LABELS,
  loadSlashCommands, cachedSlashCommands, slashCacheAge, SLASH_CACHE_TTL_MS,
  searchEntities,
  detectMode, isKebab,
} from './types'

// Host-agnostic same-origin Connect client. Cookies carry the session, so
// no token plumbing — mirrors the shell's `ops` handle. The pack owns its
// data plumbing; the host does not need to wire a client.
const ops = createShipConnectClient({ baseUrl: '' })

const RECENT_MAX = 20

const KIND_ORDER: SearchEntityKind[] = [
  'view', 'collection', 'agent', 'skill', 'session', 'collection_item', 'artifact',
]

const LAUNCH_TIER: ReadonlySet<CreateSurface> = new Set<CreateSurface>([
  'view', 'skill', 'agent', 'theme',
])

export default controller({
  scope: (init: ViewScope<Scope> & { open: Signal<boolean>; mode: Signal<BarMode> }, host?: Element) => {
    const h = host as HTMLElement | undefined
    // `open`/`mode` are `<prop>`s the runtime injects onto the scope before
    // this factory runs (component-factory: `$scope.open = props.open`).
    // The compiler omits them from the generated `Scope` because their
    // names collide with the `shared()` re-exports this factory returns,
    // so declare their injected signal types on the `init` parameter.
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
    const spawnHarness   = shared(signal(''))
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

    const isChat = shared(derived(() => mode() === 'ai'))

    const dialogRef = ref<HTMLElement>(null)

    // ── Trigger label + status dot ─────────────────────────────
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

    // ── Mode glyph + placeholders (palette input row) ──────────
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

    // ── Session chip + popover ────────────────────────────────
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

    // ── Find-mode search groups ────────────────────────────────
    const searchGroups = derived(() => {
      const groups: Record<SearchEntityKind, SearchHit[]> = {
        view: [], collection: [], collection_item: [],
        agent: [], skill: [], session: [], artifact: [],
      }
      for (const hit of searchHits()) {
        groups[hit.kind].push(hit)
      }
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

    // ── Command + mention items ────────────────────────────────
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

    // ── Create surfaces + form ─────────────────────────────────
    const createSurfaces = derived(() =>
      CREATE_SURFACES.map((s) => ({ ...s, disabled: !LAUNCH_TIER.has(s.id) }))
    )
    const currentSurfaceDef = derived(() =>
      CREATE_SURFACES.find((s) => s.id === createSurface())!
    )
    const createNameValid = derived(() => isKebab(createName()))

    // ── Chat (ai mode) ─────────────────────────────────────────
    const hasSession = derived(() => session() !== null)
    const hasAgents = derived(() => agents().length > 0)
    const hasProviders = derived(() => harnesses().length > 0)
    const daemonOffline = derived(() => daemonOnline() === false)

    // ARIA combobox: input.aria-activedescendant points at the currently
    // selected listbox item's id. The bar publishes the resolved id for
    // whichever mode the input row is in.
    const activeDescendantId = derived(() => {
      const idx = selectedIdx()
      if (idx < 0) return ''
      return `cb-item-${idx}`
    })

    // ── Imperative helpers exposed to template ─────────────────
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
      // History navigation: ArrowUp / ArrowDown in an empty composer
      // walks the recent-sends ring. Position −1 means "draft mode";
      // 0 is most-recent send. Down past 0 restores the empty draft.
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

      // `cmdk.send_key` controls Enter semantics in ai mode. Default
      // "enter" sends on plain Enter, newline on Shift+Enter. "cmd+enter"
      // inverts: Enter inserts a newline (native), Cmd/Ctrl+Enter sends.
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
      // Mirror createSurfaces' launch-tier gate — clicking a disabled
      // tile is a no-op so users can't reach the form for a surface
      // whose backend scaffold isn't wired yet.
      if (!LAUNCH_TIER.has(id)) return
      const def = CREATE_SURFACES.find((x) => x.id === id)
      createSurface.set(id)
      if (def?.shapes[0]) createShape.set(def.shapes[0])
    }

    async function submitCreate(): Promise<void> {
      if (!createNameValid() || createSubmitting()) return
      // The create-surface op was quarantined (06153c1a5) — scaffolding a
      // view/skill/agent/theme has no daemon op yet. Surface that honestly
      // instead of calling a dead RPC; the backend op is raised for build.
      createError.set('Create is not available yet — backend op pending.')
    }

    // ── Imperative chat helpers ────────────────────────────────
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

    // ── Selection navigation (palette listbox) ─────────────────
    function getPaletteItems(): NodeListOf<Element> | undefined {
      return h?.shadowRoot?.querySelectorAll('.cb-item')
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
      // Dispatch before closing: the dialog gates its <slot> on
      // {#if open()}, so flipping open=false synchronously detaches
      // the projection path and a later composed event from the
      // palette host can't reach the bar host.
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

    // ── Chat send (ai mode) ────────────────────────────────────
    async function sendCurrent(): Promise<void> {
      const text = value().trim()
      if (!text || !session()) return
      const sessionId = session()!.id

      // Optimistic render: append user bubble, clear composer + history idx.
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

    // ── Slash registry + send-key setting loaders ─────────────
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

    // ── Async data loading ─────────────────────────────────────
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

    // ── SSE session stream (ai mode) ───────────────────────────
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

    return {
      open, mode, value, session, agents, harnesses, allSessions, views,
      selectedIdx, sessionPickerOpen, spawnAgent, spawnHarness, spawning,
      spawnError, thread, workspaceId, createSurface, createName, createPurpose,
      createShape, createSubmitting, createError, isChat, dialogRef,
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
      refreshSlashCommands, refreshSendKey, loadCurrentSession,
      attachStream, detachStream,
    }
  },

  mount(state, host) {
    const open         = state.$signals.open
    const mode         = state.$signals.mode
    const value        = state.$signals.value
    const session      = state.$signals.session
    const searchHits   = state.$signals.searchHits
    const searching    = state.$signals.searching
    const dialogRef    = state.$signals.dialogRef?.() as (HTMLElement & { show(): void; close(): void }) | null

    let lastSpaceAt = 0
    let searchTimer: ReturnType<typeof setTimeout> | null = null
    let searchAbort: AbortController | null = null
    let streamAbort: AbortController | null = null
    function cancelSearch(): void {
      clearTimeout(searchTimer)
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

    // ── Open/close effect ──────────────────────────────────────
    const stopOpenSync = effect(() => {
      const isOpen = open()
      host.setAttribute('data-state', isOpen ? 'open' : 'closed')
      if (isOpen) {
        if (typeof dialogRef?.show === 'function') dialogRef.show()
        if (slashCacheAge() > SLASH_CACHE_TTL_MS) void state.refreshSlashCommands(true)
        // Focus the right pane on next microtask so {#if isChat()} has
        // rendered and the relevant child is in the DOM.
        queueMicrotask(() => {
          if (mode() === 'ai') {
            const ta = host.shadowRoot?.querySelector('.cb-chat-input') as HTMLTextAreaElement | null
            if (ta) {
              ta.focus()
              if (ta.value.length > 0) ta.select()
            }
            return
          }
          const inp = host.shadowRoot?.querySelector('.cb-search') as HTMLInputElement | null
          inp?.focus()
        })
      } else {
        if (typeof dialogRef?.close === 'function') dialogRef.close()
        host.dispatchEvent(new CustomEvent('sui-command:cancel', {
          detail: { mode: mode() }, bubbles: true, composed: true,
        }))
      }
    })

    // Auto-attach stream when session changes.
    const stopStreamSync = effect(() => {
      const s = session()
      if (s?.id) state.attachStream(s.id)
      else state.detachStream()
    })

    // Re-focus the right input whenever mode flips while the bar is open
    // (e.g., send-to-agent prefill flips mode→'ai' on an already-open bar).
    const stopModeFocusSync = effect(() => {
      const m = mode()
      if (open()) {
        queueMicrotask(() => {
          if (m === 'ai') {
            const ta = host.shadowRoot?.querySelector('.cb-chat-input') as HTMLTextAreaElement | null
            ta?.focus()
            return
          }
          const inp = host.shadowRoot?.querySelector('.cb-search') as HTMLInputElement | null
          inp?.focus()
        })
      }
      void m
    })

    // ── Find-mode unified search ───────────────────────────────
    // Debounce 200ms so each keystroke doesn't fire a request. Cancel
    // any in-flight search on a new keystroke. Cleared when the bar
    // closes or the mode flips away from 'find'.
    const stopSearchSync = effect(() => {
      const isOpen = open()
      const currentMode = mode()
      const q = value().replace(/^>+\s*/, '').trim()
      if (!isOpen || currentMode !== 'find') {
        cancelSearch()
        return
      }
      searchTimer = setTimeout(() => { void runSearch(q) }, 200)

    // ── Public API ─────────────────────────────────────────────
    type CB = {
      show(m?: string, q?: string): void
      hide(): void
      toggle(m?: string): void
      setMode(m: string): void
    }
    const cb = host as unknown as CB
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

    // ── Global ⌘K listener ─────────────────────────────────────
    function onGlobalKey(e: KeyboardEvent): void {
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault()
        const next = !open()
        open.set(next)
        host.dispatchEvent(new CustomEvent(next ? 'ship-open' : 'ship-close', {
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

    // ── Document events (bar-open / create-open / session-changed) ──
    function onSessionChanged(): void { void state.loadCurrentSession() }
    function onBarOpen(e: Event): void {
      const ce = e as CustomEvent<{ mode?: BarMode; prefill?: string }>
      if (ce.detail?.mode) mode.set(ce.detail.mode)
      if (typeof ce.detail?.prefill === 'string') value.set(ce.detail.prefill)
      open.set(true)
    }
    function onCreateOpen(): void { mode.set('create'); open.set(true) }

    function onConfigChanged(e: Event): void {
      const detail = (e as CustomEvent<{ subject?: string }>).detail
      if (!detail || detail.subject === 'cmdk.send_key') void state.refreshSendKey()
    }

    // ── Dialog close listener ──────────────────────────────────
    function onDialogClose(): void { if (open()) open.set(false) }
    dialogRef?.addEventListener('sui-close', onDialogClose)

    window.addEventListener('keydown', onGlobalKey)
    window.addEventListener('ship:session:changed', onSessionChanged)
    document.addEventListener('ship:bar:open', onBarOpen)
    document.addEventListener('ship:create:open', onCreateOpen)
    document.addEventListener('ship:config:changed', onConfigChanged)

    // Boot.
    void state.loadCurrentSession()
    void state.refreshSlashCommands()
    void state.refreshSendKey()

    // Provide the legacy shared-scope API on the host so a slotted
    // custom child (e.g. a future <ship-command-palette>) can
    // `useShared<CommandBarScope>(host, 'ship-command-bar')`.
    const ctxKey = '__sui_ctx_ship-command-bar'
    const scope: CommandBarScope = {
      open, mode, value, session, agents, harnesses, allSessions, views,
      selectedIdx, sessionPickerOpen, spawnAgent, spawnHarness, spawning,
      spawnError, thread, workspaceId, createSurface, createName, createPurpose,
      createShape, createSubmitting, createError, isChat, dialogRef,
      slashCommands, sendKey, searchHits, searching,
      daemonOnline, recentSends, historyIdx, sendError,
    }
    ;(host as unknown as Record<string, unknown>)[ctxKey] = {
      ...scope,
      endSession: state.endSession,
      loadCurrentSession: state.loadCurrentSession,
    }

    return () => {
      window.removeEventListener('keydown', onGlobalKey)
      window.removeEventListener('ship:session:changed', onSessionChanged)
      document.removeEventListener('ship:bar:open', onBarOpen)
      document.removeEventListener('ship:create:open', onCreateOpen)
      document.removeEventListener('ship:config:changed', onConfigChanged)
      if (dialogRef) dialogRef.removeEventListener('sui-close', onDialogClose)
      cancelSearch()
      streamAbort?.abort()
      stopStreamSync()
      stopSearchSync()
      stopOpenSync()
      stopModeFocusSync()
    }
  },
})

// keep the helper export alive (referenced above)
export type { CommandBarScope }
