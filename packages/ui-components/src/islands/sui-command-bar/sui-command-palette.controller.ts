import { controller, derived, useShared } from '@ship/sui-runtime'
import type { Scope } from './sui-command-palette.sui'
import type { Scope as BarScope } from './sui-command-bar.sui'
import {
  type CreateSurface, type SessionSummary,
  NAV_DESTINATIONS, SLASH_COMMANDS, CREATE_SURFACES,
  detectMode, isKebab, navigateTo,
} from './types'

export default controller<Scope>({
  scope: (_, host) => {
    const bar = useShared<BarScope>(host!, 'sui-command-bar')

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
      if (s?.status === 'thinking') return 'var(--warning,#d97706)'
      if (s) return 'var(--success,#16a34a)'
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

    const findItems = derived(() => {
      const needle = bar!.value().replace(/^>+\s*/, '').toLowerCase().trim()
      return NAV_DESTINATIONS
        .filter((d) =>
          !needle ||
          d.label.toLowerCase().includes(needle) ||
          d.keywords.toLowerCase().includes(needle) ||
          d.path.toLowerCase().includes(needle)
        )
        .map((d, i) => ({ ...d, idx: i }))
    })

    const cmdItems = derived(() => {
      const needle = bar!.value().replace(/^\/+\s*/, '').toLowerCase().trim()
      return SLASH_COMMANDS
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

    const createSurfaces = derived(() => CREATE_SURFACES)

    const currentSurfaceDef = derived(() =>
      CREATE_SURFACES.find((s) => s.id === bar!.createSurface())!
    )

    const createNameValid = derived(() => isKebab(bar!.createName()))

    // ── Imperative helpers exposed to template ────────────────────
    function endSession(id: string): void {
      void fetch(`/api/v1/sessions/${id}`, { method: 'DELETE', credentials: 'same-origin' })
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
      void navigateTo('/chats')
      bar!.open.set(false)
    }

    const hostEl = host as HTMLElement | null
    function activateNavItem(path: string): void {
      void navigateTo(path)
      bar!.value.set('')
      bar!.open.set(false)
      hostEl?.dispatchEvent(new CustomEvent('sui-command:select', {
        detail: { mode: 'find', item: { path } }, bubbles: true, composed: true,
      }))
    }

    function activateCmd(key: string): void {
      bar!.value.set('')
      switch (key) {
        case 'new': bar!.mode.set('create'); break
        case 'chats': void navigateTo('/chats'); bar!.open.set(false); break
        case 'settings': void navigateTo('/settings'); bar!.open.set(false); break
        case 'end':
          if (bar!.session()) endSession(bar!.session()!.id)
          bar!.open.set(false)
          break
        case 'close': bar!.open.set(false); break
      }
    }

    function activateMention(item: { kind: string; id: string }): void {
      const ref = item.kind === 'view' ? `@${item.id}` : `@${item.kind}:${item.id}`
      const v = bar!.value()
      bar!.value.set(v.replace(/@[^@\s]*$/, `${ref} `))
      bar!.mode.set(detectMode(bar!.value(), bar!.mode()))
    }

    async function reloadSessions(): Promise<void> {
      try {
        const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
        if (resp.ok) {
          const body = await resp.json() as { sessions: SessionSummary[] }
          bar!.allSessions.set(body.sessions)
          const active = body.sessions.find((s: SessionSummary) => s.status !== 'ended' && s.agent_id)
          if (active && active.id !== bar!.session()?.id) { bar!.session.set(active); bar!.thread.set([]) }
          else if (!active) { bar!.session.set(null) }
        }
      } catch { /* leave as-is */ }
    }

    return {
      bar,
      modeGlyph, searchPlaceholder,
      chipLabel, chipDotBg,
      activeSessions,
      findItems, cmdItems, mentionItems,
      createSurfaces, currentSurfaceDef, createNameValid,
      endSession, selectSession, startNewSession, viewChats,
      activateNavItem, activateCmd, activateMention,
    }
  },

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
      s.bar.createSurface.set(id)
      const def = CREATE_SURFACES.find((x) => x.id === id)
      if (def?.shapes[0]) s.bar.createShape.set(def.shapes[0])
    },
    spawnSession: async (s: any) => {
      const agent = s.bar.spawnAgent()
      const provider = s.bar.spawnProvider()
      if (!agent || !provider) return
      s.bar.spawning.set(true)
      s.bar.spawnError.set(null)
      try {
        const resp = await fetch('/api/v1/sessions', {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ agent_id: agent, provider }),
        })
        if (!resp.ok) {
          const err = await resp.json().catch(() => ({})) as { error?: string }
          s.bar.spawnError.set(err.error ?? `HTTP ${resp.status}`)
        } else {
          s.bar.mode.set('ai')
        }
      } catch (err) {
        s.bar.spawnError.set(err instanceof Error ? err.message : 'Unknown error')
      } finally {
        s.bar.spawning.set(false)
      }
    },
    submitCreate: async (s: any) => {
      if (!s.createNameValid() || s.bar.createSubmitting()) return
      s.bar.createSubmitting.set(true)
      s.bar.createError.set(null)
      try {
        const resp = await fetch('/api/v1/scaffold', {
          method: 'POST', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            surface: s.bar.createSurface(),
            name: s.bar.createName(),
            shape: s.bar.createShape(),
            purpose: s.bar.createPurpose(),
          }),
        })
        if (!resp.ok) {
          const err = await resp.json().catch(() => ({})) as { error?: string }
          s.bar.createError.set(err.error ?? `HTTP ${resp.status}`)
        } else {
          s.bar.createName.set('')
          s.bar.createPurpose.set('')
          s.bar.open.set(false)
        }
      } catch (err) {
        s.bar.createError.set(err instanceof Error ? err.message : 'Unknown error')
      } finally {
        s.bar.createSubmitting.set(false)
      }
    },
  },
})
