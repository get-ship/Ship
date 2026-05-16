// Project-switcher feature for <sui-rail>.
//
// Extracted to keep sui-rail.controller.ts under the 300-LOC cap.
// Owns: project fetch + switch + add, popover positioning against the
// project-badge bounding rect, document-level click-outside dismiss,
// and the singleton <style> injection for the portaled popover (whose
// nodes <sui-portal> teleports out of shadow-DOM reach).

import { chrome, effect, type Signal } from '@ship/sui-runtime'

export interface Project {
  id: string
  name: string
  path: string
  label?: string
}

interface ProjectsResponse {
  projects: Project[]
  active?: string
}

const POPOVER_STYLES = `
.sui-rail-project-popover {
  position: fixed;
  z-index: var(--z-popover, 1000);
  min-width: 220px;
  background: var(--popover, var(--card, #fff));
  color: var(--popover-foreground, var(--card-foreground, #111));
  border: 1px solid var(--border, #e5e5e5);
  border-radius: var(--radius-md, 8px);
  box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.15));
  overflow: hidden;
  font-family: var(--font-body, system-ui);
  font-size: var(--text-base, 14px);
}
.sui-rail-project-popover-title {
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted-foreground);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.sui-rail-project-popover-row {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
  transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
}
.sui-rail-project-popover-row:hover {
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
}
.sui-rail-project-popover-row.active {
  background: color-mix(in oklch, var(--primary) 12%, transparent);
  color: var(--primary);
}
.sui-rail-project-popover-row-name {
  font-size: 13px;
  font-weight: 500;
}
.sui-rail-project-popover-row-path {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sui-rail-project-popover-sep {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}
.sui-rail-project-popover-row.add {
  color: var(--primary);
  font-weight: 500;
}
.sui-rail-project-popover-empty {
  padding: 12px;
  text-align: center;
  font-size: 12px;
  color: var(--muted-foreground);
}
`

let stylesInstalled = false
export function ensurePopoverStyles(): void {
  if (stylesInstalled) return
  stylesInstalled = true
  const el = document.createElement('style')
  el.setAttribute('data-sui-rail', '')
  el.textContent = POPOVER_STYLES
  document.head.appendChild(el)
}

export interface SwitcherCtx {
  host: HTMLElement
  open: Signal<boolean>
  projects: Signal<Project[]>
  activeProjectId: Signal<string | null>
  popoverEl: () => HTMLElement | null
  badgeEl: () => HTMLElement | null
}

export interface SwitcherHandle {
  loadProjects(): Promise<void>
  dispose(): void
}

function dispatchBubble(host: HTMLElement, type: string, detail?: unknown): void {
  host.dispatchEvent(new CustomEvent(type, {
    detail,
    bubbles: true,
    composed: true,
  }))
}

export function setupProjectSwitcher(ctx: SwitcherCtx): SwitcherHandle {
  ensurePopoverStyles()
  const { host, open, projects, activeProjectId, popoverEl, badgeEl } = ctx

  const closePopover = (): void => open.set(false)

  const loadProjects = async (): Promise<void> => {
    try {
      const resp = await fetch('/api/v1/projects', { credentials: 'same-origin' })
      if (!resp.ok) return
      const body = (await resp.json()) as ProjectsResponse
      projects.set(body.projects ?? [])
      activeProjectId.set(body.active ?? null)
    } catch { /* leave previous state */ }
  }

  const switchProject = async (id: string): Promise<void> => {
    const previousActive = activeProjectId()
    // Optimistic update so the popover row + badge initial reflect the
    // pick immediately. Reverted on a non-OK response.
    activeProjectId.set(id)
    closePopover()
    try {
      const resp = await fetch(
        `/api/v1/projects/${encodeURIComponent(id)}/activate`,
        {
          method: 'POST',
          credentials: 'same-origin',
        },
      )
      if (!resp.ok) {
        activeProjectId.set(previousActive)
        chrome.notify({
          tone: 'destructive',
          title: `Switch failed (${resp.status})`,
        })
        return
      }
      // The kernel-bus `project.switched` event drives the chrome's
      // view-remount + status-bar refresh path; chrome subscribes in
      // shell2/src/main.ts. Keep the legacy DOM event so any existing
      // listeners (rail tests, devtools) continue to fire.
      dispatchBubble(host, 'ship:project:change', { id })
    } catch {
      activeProjectId.set(previousActive)
    }
    void loadProjects()
  }

  const addProjectFlow = async (): Promise<void> => {
    const path = window.prompt('Path to project directory:')
    if (!path) return
    try {
      const resp = await fetch('/api/v1/projects', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
      })
      if (!resp.ok) {
        const err = await resp.text().catch(() => '')
        chrome.notify({
          tone: 'warning',
          title: `Add failed: ${err || resp.status}`,
        })
        return
      }
      await loadProjects()
      chrome.notify({ tone: 'success', title: 'Project added.' })
    } catch (e) {
      chrome.notify({
        tone: 'destructive',
        title: String((e as Error).message ?? e),
      })
    }
  }

  const positionPopover = (): void => {
    const pop = popoverEl()
    const anchor = badgeEl()
    if (!pop || !anchor) return
    const r = anchor.getBoundingClientRect()
    const popWidth = pop.offsetWidth || 220
    const gap = 8
    const left = Math.max(8, Math.min(r.left, window.innerWidth - popWidth - 8))
    const bottom = Math.max(8, window.innerHeight - r.top + gap)
    pop.style.left = `${left}px`
    pop.style.bottom = `${bottom}px`
  }

  const onPopoverClick = (e: Event): void => {
    const target = e.target as HTMLElement | null
    if (!target) return
    if (target.closest('[data-rail-add]')) {
      void addProjectFlow()
      return
    }
    const row = target.closest('[data-project-id]') as HTMLElement | null
    if (row?.dataset.projectId) void switchProject(row.dataset.projectId)
  }

  const onBadgeClick = (): void => open.set(!open())

  const onDocClick = (e: MouseEvent): void => {
    if (!open()) return
    const path = typeof e.composedPath === 'function' ? e.composedPath() : []
    const pop = popoverEl()
    const badge = badgeEl()
    for (const node of path) {
      if (node === pop || node === badge) return
    }
    closePopover()
  }

  const onWindowChange = (): void => {
    if (open()) positionPopover()
  }

  const stopBadgeBind = effect(() => {
    const btn = badgeEl()
    if (!btn) return
    btn.addEventListener('click', onBadgeClick)
    return () => btn.removeEventListener('click', onBadgeClick)
  })

  const stopPopoverEffect = effect(() => {
    const pop = popoverEl()
    if (!pop) return
    requestAnimationFrame(positionPopover)
    pop.addEventListener('click', onPopoverClick)
    return () => pop.removeEventListener('click', onPopoverClick)
  })

  document.addEventListener('click', onDocClick, true)
  window.addEventListener('resize', onWindowChange)
  window.addEventListener('scroll', onWindowChange, { capture: true, passive: true })

  return {
    loadProjects,
    dispose() {
      stopBadgeBind()
      stopPopoverEffect()
      document.removeEventListener('click', onDocClick, true)
      window.removeEventListener('resize', onWindowChange)
      window.removeEventListener('scroll', onWindowChange, true)
    },
  }
}
