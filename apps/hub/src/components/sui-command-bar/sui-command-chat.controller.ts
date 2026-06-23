import { controller, derived, useShared } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-command-chat.sui'
import { type SessionSummary, type CommandBarScope } from './types'
import { ops } from '../../lib/client'
import { navigate } from '@ship/deck/router'

const RECENT_MAX = 20

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const bar = useShared<CommandBarScope>(host!, 'sui-command-bar')

    const headerDotBg = derived(() => {
      const s = bar!.session()
      if (s?.status === 'thinking') return 'var(--warning)'
      return 'var(--success)'
    })

    const activeSessions = derived(() =>
      bar!.allSessions().filter((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
    )

    const hasSession = derived(() => bar!.session() !== null)
    const hasAgents = derived(() => bar!.agents().length > 0)
    const hasProviders = derived(() => bar!.harnesses().length > 0)
    const daemonOffline = derived(() => bar!.daemonOnline() === false)

    function endSession(id: string): void {
      // Connect: agents.stop takes the dispatch *slug* (NOT the pty session id
      // the SSE bridge keyed on). Picker rows from agents.sessions don't carry
      // a slug yet — see SessionSummary.slug + cmdk-sessions-scrub-notes.md.
      // Without one we don't fabricate a request the server would 400 on;
      // fall through to optimistic UI drop + a warning toast.
      const summary = bar!.allSessions().find((x: SessionSummary) => x.id === id)
      const slug = summary?.slug
      if (!slug) {
        if (bar!.session()?.id === id) bar!.session.set(null)
        bar!.sessionPickerOpen.set(false)
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
      navigate('/chats')
      bar!.open.set(false)
    }

    function goToAgents(): void {
      navigate('/studio/agents')
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
      // Ported off the dead spawn route onto the unified
      // agents.dispatch Connect op (apps/shipd/.../rpc_wire.rs:624). The
      // returned slug is the handle steer/stop take; we save it on the
      // SessionSummary so subsequent messages in this session can address
      // it without a server-side lookup. The bar's loadCurrentSession
      // picks up the new row via ship:session:changed (fired by the
      // agents.session.started kernel event projection).
      const agent = s.bar.spawnAgent()
      const harness = s.bar.spawnHarness()
      if (!agent || !harness) return
      s.bar.spawning.set(true)
      s.bar.spawnError.set(null)
      try {
        const workspace = await ops.space
          .active({})
          .then((r) => r.active?.id ?? '')
          .catch(() => '')
        if (!workspace) {
          s.bar.spawnError.set('No active project — cannot spawn session.')
          return
        }
        const result = await ops.agents.dispatch({
          agent,
          harness,
          workspace,
          // Chat is conversational; pty invocation lets the harness run
          // interactively. Empty invocation defaults to Pty server-side
          // (rpc_wire.rs:647-651).
          keepalive: true,
          openTab: false,
        })
        // Stash the slug on the matching session row once loadCurrentSession
        // picks it up; optimistically bind the session signal so a user
        // typing right after dispatch can be addressed immediately.
        if (result.sessionId && result.slug) {
          const existing = s.bar.allSessions().find((x: SessionSummary) => x.id === result.sessionId)
          if (existing) existing.slug = result.slug
          const current = s.bar.session()
          if (current?.id === result.sessionId) {
            s.bar.session.set({ ...current, slug: result.slug })
          } else if (!current) {
            s.bar.session.set({
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
  const slug = s.bar.session()!.slug

  // Optimistic render: append user bubble, clear composer + history idx.
  s.bar.thread.set([
    ...s.bar.thread(),
    { id: String(Date.now() + Math.random()), sender: 'user', text },
  ])
  s.bar.value.set('')
  s.bar.historyIdx.set(-1)
  s.bar.sendError.set(null)

  // Connect: agents.steer takes the dispatch slug. If the active session has
  // no slug (loaded from agents.sessions before the cmdk that owns it gets a
  // chance to inject one), we rewind the optimistic bubble and surface a
  // dismissible error rather than fabricating a 400-bound request.
  if (!slug) {
    restoreOnFailure(s, text, 'Send failed: session has no dispatch handle yet — try again in a moment.')
    return
  }

  try {
    await ops.agents.steer({ slug, message: text })
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
