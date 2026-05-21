import { controller, derived, useShared } from '@ship/sui-runtime'
import type { Scope } from './sui-command-chat.sui'
import type { Scope as BarScope } from './sui-command-bar.sui'
import { type SessionSummary, navigateTo } from './types'

const RECENT_MAX = 20

export default controller<Scope>({
  scope: (_, host) => {
    const bar = useShared<BarScope>(host!, 'sui-command-bar')

    const headerDotBg = derived(() => {
      const s = bar!.session()
      if (s?.status === 'thinking') return 'var(--warning,#d97706)'
      return 'var(--success,#16a34a)'
    })

    const activeSessions = derived(() =>
      bar!.allSessions().filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
    )

    const hasSession = derived(() => bar!.session() !== null)
    const hasAgents = derived(() => bar!.agents().length > 0)
    const hasProviders = derived(() => bar!.harnesses().length > 0)
    const daemonOffline = derived(() => bar!.daemonOnline() === false)

    function endSession(id: string): void {
      void fetch(`/api/v1/sessions/${id}`, { method: 'DELETE', credentials: 'same-origin' })
        .catch(() => {/* best-effort */})
      if (bar!.session()?.id === id) bar!.session.set(null)
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
      void navigateTo('/chats')
      bar!.open.set(false)
    }

    function goToAgents(): void {
      void navigateTo('/studio/agents')
      bar!.open.set(false)
    }

    return {
      bar, headerDotBg, activeSessions,
      hasSession, hasAgents, hasProviders, daemonOffline,
      endSession, selectSession, startNewSession, viewChats, goToAgents,
    }
  },

  on: {
    toggleSessionPicker: (s: any) => {
      s.bar.sessionPickerOpen.set(!s.bar.sessionPickerOpen())
    },
    sendMessage: async (s: any) => {
      await sendCurrent(s)
    },
    spawnSession: async (s: any) => {
      // Mirrors palette.spawnSession but lives in chat too so the unified
      // ai mode (chat panel always renders) can start a session without
      // bouncing back through the palette.
      const agent = s.bar.spawnAgent()
      const harness = s.bar.spawnHarness()
      if (!agent || !harness) return
      s.bar.spawning.set(true)
      s.bar.spawnError.set(null)
      try {
        const projResp = await fetch('/api/v1/projects/active', { credentials: 'same-origin' })
        const projBody = projResp.ok ? await projResp.json() as { active?: { id?: string } | null } : null
        const workspace_id = projBody?.active?.id
        if (!workspace_id) {
          s.bar.spawnError.set('No active project — cannot spawn session.')
          return
        }
        const resp = await fetch('/api/v1/sessions/spawn', {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ workspace_id, kind: 'agent', agent_id: agent, harness }),
        })
        if (!resp.ok) {
          const err = await resp.json().catch(() => ({})) as { error?: string }
          s.bar.spawnError.set(err.error ?? `HTTP ${resp.status}`)
        }
      } catch (err) {
        s.bar.spawnError.set(err instanceof Error ? err.message : 'Unknown error')
      } finally {
        s.bar.spawning.set(false)
      }
    },
    onComposerKeydown: async (s: any, e: KeyboardEvent) => {
      // History navigation: ArrowUp / ArrowDown in an empty composer
      // walks the recent-sends ring. Position −1 means "draft mode";
      // 0 is most-recent send. Down past 0 restores the empty draft.
      const ta = e.target as HTMLTextAreaElement
      const recent = s.bar.recentSends() as string[]
      const idx = s.bar.historyIdx() as number
      const composerEmptyOrAtHistory =
        ta.value.trim() === '' || (idx >= 0 && ta.value === (recent[idx] ?? ''))

      if (e.key === 'ArrowUp' && composerEmptyOrAtHistory && recent.length > 0) {
        e.preventDefault()
        const next = Math.min(recent.length - 1, idx + 1)
        s.bar.historyIdx.set(next)
        const v = recent[next] ?? ''
        s.bar.value.set(v)
        queueMicrotask(() => { ta.setSelectionRange(v.length, v.length) })
        return
      }
      if (e.key === 'ArrowDown' && idx >= 0) {
        e.preventDefault()
        const next = idx - 1
        s.bar.historyIdx.set(next)
        const v = next < 0 ? '' : (recent[next] ?? '')
        s.bar.value.set(v)
        queueMicrotask(() => { ta.setSelectionRange(v.length, v.length) })
        return
      }

      // `cmdk.send_key` controls Enter semantics in ai mode. Default
      // "enter" sends on plain Enter, newline on Shift+Enter (preserves
      // the OG j2 muscle memory). "cmd+enter" inverts: Enter inserts a
      // newline (native), Cmd/Ctrl+Enter sends.
      const sendKey = s.bar.sendKey?.() ?? 'enter'
      const isSubmit = sendKey === 'cmd+enter'
        ? (e.key === 'Enter' && (e.metaKey || e.ctrlKey))
        : (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey)
      if (!isSubmit) return
      e.preventDefault()
      await sendCurrent(s)
    },
    dismissSendError: (s: any) => { s.bar.sendError.set(null) },
  },
})

async function sendCurrent(s: any): Promise<void> {
  const text = s.bar.value().trim()
  if (!text || !s.bar.session()) return
  const sessionId = s.bar.session()!.id

  // Optimistic render: append user bubble, clear composer + history idx.
  s.bar.thread.set([
    ...s.bar.thread(),
    { id: String(Date.now() + Math.random()), sender: 'user', text },
  ])
  s.bar.value.set('')
  s.bar.historyIdx.set(-1)
  s.bar.sendError.set(null)

  try {
    const resp = await fetch(`/api/v1/sessions/${sessionId}/message`, {
      method: 'POST', credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    })
    if (!resp.ok) {
      restoreOnFailure(s, text, `Send failed: HTTP ${resp.status}`)
      return
    }
    // Successful send → record in recent buffer so ArrowUp can recall.
    const recent = s.bar.recentSends() as string[]
    const next = [text, ...recent.filter((t: string) => t !== text)].slice(0, RECENT_MAX)
    s.bar.recentSends.set(next)
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Network error'
    restoreOnFailure(s, text, `Send failed: ${msg}`)
  }
}

function restoreOnFailure(s: any, text: string, message: string): void {
  // Rewind: pop the optimistic user bubble, restore composer text so
  // the user can retry. Surface a dismissible error so the failure
  // isn't silent.
  const thread = s.bar.thread() as Array<{ id: string; sender: string; text: string }>
  const popped = thread[thread.length - 1]
  if (popped && popped.sender === 'user' && popped.text === text) {
    s.bar.thread.set(thread.slice(0, -1))
  }
  if (!s.bar.value()) s.bar.value.set(text)
  s.bar.sendError.set(message)
}
