<scope>
  props: {
    open = false
    mode = 'ai'
  }
</scope>
<controller>
// FILE_CAP_OVERRIDE: <ship-command-palette> is the single-responsibility
// harness for 3 panel sub-components; keyboard + session-event subscription +
// search + focus management must co-locate here.
import { controller, signal, derived, effect, mount } from '@ship/sail-runtime'
import type { Action, Signal } from '@ship/sail-runtime'
import type { Scope } from './ship-command-palette.sail'
import {
  type BarMode, type SessionSummary, type AgentSummary, type ThreadLine, type CreateSurface,
  type SlashCommand, type SearchHit, type CommandBarScope,
  loadSlashCommands, cachedSlashCommands, slashCacheAge, SLASH_CACHE_TTL_MS,
  searchEntities, detectMode, publishCommandBar, PALETTE_CTX,
} from './types'
import { ops, ship, readSetting } from './client'
import { navigate } from '@ship/deck-router'

function asHtml(node: Element | null): HTMLElement | null {
  return node instanceof HTMLElement ? node : null
}

function rootOf(el: Element): Element {
  return el.shadowRoot ?? el
}

export default controller((ctx: Scope, host?: Element) => {
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
    if (sess?.status === 'thinking') return 'var(--warning)'
    if (sess) return 'var(--success)'
    return 'var(--muted-foreground)'
  })

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

  function openFind(): void {
    mode.set('find')
    value.set('')
    selectedIdx.set(0)
    open.set(true)
  }
  function closeBar(): void {
    open.set(false)
  }

  mount((mounted) => {
    publishCommandBar(mounted, barScope)

    let streamAbort: { abort(): void } | null = null
    let lastSpaceAt = 0
    const dialogOf = (): (HTMLElement & { show(): void; close(): void }) | null =>
      asHtml(dialogEl()) as (HTMLElement & { show(): void; close(): void }) | null

    async function refreshSlashCommands(force = false): Promise<void> {
      const list = await loadSlashCommands(force)
      slashCommands.set(list)
    }
    async function refreshSendKey(): Promise<void> {
      const v = await readSetting<unknown>('cmdk.send_key')
      if (v === 'enter' || v === 'cmd+enter') sendKey.set(v)
    }

    function focusActivePane(): void {
      queueMicrotask(() => {
        if (mode() === 'ai') {
          const chat = rootOf(mounted).querySelector('ship-command-chat')
          const ta = chat ? rootOf(chat).querySelector('textarea') as HTMLTextAreaElement | null : null
          if (ta) {
            ta.focus()
            if (ta.value.length > 0) ta.select()
          }
          return
        }
        const pal = rootOf(mounted).querySelector('ship-command-panel')
        const inp = pal ? rootOf(pal).querySelector('input') as HTMLInputElement | null : null
        inp?.focus()
      })
    }

    const stopOpenSync = effect(() => {
      const isOpen = open()
      const dialog = dialogOf()
      mounted.setAttribute('data-state', isOpen ? 'open' : 'closed')
      if (isOpen) {
        if (typeof dialog?.show === 'function') dialog.show()
        if (slashCacheAge() > SLASH_CACHE_TTL_MS) void refreshSlashCommands()
        focusActivePane()
      } else {
        if (typeof dialog?.close === 'function') dialog.close()
        mounted.dispatchEvent(new CustomEvent('sail-command:cancel', {
          detail: { mode: mode() }, bubbles: true, composed: true,
        }))
      }
    })

    type CB = { show(m?: string, q?: string): void; hide(): void; toggle(m?: string): void; setMode(m: string): void }
    ;(mounted as unknown as CB).show = (m?: string, q?: string) => {
      if (m) mode.set(m as BarMode)
      if (typeof q === 'string') value.set(q)
      open.set(true)
    }
    ;(mounted as unknown as CB).hide = () => open.set(false)
    ;(mounted as unknown as CB).toggle = (m?: string) => {
      if (open()) { open.set(false) } else {
        if (m) mode.set(m as BarMode)
        open.set(true)
      }
    }
    ;(mounted as unknown as CB).setMode = (m: string) => mode.set(m as BarMode)

    function attachStream(sessionId: string): void {
      detachStream()
      let currentAgentId: string | null = null
      const handleEvent = (event: { event_type: string; entity_id: string; payload_json?: string }): void => {
        let payload: Record<string, unknown> = {}
        try { payload = event.payload_json ? JSON.parse(event.payload_json) : {} } catch { /* malformed */ }
        const payloadSession = (payload as { session_id?: unknown }).session_id
        if (typeof payloadSession === 'string' && payloadSession !== sessionId) return
        if (event.entity_id && event.entity_id !== sessionId) return

        switch (event.event_type) {
          case 'agent.session.dispatched': {
            const s = session()
            if (s?.id === sessionId) session.set({ ...s, status: 'ready' })
            break
          }
          case 'agent.session.turn_ended':
          case 'agent.session.completed':
          case 'agent.session.failed':
          case 'agent.session.cancelled': {
            const s = session()
            if (s?.id === sessionId) session.set(null)
            void loadCurrentSession()
            break
          }
          case 'hook.tool_use_started': {
            const tool = (payload as { tool?: unknown }).tool
            const name = typeof tool === 'string' ? tool : '?'
            thread.set([...thread(), {
              id: String(Date.now() + Math.random()),
              sender: 'system',
              text: `→ tool: ${name}`,
            }])
            break
          }
          case 'hook.turn_observed': {
            currentAgentId = null
            break
          }
          default:
            break
        }
      }
      const off = ship.onNamespace('agent.session.', handleEvent)
      const offHook = ship.onNamespace('hook.', handleEvent)
      streamAbort = { abort: () => { off(); offHook() } }
    }
    function detachStream(): void { streamAbort?.abort(); streamAbort = null }

    const stopStreamSync = effect(() => {
      const s = session()
      if (s?.id) attachStream(s.id)
      else detachStream()
    })
    const stopModeFocusSync = effect(() => {
      const m = mode()
      if (open()) focusActivePane()
      void m
    })

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

    function markOnline(): void { if (!daemonOnline()) daemonOnline.set(true) }
    function markOffline(): void { if (daemonOnline()) daemonOnline.set(false) }

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
      views.set([])
    }

    function getPaletteItems(): NodeListOf<Element> | undefined {
      const pal = rootOf(mounted).querySelector('ship-command-panel')
      return pal ? rootOf(pal).querySelectorAll('[role="option"]') : undefined
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
        mounted.dispatchEvent(new CustomEvent('sail-command:select', {
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
    async function endSession(id: string): Promise<void> {
      const summary = allSessions().find((x: SessionSummary) => x.id === id)
      const slug = summary?.slug
      if (slug) {
        try { await ops.agents.stop({ slug }) } catch { /* best-effort */ }
      }
      if (session()?.id === id) session.set(null)
      await loadCurrentSession()
    }

    const onDocKeydown = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        if (open()) { open.set(false) } else { mode.set('find'); value.set(''); open.set(true) }
        return
      }
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

    const onPaletteKeydown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') { e.preventDefault(); moveSelection(1) }
      else if (e.key === 'ArrowUp') { e.preventDefault(); moveSelection(-1) }
      else if (e.key === 'Enter') { e.preventDefault(); confirmSelection() }
    }
    mounted.addEventListener('keydown', onPaletteKeydown)
    document.addEventListener('keydown', onDocKeydown)

    const onDialogClose = () => { if (open()) open.set(false) }
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

    const onSessionChanged = () => void loadCurrentSession()
    const onBarOpen = (e: Event) => {
      const ce = e as CustomEvent<{ mode?: BarMode; prefill?: string }>
      if (ce.detail?.mode) mode.set(ce.detail.mode)
      if (typeof ce.detail?.prefill === 'string') value.set(ce.detail.prefill)
      open.set(true)
    }
    const onCreateOpen = () => { mode.set('create'); open.set(true) }

    window.addEventListener('ship:session:changed', onSessionChanged)
    document.addEventListener('ship:bar:open', onBarOpen)
    document.addEventListener('ship:create:open', onCreateOpen)

    const bag = (mounted as unknown as Record<string, unknown>)[PALETTE_CTX] as Record<string, unknown> | undefined
    if (bag) {
      bag['endSession'] = endSession
      bag['loadCurrentSession'] = loadCurrentSession
    }

    void loadCurrentSession()
    void refreshSlashCommands()
    void refreshSendKey()

    const onConfigChanged = (e: Event) => {
      const detail = (e as CustomEvent<{ subject?: string }>).detail
      if (!detail || detail.subject === 'cmdk.send_key') void refreshSendKey()
    }
    document.addEventListener('ship:config:changed', onConfigChanged)

    return () => {
      document.removeEventListener('ship:config:changed', onConfigChanged)
      document.removeEventListener('keydown', onDocKeydown)
      mounted.removeEventListener('keydown', onPaletteKeydown)
      window.removeEventListener('ship:session:changed', onSessionChanged)
      document.removeEventListener('ship:bar:open', onBarOpen)
      document.removeEventListener('ship:create:open', onCreateOpen)
      stopDialogBind()
      cancelSearch()
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
    triggerLabel, dotBg, openFind, closeBar,
  }
})
</controller>

<!--
  <ship-command-palette> — @ship/chrome · harness shell for the command
  palette (the ⌘K experience). Renders the trigger pill + dialog and
  publishes host-walk state so <ship-command-panel> and <ship-command-chat>
  consume it.

  Props   : open (bool, reflect), mode (string, reflect)
  Methods : show(mode?, query?), hide(), toggle(mode?), setMode(mode)
  Events  : sui-command:select  { detail: { mode, item } }  bubbles+composed
            sui-command:cancel  { detail: { mode } }         bubbles+composed
  Modes   : ai, command, find, mention, create
  Keyboard: ⌘K toggle, space+space open-ai, ↑/↓ navigate, Enter activate, Esc close
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
  open={open()}
  on-close={closeBar}
  use:captureDialog
>
  <div>
    <if {isChat()}>
      <ship-command-chat></ship-command-chat>
    {else =>
      <ship-command-panel></ship-command-panel>
    }</if>
    <div aria-hidden="true">
      <span>Type <b>/</b> for actions<span> · </span><b>></b> to find<span> · </span><b>@</b> to mention<span> · </span><b>+</b> to create</span>
      <span><b>↵</b> confirm<span> · </span><b>esc</b> dismiss<span> · </span><b>⌘K</b> toggle</span>
    </div>
  </div>
</dialog>
