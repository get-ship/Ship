import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { Ref, Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-rail.sui'
import { setupProjectSwitcher, type Project } from './sui-rail-project-switcher'

// Type augmentation: the `scope:` factory contributes these reactive
// entries to the template scope. The compiler generates `Scope` from
// `<prop>` declarations only, so we widen it here.
interface RailScope extends Scope {
  collapsed: Signal<boolean>
  projectOpen: Signal<boolean>
  projects: Signal<Project[]>
  activeProjectId: Signal<string | null>
  activeView: Signal<string>
  sessionCount: Signal<number>
  projectInitial: () => string
  popoverEl: Ref<HTMLElement>
  projectBadgeEl: Ref<HTMLElement>
  launcherBtnEl: Ref<HTMLElement>
  createBtnEl: Ref<HTMLElement>
  toggleBtn: Ref<HTMLElement>
  missionEl: Ref<HTMLElement>
  navSlot: Ref<HTMLSlotElement>
}

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

export default controller<RailScope>({
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
      if (!cur) return ''
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
      toggleBtn: ref<HTMLElement>(),
      missionEl: ref<HTMLElement>(),
      navSlot: ref<HTMLSlotElement>(),
    }
  },

  mount(state, host) {
    const scope = state as unknown as RailScope

    // ── Toggle (collapse) button ─────────────────────────────────────
    const stopToggleAria = effect(() => {
      const btn = scope.toggleBtn()
      const col = scope.collapsed()
      if (btn) btn.setAttribute('aria-label', col ? 'Show rail' : 'Hide rail')
    })

    const onToggleClick = (): void => {
      const next = !scope.collapsed()
      scope.collapsed.set(next)
      dispatchBubble(host, 'sui-rail:toggle', { collapsed: next })
    }

    const stopToggleBind = effect(() => {
      const btn = scope.toggleBtn()
      if (!btn) return
      btn.addEventListener('click', onToggleClick)
      return () => btn.removeEventListener('click', onToggleClick)
    })

    // ── Project switcher (extracted to sui-rail-project-switcher.ts) ─
    const switcher = setupProjectSwitcher({
      host,
      open: scope.projectOpen,
      projects: scope.projects,
      activeProjectId: scope.activeProjectId,
      popoverEl: scope.popoverEl,
      badgeEl: scope.projectBadgeEl,
    })
    void switcher.loadProjects()

    // ── Launcher / Create buttons ────────────────────────────────────
    const closePopover = (): void => scope.projectOpen.set(false)
    const onLauncherClick = (e: MouseEvent): void => {
      dispatchBubble(host, 'ship:launcher:open', { anchor: e.currentTarget })
      closePopover()
    }
    const onCreateClick = (): void => {
      dispatchBubble(host, 'ship:create:open')
      closePopover()
    }

    const stopLauncherBind = effect(() => {
      const btn = scope.launcherBtnEl()
      if (!btn) return
      btn.addEventListener('click', onLauncherClick)
      return () => btn.removeEventListener('click', onLauncherClick)
    })
    const stopCreateBind = effect(() => {
      const btn = scope.createBtnEl()
      if (!btn) return
      btn.addEventListener('click', onCreateClick)
      return () => btn.removeEventListener('click', onCreateClick)
    })

    // ── Session count polling ────────────────────────────────────────
    const pollSessions = async (): Promise<void> => {
      try {
        const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = (await resp.json()) as { sessions: Array<{ status: string }> }
        const live = body.sessions.filter((s) => s.status !== 'ended').length
        scope.sessionCount.set(live)
      } catch { /* leave previous count */ }
    }

    void pollSessions()
    const poll = window.setInterval(() => void pollSessions(), SESSION_POLL_MS)
    const onExternalChange = (): void => void pollSessions()
    window.addEventListener('ship:session:changed', onExternalChange)

    const stopBadgeEffect = effect(() => {
      const item = scope.missionEl()
      if (!item) return
      const n = scope.sessionCount()
      item.setAttribute('badge', n > 0 ? String(n) : '')
    })

    // ── Active-view tracking ─────────────────────────────────────────
    const recomputeActive = (): void => {
      scope.activeView.set(routeToViewName(location.pathname))
    }
    const onNavigate = (): void => recomputeActive()
    document.addEventListener('sui:navigate', onNavigate)
    window.addEventListener('popstate', onNavigate)

    const stopActiveSync = effect(() => {
      const name = scope.activeView()
      const items = host.querySelectorAll<HTMLElement>('sui-rail-item[data-view]')
      for (const item of items) {
        if (item.dataset.view === name) item.setAttribute('active', '')
        else item.removeAttribute('active')
      }
      // Mission Control is in shadow DOM, not light children.
      const mc = scope.missionEl()
      if (mc) {
        if (name === 'mesh') mc.setAttribute('active', '')
        else mc.removeAttribute('active')
      }
    })

    return () => {
      stopToggleAria()
      stopToggleBind()
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
