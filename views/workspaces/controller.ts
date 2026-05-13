import { controller, signal, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface WorkspaceRow {
  id: string
  name: string
  path: string
  label?: string
  active?: boolean
}

interface WorkspacesResponse {
  items: WorkspaceRow[]
  active: string | null
}

export default controller<Scope>({
  scope: ({ ship }) => {
    const activatingId = signal<string>('')
    const activationError = signal<string>('')
    // Optimistic active-id override. Set immediately on click so the
    // selected row reflects the user's intent without waiting for the
    // SSE patch from `config.changed`. Cleared once the live payload
    // reports the same id (or on error).
    const pendingActiveId = signal<string>('')

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any

    async function activate(id: string): Promise<void> {
      activatingId.set(id)
      activationError.set('')
      pendingActiveId.set(id)
      try {
        await s.workspaces.activate({ id })
        // Live `<data>` projection picks up the new active id via
        // `config.changed` and re-emits — derived `effectiveActiveId`
        // falls back to the projection value once it agrees.
      } catch (e) {
        pendingActiveId.set('')
        activationError.set((e as Error).message)
      } finally {
        activatingId.set('')
      }
    }

    return {
      activatingId,
      activationError,
      pendingActiveId,
      activate,
    }
  },
  derived: {
    items: ({ projects }): WorkspaceRow[] =>
      (unwrap(projects) as WorkspacesResponse | undefined)?.items ?? [],
    activeId: ({ projects }): string =>
      (unwrap(projects) as WorkspacesResponse | undefined)?.active ?? '',
    // Optimistic overlay — show the pending id until the projection
    // confirms it, then fall back to the daemon-reported value.
    effectiveActiveId: ({ projects, pendingActiveId }): string => {
      const live = (unwrap(projects) as WorkspacesResponse | undefined)?.active ?? ''
      const pending = pendingActiveId()
      return pending && pending !== live ? pending : live
    },
  },
})
