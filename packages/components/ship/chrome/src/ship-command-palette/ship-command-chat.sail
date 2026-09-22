<scope>
  props: {}
</scope>
<controller>
// FILE_CAP_OVERRIDE: ai-mode chat owns session picker + spawn + send in one panel.
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './ship-command-chat.sail'
import { type SessionSummary, commandBarOf } from './types'
import { ops } from './client'
import { navigate } from '@ship/deck-router'

const RECENT_MAX = 20

export default controller((_ctx: Scope, host?: Element) => {
  const bar = commandBarOf(host)

  const headerDotBg = derived(() => {
    const s = bar.session()
    if (s?.status === 'thinking') return 'var(--warning)'
    return 'var(--success)'
  })

  const activeSessions = derived(() =>
    bar.allSessions().filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
  )

  const hasSession = derived(() => bar.session() !== null)
  const hasAgents = derived(() => bar.agents().length > 0)
  const hasProviders = derived(() => bar.harnesses().length > 0)
  const daemonOffline = derived(() => bar.daemonOnline() === false)

  function endSession(id: string): void {
    // Connect: agents.stop takes the dispatch *slug* (NOT the pty session id
    // the SSE bridge keyed on). Picker rows from agents.sessions don't carry
    // a slug yet — see SessionSummary.slug + cmdk-sessions-scrub-notes.md.
    // Without one we don't fabricate a request the server would 400 on;
    // fall through to optimistic UI drop + a warning toast.
    const summary = bar.allSessions().find((x: SessionSummary) => x.id === id)
    const slug = summary?.slug
    if (!slug) {
      if (bar.session()?.id === id) bar.session.set(null)
      bar.sessionPickerOpen.set(false)
      document.dispatchEvent(new CustomEvent('sui:notify', {
        detail: {
          tone: 'warning',
          title: 'End session',
          description: 'Session is not locally dispatchable; cleared from view.',
        },
      }))
      return
    }
    void ops.agents.stop({ slug }).catch(() => {/* best-effort */})
    if (bar.session()?.id === id) bar.session.set(null)
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

  function goToAgents(): void {
    navigate('/studio/agents')
    bar.open.set(false)
  }

  function toggleSessionPicker(): void {
    bar.sessionPickerOpen.set(!bar.sessionPickerOpen())
  }

  function dismissSendError(): void {
    bar.sendError.set(null)
  }

  function onComposerInput(e: Event): void {
    bar.value.set((e.currentTarget as HTMLTextAreaElement).value)
  }

  function onSpawnAgentInput(e: Event): void {
    bar.spawnAgent.set((e.currentTarget as HTMLSelectElement).value)
  }

  function onSpawnHarnessInput(e: Event): void {
    bar.spawnHarness.set((e.currentTarget as HTMLSelectElement).value)
  }

  async function sendMessage(): Promise<void> {
    await sendCurrent(bar)
  }

  async function spawnSession(): Promise<void> {
    const agent = bar.spawnAgent()
    const harness = bar.spawnHarness()
    if (!agent || !harness) return
    bar.spawning.set(true)
    bar.spawnError.set(null)
    try {
      const workspace = await ops.space
        .active({})
        .then((r) => r.active?.id ?? '')
        .catch(() => '')
      if (!workspace) {
        bar.spawnError.set('No active project — cannot spawn session.')
        return
      }
      const result = await ops.agents.dispatch({
        agent,
        harness,
        workspace,
        keepalive: true,
        openTab: false,
      })
      if (result.sessionId && result.slug) {
        const existing = bar.allSessions().find((x: SessionSummary) => x.id === result.sessionId)
        if (existing) existing.slug = result.slug
        const current = bar.session()
        if (current?.id === result.sessionId) {
          bar.session.set({ ...current, slug: result.slug })
        } else if (!current) {
          bar.session.set({
            id: result.sessionId,
            workspace_id: workspace,
            agent_id: agent,
            harness,
            status: 'spawning',
            slug: result.slug,
          })
        }
      }
    } catch (err) {
      bar.spawnError.set(err instanceof Error ? err.message : 'Unknown error')
    } finally {
      bar.spawning.set(false)
    }
  }

  async function onComposerKeydown(e: KeyboardEvent): Promise<void> {
    const ta = e.target as HTMLTextAreaElement
    const recent = bar.recentSends() as string[]
    const idx = bar.historyIdx() as number
    const composerEmptyOrAtHistory =
      ta.value.trim() === '' || (idx >= 0 && ta.value === (recent[idx] ?? ''))

    if (e.key === 'ArrowUp' && composerEmptyOrAtHistory && recent.length > 0) {
      e.preventDefault()
      const next = Math.min(recent.length - 1, idx + 1)
      bar.historyIdx.set(next)
      const v = recent[next] ?? ''
      bar.value.set(v)
      queueMicrotask(() => { ta.setSelectionRange(v.length, v.length) })
      return
    }
    if (e.key === 'ArrowDown' && idx >= 0) {
      e.preventDefault()
      const next = idx - 1
      bar.historyIdx.set(next)
      const v = next < 0 ? '' : (recent[next] ?? '')
      bar.value.set(v)
      queueMicrotask(() => { ta.setSelectionRange(v.length, v.length) })
      return
    }

    const sendKey = bar.sendKey?.() ?? 'enter'
    const isSubmit = sendKey === 'cmd+enter'
      ? (e.key === 'Enter' && (e.metaKey || e.ctrlKey))
      : (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey)
    if (!isSubmit) return
    e.preventDefault()
    await sendCurrent(bar)
  }

  return {
    bar, headerDotBg, activeSessions,
    hasSession, hasAgents, hasProviders, daemonOffline,
    endSession, selectSession, startNewSession, viewChats, goToAgents,
    toggleSessionPicker, sendMessage, spawnSession, onComposerKeydown,
    dismissSendError, onComposerInput, onSpawnAgentInput, onSpawnHarnessInput,
  }
})

async function sendCurrent(bar: ReturnType<typeof commandBarOf>): Promise<void> {
  const text = bar.value().trim()
  if (!text || !bar.session()) return
  const slug = bar.session()!.slug

  bar.thread.set([
    ...bar.thread(),
    { id: String(Date.now() + Math.random()), sender: 'user', text },
  ])
  bar.value.set('')
  bar.historyIdx.set(-1)
  bar.sendError.set(null)

  if (!slug) {
    restoreOnFailure(bar, text, 'Send failed: session has no dispatch handle yet — try again in a moment.')
    return
  }

  try {
    await ops.agents.steer({ slug, message: text })
    const recent = bar.recentSends() as string[]
    const next = [text, ...recent.filter((t: string) => t !== text)].slice(0, RECENT_MAX)
    bar.recentSends.set(next)
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Network error'
    restoreOnFailure(bar, text, `Send failed: ${msg}`)
  }
}

function restoreOnFailure(bar: ReturnType<typeof commandBarOf>, text: string, message: string): void {
  const thread = bar.thread() as Array<{ id: string; sender: string; text: string }>
  const popped = thread[thread.length - 1]
  if (popped && popped.sender === 'user' && popped.text === text) {
    bar.thread.set(thread.slice(0, -1))
  }
  if (!bar.value()) bar.value.set(text)
  bar.sendError.set(message)
}
</controller>

<!--
  <ship-command-chat> — @ship/chrome · chat panel for the unified ai mode.
  Always rendered when bar.mode = 'ai'. Body branches:
    daemonOffline → "Daemon unreachable" empty state
    no agents     → "Set an active agent in /studio/agents"
    no session    → inline spawn picker (agent + harness + Start)
    session       → message thread
  Composer is always present so prefill text from `ship:bar:open`
  is visible + editable before submission.
-->

<div>
  <span style:background={headerDotBg()} aria-hidden="true"></span>
  <div>
    <span>{bar.session()?.agent_id ?? 'No session'}</span>
    <span>{bar.session()?.harness ?? '—'} · {bar.session()?.workspace_id ?? ''}</span>
  </div>
  <span></span>
  <div>
    <span role="button" aria-haspopup="listbox" on:click={toggleSessionPicker}>Switch</span>
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
  <kbd aria-label="Escape to close">esc</kbd>
</div>

<if {daemonOffline()}>
  <div role="status">
    <icon name="alert-triangle" size="sm" aria-hidden="true"></icon>
    <span>Daemon unreachable. Reconnect to send messages.</span>
  </div>
</if>
<if {bar.sendError()}>
  <div role="alert">
    <icon name="alert-circle" size="sm" aria-hidden="true"></icon>
    <span>{bar.sendError()}</span>
    <button type="button" aria-label="Dismiss" on:click={dismissSendError}>×</button>
  </div>
</if>

<div>
  <if {hasSession()}>
    <if {bar.thread().length === 0}>
      <div>
        <div>Connected to {bar.session()?.agent_id ?? 'agent'}</div>
        <div>Type below and press Enter. ArrowUp recalls recent messages.</div>
      </div>
    {else =>
      <for {bar.thread() as msg} key={msg.id}>
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
        <select value={bar.spawnAgent()} on:input={onSpawnAgentInput}>
          <for {bar.agents() as a} key={a.id}>
            <option value={a.id}>{a.name ?? a.label ?? a.id}</option>
          </for>
        </select>
      </div>
      <div>
        <label>Harness</label>
        <if {hasProviders()}>
          <select value={bar.spawnHarness()} on:input={onSpawnHarnessInput}>
            <for {bar.harnesses() as p} key={p}>
              <option value={p}>{p}</option>
            </for>
          </select>
        {else =>
          <span>No harnesses installed</span>
        }</if>
      </div>
      <if {bar.spawnError()}>
        <div>{bar.spawnError()}</div>
      </if>
      <button
        type="button"
        disabled={bar.spawning() || !bar.spawnAgent() || !bar.spawnHarness()}
        on:click={spawnSession}
      >{bar.spawning() ? 'Starting…' : 'Start session'}</button>
    </div>
  }</if>
</div>

<div>
  <textarea
    rows="2"
    placeholder={hasSession() ? 'Message the agent…' : 'Draft a message — start a session above to send'}
    value={bar.value()}
    on:input={onComposerInput}
    on:keydown={onComposerKeydown}
  ></textarea>
  <button
    type="button"
    disabled={!bar.value().trim() || !hasSession()}
    on:click={sendMessage}
  >Send</button>
</div>
