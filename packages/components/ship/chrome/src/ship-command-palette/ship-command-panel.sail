<scope>
  props: {}
</scope>
<controller>
// FILE_CAP_OVERRIDE: panel is a multi-mode switch (find/command/mention/create)
// sharing one search row and the palette host-walk scope.
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './ship-command-panel.sail'
import {
  type CreateSurface, type SessionSummary, type SearchHit, type SearchEntityKind,
  CREATE_SURFACES, SEARCH_KIND_LABELS,
  detectMode, isKebab, commandBarOf,
} from './types'
import { ops } from './client'
import { navigate } from '@ship/deck-router'

export default controller((_ctx: Scope, host?: Element) => {
  const bar = commandBarOf(host)

  const modeGlyph = derived(() => {
    const glyphs: Record<string, string> = {
      command: '/', find: '>', mention: '@', create: '+', ai: 'ai',
    }
    return glyphs[bar.mode()] ?? '>'
  })

  const searchPlaceholder = derived(() => {
    const placeholders: Record<string, string> = {
      command: 'Run a Ship action…',
      find:    'Jump to a view…',
      mention: 'Reference an agent, view, file…',
      create:  'Create a view, skill, agent…',
      ai:      'Ask Ship or type / for actions',
    }
    return placeholders[bar.mode()] ?? 'Search…'
  })

  const chipLabel = derived(() => bar.session()?.agent_id ?? 'No session')

  const chipDotBg = derived(() => {
    const s = bar.session()
    if (s?.status === 'thinking') return 'var(--warning)'
    if (s) return 'var(--success)'
    return 'var(--muted-foreground)'
  })

  const activeSessions = derived(() =>
    bar.allSessions().filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
  )

  function mentionIcon(kind: string): string {
    if (kind === 'view') return 'app-window'
    if (kind === 'agent') return 'bot'
    return 'message-square-plus'
  }

  const KIND_ORDER: SearchEntityKind[] = [
    'view', 'collection', 'agent', 'skill', 'session', 'collection_item', 'artifact',
  ]
  const searchGroups = derived(() => {
    const groups: Record<SearchEntityKind, SearchHit[]> = {
      view: [], collection: [], collection_item: [],
      agent: [], skill: [], session: [], artifact: [],
    }
    for (const hit of bar.searchHits()) {
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
  const findHasResults = derived(() => bar.searchHits().length > 0)
  const findSearching = derived(() => bar.searching())

  const cmdItems = derived(() => {
    const needle = bar.value().replace(/^\/+\s*/, '').toLowerCase().trim()
    return bar.slashCommands()
      .filter((c) =>
        !needle ||
        c.key.includes(needle) ||
        c.label.toLowerCase().includes(needle) ||
        c.hint.toLowerCase().includes(needle)
      )
      .map((c, i) => ({ ...c, idx: i }))
  })

  const mentionItems = derived(() => {
    const needle = bar.value().replace(/^@+/, '').toLowerCase().trim()
    type Ref = { kind: 'view' | 'agent' | 'session'; id: string; label: string; idx: number; icon: string }
    const refs: Omit<Ref, 'idx' | 'icon'>[] = [
      ...bar.views().map((v: string) => ({ kind: 'view' as const, id: v, label: v })),
      ...bar.agents().map((a: { id: string }) => ({ kind: 'agent' as const, id: a.id, label: a.id })),
      ...bar.allSessions()
        .filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
        .map((s: SessionSummary) => ({ kind: 'session' as const, id: s.id, label: `${s.agent_id} (session)` })),
    ]
    return refs
      .filter((r) => !needle || r.id.toLowerCase().includes(needle))
      .slice(0, 40)
      .map((r, i) => ({ ...r, idx: i, icon: mentionIcon(r.kind) }))
  })

  const LAUNCH_TIER: ReadonlySet<CreateSurface> = new Set<CreateSurface>([
    'view', 'skill', 'agent', 'theme',
  ])
  const createSurfaces = derived(() =>
    CREATE_SURFACES.map((s) => ({ ...s, disabled: !LAUNCH_TIER.has(s.id) }))
  )

  const activeDescendantId = derived(() => {
    const idx = bar.selectedIdx()
    if (idx < 0) return ''
    return `cb-item-${idx}`
  })

  const currentSurfaceDef = derived(() =>
    CREATE_SURFACES.find((s) => s.id === bar.createSurface())!
  )

  const createNameValid = derived(() => isKebab(bar.createName()))

  function endSession(id: string): void {
    const summary = bar.allSessions().find((x: SessionSummary) => x.id === id)
    const slug = summary?.slug
    if (!slug) {
      bar.sessionPickerOpen.set(false)
      document.dispatchEvent(new CustomEvent('sui:notify', {
        detail: {
          tone: 'warning',
          title: 'End session',
          description: 'Session is not locally dispatchable; cleared from view.',
        },
      }))
      if (bar.session()?.id === id) bar.session.set(null)
      return
    }
    void ops.agents.stop({ slug })
      .catch(() => {/* best-effort */})
      .then(() => {
        if (bar.session()?.id === id) bar.session.set(null)
        void reloadSessions()
      })
    bar.sessionPickerOpen.set(false)
  }

  function selectSession(id: string): void {
    const s = bar.allSessions().find((x: SessionSummary) => x.id === id)
    if (s) { bar.session.set(s); bar.thread.set([]) }
    bar.sessionPickerOpen.set(false)
  }

  function startNewSession(): void {
    bar.sessionPickerOpen.set(false)
    bar.mode.set('ai')
    bar.session.set(null)
  }

  function viewChats(): void {
    bar.sessionPickerOpen.set(false)
    navigate('/chats')
    bar.open.set(false)
  }

  const hostEl = host as HTMLElement | null
  function activateNavItem(path: string): void {
    hostEl?.dispatchEvent(new CustomEvent('sail-command:select', {
      detail: { mode: 'find', item: { path } }, bubbles: true, composed: true,
    }))
    navigate(path)
    bar.value.set('')
    bar.open.set(false)
  }

  function activateCmd(key: string): void {
    bar.value.set('')
    switch (key) {
      case 'new': bar.mode.set('create'); break
      case 'chats': navigate('/chats'); bar.open.set(false); break
      case 'settings': navigate('/settings'); bar.open.set(false); break
      case 'theme': navigate('/theme-studio'); bar.open.set(false); break
      case 'end':
        if (bar.session()) endSession(bar.session()!.id)
        bar.open.set(false)
        break
      case 'close': bar.open.set(false); break
    }
  }

  function activateMention(item: { kind: string; id: string }): void {
    const ref = item.kind === 'view' ? `@${item.id}` : `@${item.kind}:${item.id}`
    const v = bar.value()
    bar.value.set(v.replace(/@[^@\s]*$/, `${ref} `))
    bar.mode.set(detectMode(bar.value(), bar.mode()))
  }

  async function reloadSessions(): Promise<void> {
    try {
      const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
      if (resp.ok) {
        const body = await resp.json() as { sessions: SessionSummary[] }
        bar.allSessions.set(body.sessions)
        const active = body.sessions.find((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
        if (active && active.id !== bar.session()?.id) { bar.session.set(active); bar.thread.set([]) }
        else if (!active) { bar.session.set(null) }
      }
    } catch { /* leave as-is */ }
  }

  function onInput(e: Event): void {
    const v = (e.target as HTMLInputElement).value
    bar.value.set(v)
    bar.mode.set(detectMode(v, bar.mode()))
    bar.selectedIdx.set(0)
  }

  function toggleSessionPicker(): void {
    bar.sessionPickerOpen.set(!bar.sessionPickerOpen())
  }

  function selectSurface(id: CreateSurface): void {
    const def = CREATE_SURFACES.find((x) => x.id === id)
    if (!LAUNCH_TIER.has(id)) return
    bar.createSurface.set(id)
    if (def?.shapes[0]) bar.createShape.set(def.shapes[0])
  }

  async function submitCreate(): Promise<void> {
    if (!createNameValid() || bar.createSubmitting()) return
    bar.createError.set('Create is not available yet — backend op pending.')
  }

  function onCreateNameInput(e: Event): void {
    bar.createName.set((e.currentTarget as HTMLInputElement).value)
  }
  function onCreateShapeInput(e: Event): void {
    bar.createShape.set((e.currentTarget as HTMLSelectElement).value)
  }
  function onCreatePurposeInput(e: Event): void {
    bar.createPurpose.set((e.currentTarget as HTMLInputElement).value)
  }

  return {
    bar,
    modeGlyph, searchPlaceholder,
    chipLabel, chipDotBg,
    activeSessions,
    searchGroups, findHasResults, findSearching,
    cmdItems, mentionItems,
    activeDescendantId,
    createSurfaces, currentSurfaceDef, createNameValid,
    endSession, selectSession, startNewSession, viewChats,
    activateNavItem, activateCmd, activateMention,
    onInput, toggleSessionPicker, selectSurface, submitCreate,
    onCreateNameInput, onCreateShapeInput, onCreatePurposeInput,
  }
})
</controller>

<!--
  <ship-command-panel> — @ship/chrome · the inner command palette panel
  (find / command / mention / create modes). Consumed inside
  <ship-command-palette>. Accesses harness state via the palette host bag.
-->

<div>
  <span data-ai={bar.mode() === 'ai'}>{modeGlyph()}</span>
  <input
    type="text"
    autocomplete="off"
    spellcheck="false"
    placeholder={searchPlaceholder()}
    value={bar.value()}
    aria-label="Command bar search"
    aria-autocomplete="list"
    aria-activedescendant={activeDescendantId()}
    on:input={onInput}
  />
  <kbd aria-label="Escape to close">esc</kbd>
  <div>
    <span role="button" aria-haspopup="listbox" on:click={toggleSessionPicker}>
      <span style:background={chipDotBg()}></span>
      <span>{chipLabel()}</span>
    </span>
    <if {bar.sessionPickerOpen()}>
      <div role="listbox" aria-label="Sessions">
        <for {activeSessions() as s} key={s.id}>
          <div
            data-current={bar.session()?.id === s.id}
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

<div>
  <if {bar.mode() === 'find'}>
    <for {searchGroups() as group} key={group.kind}>
      <div>{group.label}</div>
      <div role="listbox" aria-label={group.label}>
        <for {group.items as item} key={item.kind + ':' + item.id}>
          <div
            role="option"
            id={'cb-item-' + item.idx}
            aria-selected={bar.selectedIdx() === item.idx ? 'true' : 'false'}
            data-path={item.path}
            data-kind={item.kind}
            data-id={item.id}
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
  {else-if {bar.mode() === 'command'} =>
    <if {cmdItems().length > 0}>
      <div>Actions</div>
      <div role="listbox">
        <for {cmdItems() as item} key={item.key}>
          <div
            role="option"
            id={'cb-item-' + item.idx}
            aria-selected={bar.selectedIdx() === item.idx ? 'true' : 'false'}
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
  }{else-if {bar.mode() === 'mention'} =>
    <if {mentionItems().length > 0}>
      <div>Reference</div>
      <div role="listbox">
        <for {mentionItems() as item} key={item.id + item.kind}>
          <div
            role="option"
            id={'cb-item-' + item.idx}
            aria-selected={bar.selectedIdx() === item.idx ? 'true' : 'false'}
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
  }{else-if {bar.mode() === 'create'} =>
    <div>Create</div>
    <div role="group" aria-label="Surface type">
      <for {createSurfaces() as surf} key={surf.id}>
        <button
          type="button"
          data-surface={surf.id}
          aria-pressed={bar.createSurface() === surf.id ? 'true' : 'false'}
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
        <input type="text" placeholder={'my-' + bar.createSurface()} value={bar.createName()} on:input={onCreateNameInput} autocomplete="off" />
      </label>
      <label>
        <span>{currentSurfaceDef().shapeLabel}</span>
        <select value={bar.createShape()} on:input={onCreateShapeInput}>
          <for {currentSurfaceDef().shapes as sh} key={sh}>
            <option value={sh}>{sh}</option>
          </for>
        </select>
      </label>
      <label>
        <span>Purpose (optional)</span>
        <input type="text" placeholder="One-line description" value={bar.createPurpose()} on:input={onCreatePurposeInput} autocomplete="off" />
      </label>
    </div>
    <if {bar.createError()}>
      <div>{bar.createError()}</div>
    </if>
    <div>
      <button
        type="button"
        disabled={bar.createSubmitting() || !createNameValid()}
        on:click={submitCreate}
      >{bar.createSubmitting() ? 'Creating…' : 'Create ' + currentSurfaceDef().label}</button>
    </div>
  }{else =>
    <div>…</div>
  }</if>
</div>
