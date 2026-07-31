<controller>
import { controller, effect, ref, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
// Two-state rail: visible (icon-only, 48px) vs hidden (display:none).
// The arrow toggle button was removed — the top-nav menu button is the
// sole control surface, dispatching `sui:rail:toggle` which the shell
// app translates into a [collapsed] attribute on the rail.
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

export default controller((_ctx: ViewScope<Scope>) => {

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
      navSlot: ref<HTMLSlotElement>(),
    }
})
</controller>

<!--
  <ship-rail> — vertical chrome nav. IS the rail: it owns its own markup
  (left-rail nav with pinned views, a project switcher, launcher trigger,
  and create button) and composes only registered primitives (sui-icon,
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
      the configured list URL; adds new project via prompt;
      transition:fade on open/close.
    - Active-view: listens to sui:navigate (and reads location.pathname);
      mirrors the first path segment onto slotted
      <ship-rail-item[data-view]> children as the `active` attribute.
    - Click-outside dismiss for the project popover.
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="collapsed" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <button
    type="button"
    title="Switch project"
    aria-label="Switch project"
    aria-haspopup="dialog"
    aria-expanded={projectOpen() ? 'true' : 'false'}
    bind:this={projectBadgeEl}
  >
    <icon name="grid-2x2" size="md"></icon>
  </button>
</div>
<nav>
  <slot bind:this={navSlot}></slot>
</nav>
<div>
  <div>
    <button
      type="button"
      title="Create new (⌘N)"
      aria-label="Create new"
      bind:this={createBtnEl}
    >
      <icon name="plus" size="sm"></icon>
    </button>
    <slot name="footer"></slot>
  </div>
</div>

<!--
  Project popover is teleported to <body> via <sui-portal> so it escapes
  the rail's overflow:hidden + stacking context. The <if> mounts the
  popover only when open; transition:fade animates entry/exit. Position
  is computed by the controller against the project-badge's bounding
  rect — see updatePopoverPosition() in ship-rail.controller.ts.
-->
<sui-portal target="body">
  <if cond={projectOpen()}>
    <div
      role="dialog"
      aria-label="Switch project"
      bind:this={popoverEl}
      transition:fade={{ duration: 150 }}
    >
      <div>Projects</div>
      <if cond={projects().length === 0}>
        <div>No projects tracked.</div>
      </if>
      <for as="p" of={projects()} key={p.id}>
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
