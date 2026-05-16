import { controller, derived, useShared } from '@ship/sui-runtime'
import type { Scope } from './sui-command-chat.sui'
import type { Scope as BarScope } from './sui-command-bar.sui'
import { type SessionSummary, navigateTo } from './types'

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

    return { bar, headerDotBg, activeSessions, endSession, selectSession, startNewSession, viewChats }
  },

  on: {
    toggleSessionPicker: (s: any) => {
      s.bar.sessionPickerOpen.set(!s.bar.sessionPickerOpen())
    },
    sendMessage: async (s: any) => {
      const text = s.bar.value().trim()
      if (!text || !s.bar.session()) return
      const sessionId = s.bar.session()!.id
      s.bar.thread.set([
        ...s.bar.thread(),
        { id: String(Date.now() + Math.random()), sender: 'user', text },
      ])
      s.bar.value.set('')
      try {
        await fetch(`/api/v1/sessions/${sessionId}/message`, {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text }),
        })
      } catch { /* stream delivers reply; ignore send errors */ }
    },
    onComposerKeydown: async (s: any, e: KeyboardEvent) => {
      // `cmdk.send_key` controls Enter semantics in ai mode. Default
      // "enter" sends on plain Enter, newline on Shift+Enter (preserves
      // the OG j2 muscle memory). "cmd+enter" inverts: Enter inserts a
      // newline (native), Cmd/Ctrl+Enter sends — useful for users who
      // type multi-line prompts and don't want to send mid-thought.
      const sendKey = s.bar.sendKey?.() ?? 'enter'
      const isSubmit = sendKey === 'cmd+enter'
        ? (e.key === 'Enter' && (e.metaKey || e.ctrlKey))
        : (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey)
      if (!isSubmit) return
      e.preventDefault()
      const text = s.bar.value().trim()
      if (!text || !s.bar.session()) return
      const sessionId = s.bar.session()!.id
      s.bar.thread.set([
        ...s.bar.thread(),
        { id: String(Date.now() + Math.random()), sender: 'user', text },
      ])
      s.bar.value.set('')
      try {
        await fetch(`/api/v1/sessions/${sessionId}/message`, {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text }),
        })
      } catch { /* ignore */ }
    },
  },
})
