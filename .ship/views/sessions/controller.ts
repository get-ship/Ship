import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface SessionItem {
  id: string
  kind: string
  workspace_id: string
  agent_id?: string
  provider?: string
  status: string
  spawned_at: string
  last_activity: string
}

interface SessionsState { items: SessionItem[] }

export default controller<Scope>({
  scope: ({ ship }) => {
    const spawning = signal(false)
    const endingId = signal<string>('')
    const mutationError = signal<string>('')

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any

    // Active workspace lives in `settings.state` — sessions.start
    // requires it. Until the SDK exposes a typed accessor we read
    // from settings out-of-band. Pulling it from the URL params /
    // local settings would also work; settings.state was the
    // closest existing surface.
    async function spawnShell(): Promise<void> {
      spawning.set(true)
      mutationError.set('')
      try {
        const ws = await activeWorkspaceId()
        await s.sessions.start({ workspace_id: ws, kind: 'shell' })
        // No manual refresh — the live `<data>` subscription receives
        // a fresh patch from the daemon's sessions projection.
      } catch (e) {
        mutationError.set((e as Error).message)
      } finally {
        spawning.set(false)
      }
    }

    async function endSession(id: string): Promise<void> {
      endingId.set(id)
      mutationError.set('')
      try {
        await s.sessions.end({ id })
      } catch (e) {
        mutationError.set((e as Error).message)
      } finally {
        endingId.set('')
      }
    }

    return {
      spawning,
      endingId,
      mutationError,
      spawnShell,
      endSession,
    }
  },
  derived: {
    items: ({ sessions }): SessionItem[] =>
      (unwrap(sessions) as SessionsState | undefined)?.items ?? [],
    total: ({ sessions }): number =>
      (unwrap(sessions) as SessionsState | undefined)?.items.length ?? 0,
  },
})

// One-shot fetch — keeps the controller free of an extra live
// subscription just for the active project id. Falls back to the
// empty string, which the daemon's spawn handler rejects with a
// clear validation error so we don't silently no-op.
async function activeWorkspaceId(): Promise<string> {
  try {
    const resp = await fetch('/api/v1/projects', { credentials: 'same-origin' })
    if (!resp.ok) return ''
    const body = (await resp.json()) as { active?: string | null }
    return body.active ?? ''
  } catch {
    return ''
  }
}
