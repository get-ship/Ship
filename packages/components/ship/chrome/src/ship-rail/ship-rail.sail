<scope>
  props: {
    collapsed = false
  }
</scope>
<controller>
import { controller, effect, mount, signal } from '@ship/sail-runtime'
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './ship-rail.sail'
import { setupProjectSwitcher, type Project } from './ship-rail-project-switcher'

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

function asHtml(node: Element | null): HTMLElement | null {
  return node instanceof HTMLElement ? node : null
}

export default controller((_ctx: Scope) => {
  const projectOpen = signal(false)
  const projects = signal<Project[]>([])
  const activeProjectId = signal<string | null>(null)
  const activeView = signal<string>(
    typeof location !== 'undefined' ? routeToViewName(location.pathname) : '',
  )
  const popoverEl = signal<Element | null>(null)
  const projectBadgeEl = signal<Element | null>(null)
  const createBtnEl = signal<Element | null>(null)
  const launcherBtnEl = signal<Element | null>(null)

  const capturePopover: Action<void> = (node) => {
    popoverEl.set(node)
    return { destroy: () => popoverEl.set(null) }
  }
  const captureBadge: Action<void> = (node) => {
    projectBadgeEl.set(node)
    return { destroy: () => projectBadgeEl.set(null) }
  }
  const captureCreate: Action<void> = (node) => {
    createBtnEl.set(node)
    return { destroy: () => createBtnEl.set(null) }
  }

  const projectInitial = (): string => {
    const list = projects()
    const cur = list.find((p) => p.id === activeProjectId()) ?? list[0]
    // Always render SOMETHING — `·` placeholder when no project is tracked
    // so the badge isn't a featureless solid color square. Matches j2
    // shell's ship-rail behavior.
    if (!cur) return '·'
    return (cur.label ?? cur.name).charAt(0).toUpperCase()
  }

  mount((host) => {
    // Landmark a11y: declare the rail as a navigation region so assistive
    // tech announces it as a landmark and `aria-label` distinguishes it
    // from other nav landmarks (e.g. top-nav). Done in mount() rather
    // than the template so it cannot be stripped by markup edits.
    if (!host.hasAttribute('role')) host.setAttribute('role', 'navigation')
    if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'Primary navigation')

    const switcher = setupProjectSwitcher({
      host,
      open: projectOpen,
      projects,
      activeProjectId,
      popoverEl: () => asHtml(popoverEl()),
      badgeEl: () => asHtml(projectBadgeEl()),
    })
    void switcher.loadProjects()

    const closePopover = (): void => projectOpen.set(false)
    const onLauncherClick = (e: MouseEvent): void => {
      dispatchBubble(host, 'ship:launcher:open', { anchor: e.currentTarget })
      closePopover()
    }
    const onCreateClick = (): void => {
      dispatchBubble(host, 'ship:create:open')
      closePopover()
    }

    const stopLauncherBind = effect(() => {
      const btn = asHtml(launcherBtnEl())
      if (!btn) return
      btn.addEventListener('click', onLauncherClick)
      return () => btn.removeEventListener('click', onLauncherClick)
    })
    const stopCreateBind = effect(() => {
      const btn = asHtml(createBtnEl())
      if (!btn) return
      btn.addEventListener('click', onCreateClick)
      return () => btn.removeEventListener('click', onCreateClick)
    })

    const recomputeActive = (): void => {
      activeView.set(routeToViewName(location.pathname))
    }
    const onNavigate = (): void => recomputeActive()
    document.addEventListener('sui:navigate', onNavigate)
    window.addEventListener('popstate', onNavigate)

    const stopActiveSync = effect(() => {
      const name = activeView()
      const items = host.querySelectorAll<HTMLElement>('ship-rail-item[data-view]')
      for (const item of items) {
        if (item.dataset.view === name) item.setAttribute('active', '')
        else item.removeAttribute('active')
      }
    })

    // Arrow-key nav — Up/Down cycle focus through rail items, Home/End
    // jump to first/last. Enter activates (browser handles since <a>).
    const rootOf = (el: Element): Element => el.shadowRoot ?? el
    const focusableItems = (): HTMLAnchorElement[] => {
      const list: HTMLAnchorElement[] = []
      for (const item of host.querySelectorAll<HTMLElement>('ship-rail-item')) {
        if (item.hasAttribute('disabled')) continue
        const link = rootOf(item).querySelector('a') as HTMLAnchorElement | null
        if (link) list.push(link)
      }
      return list
    }
    const onKeydown = (e: KeyboardEvent): void => {
      const inRail = (e.target as Node | null) && host.contains(e.target as Node)
      if (!inRail) return
      const list = focusableItems()
      if (list.length === 0) return
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
  })

  return {
    projectOpen,
    projects,
    activeProjectId,
    activeView,
    projectInitial,
    capturePopover,
    captureBadge,
    captureCreate,
  }
})
</controller>

<!--
  <ship-rail> — vertical chrome nav. IS the rail: it owns its own markup
  (left-rail nav with pinned views, a project switcher, launcher trigger,
  and create button) and composes only registered primitives (icon,
  sui-portal) — it does not wrap a layout primitive.

  Two states only:
    • visible  — icon-only (no labels), 48px wide
    • hidden   — display:none (toggled by the top-nav menu button via the
                 `sui:rail:toggle` document event)

  No arrow-toggle button. Rail items show icons only; their `label` prop
  surfaces as the native `title` tooltip on hover.

  Prop / event surface
  ─────────────────────────────────────────────────────────────────────────────
  Props:
    collapsed (bool, false, reflect)  — when set, rail is hidden entirely
  ─────────────────────────────────────────────────────────────────────────────
  Events bubbled from the rail's footer actions:
    ship:launcher:open      { anchor }      — launcher-button click
    ship:create:open        { }             — create-button click
    ship:project:change     { id }          — project-switcher selection
  ─────────────────────────────────────────────────────────────────────────────
  Slots: (default) — accepts <ship-rail-item> and <ship-rail-section> children
  Parts: nav, footer, project-badge, project-popover, launcher-btn,
         create-btn
  ─────────────────────────────────────────────────────────────────────────────
  ARIA: host carries role="navigation" + aria-label="Primary navigation"
        (applied by the controller in mount()). Slotted <ship-rail-item>s
        render as anchors and expose aria-current="page" when active.
  ─────────────────────────────────────────────────────────────────────────────
  Behavior (controller-owned):
    - Project switcher: portal-positioned popover; lists projects from
      the configured list URL; adds new project via prompt.
    - Active-view: listens to sui:navigate (and reads location.pathname);
      mirrors the first path segment onto slotted
      <ship-rail-item[data-view]> children as the `active` attribute.
    - Click-outside dismiss for the project popover.
  ─────────────────────────────────────────────────────────────────────────────
-->

<div>
  <button
    type="button"
    title="Switch project"
    aria-label="Switch project"
    aria-haspopup="dialog"
    aria-expanded={projectOpen() ? 'true' : 'false'}
    use:captureBadge
  >
    <icon name="grid-2x2" size="md"></icon>
  </button>
</div>
<nav>
  <slot></slot>
</nav>
<div>
  <div>
    <button
      type="button"
      title="Create new (⌘N)"
      aria-label="Create new"
      use:captureCreate
    >
      <icon name="plus" size="sm"></icon>
    </button>
    <slot name="footer"></slot>
  </div>
</div>

<!--
  Project popover is teleported to <body> via <sui-portal> so it escapes
  the rail's overflow:hidden + stacking context. The <if> mounts the
  popover only when open. Position is computed by the controller against
  the project-badge's bounding rect.
-->
<sui-portal target="body">
  <if {projectOpen()}>
    <div
      role="dialog"
      aria-label="Switch project"
      use:capturePopover
      transition:fade={{ duration: 150 }}
    >
      <div>Projects</div>
      <if {projects().length === 0}>
        <div>No projects tracked.</div>
      </if>
      <for {projects() as p} key={p.id}>
        <div
          data-active={p.id === activeProjectId()}
          data-project-id={p.id}
        >
          <span>{p.label || p.name}</span>
          <span>{p.path}</span>
          <button
            type="button"
            data-space-open={p.id}
            title="Open this project in its own window"
            aria-label="Open in its own window"
          >
            <icon name="external-link" size="sm"></icon>
          </button>
        </div>
      </for>
      <div></div>
      <div data-rail-add>
        <span>+ Add project</span>
      </div>
    </div>
  </if>
</sui-portal>
