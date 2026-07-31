<controller>
import { controller, derived, useShared } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-command-panel.sail'
import {
  type CreateSurface, type SessionSummary, type SearchHit, type SearchEntityKind,
  type CommandBarScope,
  CREATE_SURFACES, SEARCH_KIND_LABELS,
  detectMode, isKebab,
} from './types'
import { ops } from './client'
import { navigate } from '@ship/deck-router'

export default controller((_ctx: ViewScope<Scope>, host?: Element) => {

    const bar = useShared<CommandBarScope>(host!, 'ship-command-palette')

    const modeGlyph = derived(() => {
      const glyphs: Record<string, string> = {
        command: '/', find: '>', mention: '@', create: '+', ai: 'ai',
      }
      return glyphs[bar!.mode()] ?? '>'
    })

    const searchPlaceholder = derived(() => {
      const placeholders: Record<string, string> = {
        command: 'Run a Ship action…',
        find:    'Jump to a view…',
        mention: 'Reference an agent, view, file…',
        create:  'Create a view, skill, agent…',
        ai:      'Ask Ship or type / for actions',
      }
      return placeholders[bar!.mode()] ?? 'Search…'
    })

    const chipLabel = derived(() => bar!.session()?.agent_id ?? 'No session')

    const chipDotBg = derived(() => {
      const s = bar!.session()
      if (s?.status === 'thinking') return 'var(--warning)'
      if (s) return 'var(--success)'
      return 'var(--muted-foreground)'
    })

    const activeSessions = derived(() =>
      bar!.allSessions().filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
    )

    function mentionIcon(kind: string): string {
      if (kind === 'view') return 'app-window'
      if (kind === 'agent') return 'bot'
      return 'message-square-plus'
    }

    // Sections of /api/v1/search hits grouped by kind. The bar controller
    // populates bar.searchHits on a 200ms debounce; we render the groups
    // in their preserved server order with monotonically-assigned indices
    // so selectedIdx (shared with cmd/mention modes) maps to a unique
    // listbox row.
    const KIND_ORDER: SearchEntityKind[] = [
      'view', 'collection', 'agent', 'skill', 'session', 'collection_item', 'artifact',
    ]
    const searchGroups = derived(() => {
      const groups: Record<SearchEntityKind, SearchHit[]> = {
        view: [], collection: [], collection_item: [],
        agent: [], skill: [], session: [], artifact: [],
      }
      for (const hit of bar!.searchHits()) {
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
    const findHasResults = derived(() => bar!.searchHits().length > 0)
    const findSearching = derived(() => bar!.searching())

    const cmdItems = derived(() => {
      const needle = bar!.value().replace(/^\/+\s*/, '').toLowerCase().trim()
      return bar!.slashCommands()
        .filter((c) =>
          !needle ||
          c.key.includes(needle) ||
          c.label.toLowerCase().includes(needle) ||
          c.hint.toLowerCase().includes(needle)
        )
        .map((c, i) => ({ ...c, idx: i }))
    })

    const mentionItems = derived(() => {
      const needle = bar!.value().replace(/^@+/, '').toLowerCase().trim()
      type Ref = { kind: 'view' | 'agent' | 'session'; id: string; label: string; idx: number; icon: string }
      const refs: Omit<Ref, 'idx' | 'icon'>[] = [
        ...bar!.views().map((v: string) => ({ kind: 'view' as const, id: v, label: v })),
        ...bar!.agents().map((a: { id: string }) => ({ kind: 'agent' as const, id: a.id, label: a.id })),
        ...bar!.allSessions()
          .filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
          .map((s: SessionSummary) => ({ kind: 'session' as const, id: s.id, label: `${s.agent_id} (session)` })),
      ]
      return refs
        .filter((r) => !needle || r.id.toLowerCase().includes(needle))
        .slice(0, 40)
        .map((r, i) => ({ ...r, idx: i, icon: mentionIcon(r.kind) }))
    })

    // Launch-tier surfaces have backend scaffold endpoints today; the
    // others are visible but disabled until their scaffold lands so
    // users know the surface exists without hitting a 501.
    const LAUNCH_TIER: ReadonlySet<CreateSurface> = new Set<CreateSurface>([
      'view', 'skill', 'agent', 'theme',
    ])
    const createSurfaces = derived(() =>
      CREATE_SURFACES.map((s) => ({ ...s, disabled: !LAUNCH_TIER.has(s.id) }))
    )

    // ARIA combobox: input.aria-activedescendant points at the currently
    // selected listbox item's id.
    const activeDescendantId = derived(() => {
      const idx = bar!.selectedIdx()
      if (idx < 0) return ''
      return `cb-item-${idx}`
    })

    const currentSurfaceDef = derived(() =>
      CREATE_SURFACES.find((s) => s.id === bar!.createSurface())!
    )

    const createNameValid = derived(() => isKebab(bar!.createName()))

    // ── Imperative helpers exposed to template ────────────────────
    function endSession(id: string): void {
      // Ported off the dead per-session DELETE onto agents.stop (Connect).
      // agents.stop takes the dispatch *slug*
      // — the pty session id the old route keyed on is a different
      // value. Sessions we dispatched carry the slug on their
      // SessionSummary; sessions loaded from agents.sessions don't
      // (yet — see cmdk-sessions-scrub-notes.md). Without a slug we
      // can't stop cleanly; surface a toast and bail rather than
      // fabricating a 400-bound request.
      const summary = bar!.allSessions().find((x: SessionSummary) => x.id === id)
      const slug = summary?.slug
      if (!slug) {
        bar!.sessionPickerOpen.set(false)
        document.dispatchEvent(new CustomEvent('sui:notify', {
          detail: {
            tone: 'warning',
            title: 'End session',
            description: 'Session is not locally dispatchable; cleared from view.',
          },
        }))
        if (bar!.session()?.id === id) bar!.session.set(null)
        return
      }
      void ops.agents.stop({ slug })
        .catch(() => {/* best-effort */})
        .then(() => {
          if (bar!.session()?.id === id) bar!.session.set(null)
          void reloadSessions()
        })
      bar!.sessionPickerOpen.set(false)
    }

    function selectSession(id: string): void {
      const s = bar!.allSessions().find((x: SessionSummary) => x.id === id)
      if (s) { bar!.session.set(s); bar!.thread.set([]) }
      bar!.sessionPickerOpen.set(false)
    }

    function startNewSession(): void {
      bar!.sessionPickerOpen.set(false)
      bar!.mode.set('ai')
      bar!.session.set(null)
    }

    function viewChats(): void {
      bar!.sessionPickerOpen.set(false)
      navigate('/chats')
      bar!.open.set(false)
    }

    const hostEl = host as HTMLElement | null
    function activateNavItem(path: string): void {
      // Dispatch before closing: sui-dialog gates its <slot> on {#if open()},
      // so flipping open=false synchronously detaches the projection path and
      // a later composed event from the palette host can't reach the bar host.
      hostEl?.dispatchEvent(new CustomEvent('sail-command:select', {
        detail: { mode: 'find', item: { path } }, bubbles: true, composed: true,
      }))
      navigate(path)
      bar!.value.set('')
      bar!.open.set(false)
}),

  on: {
    onInput: (s: any, e: Event) => {
      const v = (e.target as HTMLInputElement).value
      s.bar.value.set(v)
      s.bar.mode.set(detectMode(v, s.bar.mode()))
      s.bar.selectedIdx.set(0)
    },
    toggleSessionPicker: (s: any) => {
      s.bar.sessionPickerOpen.set(!s.bar.sessionPickerOpen())
    },
    selectSurface: (s: any, id: CreateSurface) => {
      const def = CREATE_SURFACES.find((x) => x.id === id)
      // Mirror createSurfaces' launch-tier gate — clicking a disabled
      // tile is a no-op so users can't reach the form for a surface
      // whose backend scaffold isn't wired yet.
      const launch: ReadonlySet<CreateSurface> = new Set<CreateSurface>([
        'view', 'skill', 'agent', 'theme',
      ])
      if (!launch.has(id)) return
      s.bar.createSurface.set(id)
      if (def?.shapes[0]) s.bar.createShape.set(def.shapes[0])
    },
    submitCreate: async (s: any) => {
      if (!s.createNameValid() || s.bar.createSubmitting()) return
      // The create-surface op was quarantined (06153c1a5) — scaffolding a
      // view/skill/agent/theme has no daemon op yet. Surface that honestly
      // instead of calling a dead RPC; the backend op is raised for build.
      s.bar.createError.set('Create is not available yet — backend op pending.')
    },
  },
})
</controller>

<!--
  <ship-command-panel> — @ship/chrome · the inner command palette panel
  (find / command / mention / create modes). Consumed inside
  <ship-command-palette>. Accesses harness state via useShared.
-->
<!-- FILE_CAP_OVERRIDE: panel template is a multi-mode switch (find/command/mention/create);
     each mode's markup is self-contained and cannot be split into separate files without
     losing the shared search input row and the palette shared-scope bindings. -->

<!-- style block removed: use sx / external CSS -->

<!-- Input row -->
<div>
  <span data-ai={bar.mode() === 'ai'}>{modeGlyph()}</span>
  <input
    type="text"
    autocomplete="off"
    spellcheck="false"
    placeholder={searchPlaceholder()}
    bind:value={bar.value}
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
    <if cond={bar.sessionPickerOpen()}>
      <div role="listbox" aria-label="Sessions">
        <for as="s" of={activeSessions()} key={s.id}>
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
        <if cond={activeSessions().length === 0}>
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

<!-- Body — mode-switched content -->
<div>
  <match on={bar.mode()}>
    <case of={'find'}>
      <for as="group" of={searchGroups()} key={group.kind}>
        <div>{group.label}</div>
        <div role="listbox" aria-label={group.label}>
          <for as="item" of={group.items} key={item.kind + ':' + item.id}>
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
      <if cond={!findHasResults() && findSearching()}>
        <div>Searching…</div>
      </if>
      <if cond={!findHasResults() && !findSearching()}>
        <div>No matches.</div>
      </if>

    </case><case of={'command'}>
      <if cond={cmdItems().length > 0}>
        <div>Actions</div>
        <div role="listbox">
          <for as="item" of={cmdItems()} key={item.key}>
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
      <else>
        <div>No commands match.</div>
      </else></if>

    </case><case of={'mention'}>
      <if cond={mentionItems().length > 0}>
        <div>Reference</div>
        <div role="listbox">
          <for as="item" of={mentionItems()} key={item.id + item.kind}>
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
      <else>
        <div>No matching views or agents.</div>
      </else></if>

    </case><case of={'create'}>
      <div>Create</div>
      <div role="group" aria-label="Surface type">
        <for as="surf" of={createSurfaces()} key={surf.id}>
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
            <if cond={surf.disabled}>
              <span>post-launch</span>
            </if>
          </button>
        </for>
      </div>
      <div>
        <label>
          <span>Name (kebab-case)</span>
          <input type="text" placeholder={'my-' + bar.createSurface()} bind:value={bar.createName} autocomplete="off" />
        </label>
        <label>
          <span>{currentSurfaceDef().shapeLabel}</span>
          <select bind:value={bar.createShape}>
            <for as="sh" of={currentSurfaceDef().shapes} key={sh}>
              <option value={sh}>{sh}</option>
            </for>
          </select>
        </label>
        <label>
          <span>Purpose (optional)</span>
          <input type="text" placeholder="One-line description" bind:value={bar.createPurpose} autocomplete="off" />
        </label>
      </div>
      <if cond={bar.createError()}>
        <div>{bar.createError()}</div>
      </if>
      <div>
        <button
          type="button"
          disabled={bar.createSubmitting() || !createNameValid()}
          on:click={submitCreate}
        >{bar.createSubmitting() ? 'Creating…' : 'Create ' + currentSurfaceDef().label}</button>
      </div>

    </case><default>
      <!-- ai mode is owned by <ship-command-chat> via isChat(); the panel
           never renders for mode='ai'. This branch is a defensive
           no-op for any transient unknown mode. -->
      <div>…</div>
  </default></match>
</div>
