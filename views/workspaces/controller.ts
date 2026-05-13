import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface ProjectsResponse {
  projects: Array<{
    id: string
    name: string
    path: string
    label?: string
  }>
  active?: string | null
}

export default controller<Scope>({
  scope: ({ ship }) => {
    // Cache-buster signal — incremented after a successful activate
    // forces the `live()` thunk to re-fetch.
    const refreshTick = signal(0)
    const activatingId = signal<string>('')
    const activationError = signal<string>('')

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const projects = live<ProjectsResponse>(() => {
      // Read so dependency tracks the tick.
      refreshTick()
      return s.workspaces.list() as Promise<ProjectsResponse>
    })

    async function activate(id: string): Promise<void> {
      activatingId.set(id)
      activationError.set('')
      try {
        await s.workspaces.activate({ id })
        // Refresh the list — daemon emits `config.changed`, but
        // there's no live subscription on this view, so kick it
        // ourselves. The promise-source thunk re-runs.
        refreshTick.set(refreshTick() + 1)
      } catch (e) {
        activationError.set((e as Error).message)
      } finally {
        activatingId.set('')
      }
    }

    return {
      projects,
      activatingId,
      activationError,
      activate,
    }
  },
  derived: {
    items: ({ projects }) =>
      (unwrap(projects) as ProjectsResponse | undefined)?.projects ?? [],
    activeId: ({ projects }): string =>
      (unwrap(projects) as ProjectsResponse | undefined)?.active ?? '',
  },
})
