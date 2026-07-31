<controller>
// ship-command-bar.controller.ts — @ship/chrome · the cmdk keystone.
//
// Consolidated from the shell's three-controller command surface (palette
// input row + palette body + chat body). The shell version was a
// three-controller tree sharing state via `useShared`; the pack
// consolidates the three into one — input row, palette body, chat body
// all live in the same shadow root (`ship-command-bar.sui`).
//
// Audit refactors applied:
//   1. `chrome.navigate` and `window.ship?.navigate` are gone. Navigation
//      routes through `navigate()` from `@ship/deck-router` (unified router).
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

import { controller, signal, derived, effect, ref, shared, type Signal, type ViewScope } from '@ship/sail-runtime'
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

export default controller((init: ViewScope<Scope> & { open: Signal<boolean>; mode: Signal<BarMode> }, host?: Element) => {

    const h = host as HTMLElement | undefined
    // `open`/`mode` are `<!-- <prop> -->`s the runtime injects onto the scope before
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
})

// keep the helper export alive (referenced above)
export type { CommandBarScope }
</controller>

<!--
  ship-command-bar.sail — @ship/chrome · the cmdk keystone. ★ the one chrome locus.

  Consolidated from the shell's three-component command surface (palette +
  chat + bar). The shell version was a
  three-component tree sharing state via `useShared`; the pack consolidates
  them into a single shadow root so the 4-mode keystone surface ships as
  one tag. Slotted custom children can still consume the shared scope
  through `useShared<CommandBarScope>(host, 'ship-command-bar')` — the
  host provisions it via `shared(...)` wrappers.

  Surface (4-mode keystone, preserved from the launch pack):
    Modes      : find · search · create · ai (reflected `mode` attribute)
    Props      : open (bool, reflect), mode (string, reflect)
    Methods    : show(mode?, query?) · hide() · toggle(mode?) · setMode(mode)
    Events     : ship-open / ship-close (⌘K · esc) · ship-mode-change {mode} ·
                 ship-submit {query, mode} (↵) · ship-select {id, kind, path}
    Document   : ship:bar:open {mode?, prefill?} · ship:create:open

  The shell's chrome.* refactor lands here: the audit drops
  `chrome.navigate` and `window.ship?.navigate`; the pack uses
  `navigate()` from `@ship/deck-router` instead.
-->

<!-- <prop name="open" type="bool" default="false" reflect /> -->
<!-- <prop name="mode" type="string" default="ai" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<!-- Trigger pill -->
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
 
  bind:this={dialogRef}
>
  <div>
    <!-- ── Input row (shared) ────────────────────────────────── -->
    <div>
      <span data-ai={isChat()}>{modeGlyph()}</span>
      <input
        type="text"
        autocomplete="off"
        spellcheck="false"
        placeholder={searchPlaceholder()}
        bind:value={value}
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
        <if cond={sessionPickerOpen()}>
          <div role="listbox" aria-label="Sessions">
            <for as="s" of={activeSessions()} key={s.id}>
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

    <!-- ── Body: mode-switched content ──────────────────────── -->
    <if cond={isChat()}>
      <!-- ai mode: chat panel -->
      <div>
        <if cond={daemonOffline()}>
          <div role="status">
            <icon name="alert-triangle" size="sm" aria-hidden="true"></icon>
            <span>Daemon unreachable. Reconnect to send messages.</span>
          </div>
        </if>
        <if cond={sendError()}>
          <div role="alert">
            <icon name="alert-circle" size="sm" aria-hidden="true"></icon>
            <span>{sendError()}</span>
            <button type="button" aria-label="Dismiss" on:click={dismissSendError}>×</button>
          </div>
        </if>
        <div>
          <if cond={hasSession()}>
            <if cond={thread().length === 0}>
              <div>
                <div>Connected to {session()?.agent_id ?? 'agent'}</div>
                <div>Type below and press Enter. ArrowUp recalls recent messages.</div>
              </div>
            <else>
              <for as="msg" of={thread()} key={msg.id}>
                <div data-sender={msg.sender}>
                  <div>{msg.text}</div>
                </div>
              </for>
            </else></if>
          <else-if cond={!hasAgents()}>
            <div>
              <div>No agents configured</div>
              <div>Open <button type="button" on:click={goToAgents}>/studio/agents</button> to add one, then start a session.</div>
            </div>
          </else-if><else>
            <div>
              <div>Start a session</div>
              <div>
                <label>Agent</label>
                <select bind:value={spawnAgent}>
                  <for as="a" of={agents()} key={a.id}>
                    <option value={a.id}>{a.name ?? a.label ?? a.id}</option>
                  </for>
                </select>
              </div>
              <div>
                <label>Harness</label>
                <if cond={hasProviders()}>
                  <select bind:value={spawnHarness}>
                    <for as="p" of={harnesses()} key={p}>
                      <option value={p}>{p}</option>
                    </for>
                  </select>
                <else>
                  <span>No harnesses installed</span>
                </else></if>
              </div>
              <if cond={spawnError()}>
                <div>{spawnError()}</div>
              </if>
              <button
                type="button"
                disabled={spawning() || !spawnAgent() || !spawnHarness()}
                on:click={spawnSession}
              >{spawning() ? 'Starting…' : 'Start session'}</button>
            </div>
          </else></if>
        </div>
        <div>
          <textarea
            rows="2"
            placeholder={hasSession() ? 'Message the agent…' : 'Draft a message — start a session above to send'}
            bind:value={value}
            on:keydown={onComposerKeydown}
          ></textarea>
          <button
            type="button"
            disabled={!value().trim() || !hasSession()}
            on:click={sendMessage}
          >Send</button>
        </div>
      </div>
    <else>
      <!-- find / command / mention / create mode: palette body -->
      <div>
        <match on={mode()}>
          <case of={'find'}>
            <for as="group" of={searchGroups()} key={group.kind}>
              <div>{group.label}</div>
              <div role="listbox" aria-label={group.label}>
                <for as="item" of={group.items} key={item.kind + ':' + item.id}>
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
                  aria-pressed={createSurface() === surf.id ? 'true' : 'false'}
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
                <input type="text" placeholder={'my-' + createSurface()} bind:value={createName} autocomplete="off" />
              </label>
              <label>
                <span>{currentSurfaceDef().shapeLabel}</span>
                <select bind:value={createShape}>
                  <for as="sh" of={currentSurfaceDef().shapes} key={sh}>
                    <option value={sh}>{sh}</option>
                  </for>
                </select>
              </label>
              <label>
                <span>Purpose (optional)</span>
                <input type="text" placeholder="One-line description" bind:value={createPurpose} autocomplete="off" />
              </label>
            </div>
            <if cond={createError()}>
              <div>{createError()}</div>
            </if>
            <div>
              <button
                type="button"
                disabled={createSubmitting() || !createNameValid()}
                on:click={submitCreate}
              >{createSubmitting() ? 'Creating…' : 'Create ' + currentSurfaceDef().label}</button>
            </div>

          </case><default>
            <div>…</div>
        </default></match>
      </div>
    </else></if>
  </div>
</dialog>
