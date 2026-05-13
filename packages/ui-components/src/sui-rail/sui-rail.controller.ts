import { controller, effect, ref, signal } from '@ship/sui-runtime'
// Two-state rail: visible (icon-only, 48px) vs hidden (display:none).
// The arrow toggle button was removed — the top-nav menu button is the
// sole control surface, dispatching `sui:rail:toggle` which the shell
// app translates into a [collapsed] attribute on the rail.
import type { Scope } from './sui-rail.sui'
import { setupProjectSwitcher, type Project } from './sui-rail-project-switcher'

const SESSION_POLL_MS = 15000

function routeToViewName(path: string): string {
  const seg = path.split('/').filter(Boolean)[0]
  return seg ?? 'dashboard'
}

function dispatchBubble(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail,
    bubbles: true,
    composed: true,
  }))
}

export default controller<Scope>({
  scope: () => {
    const projectOpen = signal(false)
    const projects = signal<Project[]>([])
    const activeProjectId = signal<string | null>(null)
    const activeView = signal<string>(
      typeof location !== 'undefined' ? routeToViewName(location.pathname) : '',
    )
    const sessionCount = signal(0)
    const projectInitial = (): string => {
      const list = projects()
      const cur = list.find((p) => p.id === activeProjectId()) ?? list[0]
      // Always render SOMETHING — `·` placeholder when no project is tracked
      // so the badge isn't a featureless solid color square. Matches j2
      // shell's ship-rail behavior.
      if (!cur) return '·'
      return (cur.label ?? cur.name).charAt(0).toUpperCase()
    }
    return {
      projectOpen,
      projects,
      activeProjectId,
      activeView,
      sessionCount,
      projectInitial,
      popoverEl: ref<HTMLElement>(),
      projectBadgeEl: ref<HTMLElement>(),
      launcherBtnEl: ref<HTMLElement>(),
      createBtnEl: ref<HTMLElement>(),
      missionEl: ref<HTMLElement>(),
      navSlot: ref<HTMLSlotElement>(),
    }
  },

  mount(state, host) {
    const switcher = setupProjectSwitcher({
      host,
      open: state.projectOpen,
      projects: state.projects,
      activeProjectId: state.activeProjectId,
      popoverEl: state.popoverEl,
      badgeEl: state.projectBadgeEl,
    })
    void switcher.loadProjects()

    const closePopover = (): void => state.projectOpen.set(false)
    const onLauncherClick = (e: MouseEvent): void => {
      dispatchBubble(host, 'ship:launcher:open', { anchor: e.currentTarget })
      closePopover()
    }
    const onCreateClick = (): void => {
      dispatchBubble(host, 'ship:create:open')
      closePopover()
    }

    const stopLauncherBind = effect(() => {
      const btn = state.launcherBtnEl()
      if (!btn) return
      btn.addEventListener('click', onLauncherClick)
      return () => btn.removeEventListener('click', onLauncherClick)
    })
    const stopCreateBind = effect(() => {
      const btn = state.createBtnEl()
      if (!btn) return
      btn.addEventListener('click', onCreateClick)
      return () => btn.removeEventListener('click', onCreateClick)
    })

    const pollSessions = async (): Promise<void> => {
      try {
        const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = (await resp.json()) as { sessions: Array<{ status: string }> }
        const live = body.sessions.filter((s) => s.status !== 'ended').length
        state.sessionCount.set(live)
      } catch { /* leave previous count */ }
    }

    void pollSessions()
    const poll = window.setInterval(() => void pollSessions(), SESSION_POLL_MS)
    const onExternalChange = (): void => void pollSessions()
    window.addEventListener('ship:session:changed', onExternalChange)

    const stopBadgeEffect = effect(() => {
      const item = state.missionEl()
      if (!item) return
      const n = state.sessionCount()
      item.setAttribute('badge', n > 0 ? String(n) : '')
    })

    const recomputeActive = (): void => {
      state.activeView.set(routeToViewName(location.pathname))
    }
    const onNavigate = (): void => recomputeActive()
    document.addEventListener('sui:navigate', onNavigate)
    window.addEventListener('popstate', onNavigate)

    const stopActiveSync = effect(() => {
      const name = state.activeView()
      const items = host.querySelectorAll<HTMLElement>('sui-rail-item[data-view]')
      for (const item of items) {
        if (item.dataset.view === name) item.setAttribute('active', '')
        else item.removeAttribute('active')
      }
      const mc = state.missionEl()
      if (mc) {
        if (name === 'mesh') mc.setAttribute('active', '')
        else mc.removeAttribute('active')
      }
    })

    return () => {
      stopLauncherBind()
      stopCreateBind()
      stopBadgeEffect()
      stopActiveSync()
      switcher.dispose()
      document.removeEventListener('sui:navigate', onNavigate)
      window.removeEventListener('popstate', onNavigate)
      window.removeEventListener('ship:session:changed', onExternalChange)
      window.clearInterval(poll)
    }
  },
})
