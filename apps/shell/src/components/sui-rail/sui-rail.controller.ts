import { controller, effect, ref, signal } from '@ship/sui-runtime'
// Two-state rail: visible (icon-only, 48px) vs hidden (display:none).
// The arrow toggle button was removed — the top-nav menu button is the
// sole control surface, dispatching `sui:rail:toggle` which the shell
// app translates into a [collapsed] attribute on the rail.
import type { Scope } from './sui-rail.sui'
import { setupProjectSwitcher, type Project } from './sui-rail-project-switcher'

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
      projectInitial,
      popoverEl: ref<HTMLElement>(),
      projectBadgeEl: ref<HTMLElement>(),
      launcherBtnEl: ref<HTMLElement>(),
      createBtnEl: ref<HTMLElement>(),
      viewPickerEl: ref<HTMLElement>(),
      viewPickerBtnEl: ref<HTMLElement>(),
      navSlot: ref<HTMLSlotElement>(),
    }
  },

  mount(state, host) {
    // Landmark a11y: declare the rail as a navigation region so assistive
    // tech announces it as a landmark and `aria-label` distinguishes it
    // from other nav landmarks (e.g. top-nav). Done in mount() rather
    // than the template so it cannot be stripped by markup edits.
    if (!host.hasAttribute('role')) host.setAttribute('role', 'navigation')
    if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'Primary navigation')

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
    })

    // Arrow-key nav — Up/Down cycle focus through rail items, Home/End
    // jump to first/last. Enter activates (browser handles since <a>).
    // Per OG harvest AVOID list — rail had no keyboard nav previously.
    const focusableItems = (): HTMLAnchorElement[] => {
      const list: HTMLAnchorElement[] = []
      for (const item of host.querySelectorAll<HTMLElement>('sui-rail-item')) {
        if (item.hasAttribute('disabled')) continue
        const link = item.shadowRoot?.querySelector('a.item') as HTMLAnchorElement | null
        if (link) list.push(link)
      }
      return list
    }
    const onKeydown = (e: KeyboardEvent): void => {
      const inRail = (e.target as Node | null) && host.contains(e.target as Node)
      if (!inRail) return
      const list = focusableItems()
      if (list.length === 0) return
      const active = host.shadowRoot?.activeElement
      const fromSlotted = list.findIndex((a) => a.matches(':focus'))
      const idx = fromSlotted >= 0 ? fromSlotted : 0
      let next: number | null = null
      switch (e.key) {
        case 'ArrowDown': next = (idx + 1) % list.length; break
        case 'ArrowUp': next = (idx - 1 + list.length) % list.length; break
        case 'Home': next = 0; break
        case 'End': next = list.length - 1; break
        default: return
      }
      if (next == null) return
      e.preventDefault()
      list[next]?.focus()
      void active // satisfy TS unused-var
    }
    host.addEventListener('keydown', onKeydown)

    return () => {
      stopLauncherBind()
      stopCreateBind()
      stopActiveSync()
      switcher.dispose()
      host.removeEventListener('keydown', onKeydown)
      document.removeEventListener('sui:navigate', onNavigate)
      window.removeEventListener('popstate', onNavigate)
    }
  },
})
