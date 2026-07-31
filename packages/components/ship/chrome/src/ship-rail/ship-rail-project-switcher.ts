// Project-switcher feature for <ship-rail>.
//
// @ship/chrome pack port of the shell's left-rail project switcher.
// The shell version was the source of truth; the chrome pack version
// is self-contained (no shell imports) and uses `fetch` + document
// CustomEvents for host-bound actions.
//
// Owns: project fetch + switch + add, popover positioning against the
// project-badge bounding rect, document-level click-outside dismiss,
// and the singleton <style> injection for the portaled popover (whose
// nodes <sui-portal> teleports out of shadow-DOM reach).
//
// Host contract (the shell wires these):
//   - GET  {listUrl}                     → { spaces, activeId }
//   - POST {listUrl}                     { path, name }   add a project
//   - POST {listUrl}/{id}/activate       { }              switch active
//   - openSpace(id) is inlined below (Tauri invoke vs window.open)

import { effect, type Signal } from '@ship/sail-runtime'

export interface Project {
  id: string
  name: string
  path: string
  label?: string
}

const POPOVER_STYLES = `
.ship-rail-project-popover {
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
.ship-rail-project-popover-title {
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted-foreground);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.ship-rail-project-popover-row {
  position: relative;
  padding: 8px 32px 8px 12px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
  transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
}
.ship-rail-project-popover-row:hover {
  background: var(--hover-bg);
}
.ship-rail-project-popover-row.active {
  background: var(--selected-bg);
  color: var(--primary);
}
.ship-rail-project-popover-row-name {
  font-size: 13px;
  font-weight: 500;
}
.ship-rail-project-popover-row-path {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ship-rail-project-popover-open {
  position: absolute;
  top: 50%;
  right: 6px;
  transform: translateY(-50%);
  appearance: none;
  background: transparent;
  border: none;
  padding: 4px;
  margin: 0;
  cursor: pointer;
  color: var(--muted-foreground);
  border-radius: var(--radius-sm, 4px);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--duration-fast, 100ms) var(--ease-out, ease-out), background-color var(--duration-fast, 100ms) var(--ease-out, ease-out), color var(--duration-fast, 100ms) var(--ease-out, ease-out);
}
.ship-rail-project-popover-row:hover .ship-rail-project-popover-open,
.ship-rail-project-popover-open:focus-visible {
  opacity: 1;
}
.ship-rail-project-popover-open:hover {
  background: var(--hover-bg);
  color: var(--foreground);
}
.ship-rail-project-popover-sep {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}
.ship-rail-project-popover-row.add {
  color: var(--primary);
  font-weight: 500;
}
.ship-rail-project-popover-empty {
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
  el.setAttribute('data-ship-rail', '')
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
  /** Base URL for the project registry. The pack uses fetch against this URL
   *  to list/add/activate. Default: `/api/v1/spaces`. */
  listUrl?: string
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

// Document-level toast notify — replaces `chrome.notify` (the audit's
// refactor rule 1). Shell's notify listener renders the toast; the pack
// doesn't need a chrome dep just to surface a message.
function notify(opts: { tone?: string; title?: string; description?: string }): void {
  if (typeof document === 'undefined') return
  const detail: Record<string, unknown> = { ...opts }
  if (opts.tone !== undefined) detail['tone'] = opts.tone
  document.dispatchEvent(new CustomEvent('sui:notify', { detail, bubbles: true }))
}

interface ListResp {
  spaces?: Array<{ id: string; name: string; path: string; label?: string }>
  activeId?: string | null
}


async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
  const resp = await fetch(url, {
    credentials: 'same-origin',
    headers: { 'content-type': 'application/json' },
    ...init,
  })
  if (!resp.ok) {
    let msg = `${resp.status} ${resp.statusText}`
    try { const t = await resp.text(); if (t) msg = t } catch { /* ignore */ }
    throw new Error(msg)
  }
  return resp.json()
}

export function setupProjectSwitcher(ctx: SwitcherCtx): SwitcherHandle {
  ensurePopoverStyles()
  const { host, open, projects, activeProjectId, popoverEl, badgeEl } = ctx

  const closePopover = (): void => open.set(false)
  const listUrl = (): string => ctx.listUrl ?? '/api/v1/spaces'

  const loadProjects = async (): Promise<void> => {
    try {
      const data = await fetchJson(listUrl()) as ListResp
      const list = (data.spaces ?? []).map((s) => ({
        id: s.id,
        name: s.name,
        path: s.path,
        ...(s.label !== undefined ? { label: s.label } : {}),
      }))
      projects.set(list)
      activeProjectId.set(data.activeId ?? null)
    } catch { /* leave previous state */ }
  }

  const switchProject = async (id: string): Promise<void> => {
    const previousActive = activeProjectId()
    // Optimistic update so the popover row + badge initial reflect the
    // pick immediately. Reverted on a non-OK response.
    activeProjectId.set(id)
    closePopover()
    try {
      await fetchJson(`${listUrl()}/${encodeURIComponent(id)}/activate`, { method: 'POST' })
      // The kernel-bus `project.switched` event drives the chrome's
      // view-remount + status-bar refresh path; chrome subscribes in
      // shell/src/main.ts. Keep the legacy DOM event so any existing
      // listeners (rail tests, devtools) continue to fire.
      dispatchBubble(host, 'ship:project:change', { id })
    } catch {
      activeProjectId.set(previousActive)
      notify({ tone: 'destructive', title: 'Switch failed' })
    }
    void loadProjects()
  }

  const addProjectFlow = async (): Promise<void> => {
    const path = window.prompt('Path to project directory:')
    if (!path) return
    try {
      await fetchJson(listUrl(), {
        method: 'POST',
        body: JSON.stringify({ path, name: '' }),
      })
      await loadProjects()
      notify({ tone: 'success', title: 'Project added.' })
    } catch (e) {
      notify({
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
    const popHeight = pop.offsetHeight || 240
    const gap = 8
    // Anchor to the badge's RIGHT edge (popover opens to the right of
    // the rail). Vertical: align to the badge's top, but flip up if it
    // would overflow the bottom. Project tile is now at the TOP of the
    // rail (was footer); the old "bottom: window.innerHeight - r.top"
    // calc put the popover above the badge — off-screen above the
    // viewport when the badge sits at the very top.
    const left = Math.min(r.right + gap, window.innerWidth - popWidth - 8)
    let top = r.top
    if (top + popHeight + 8 > window.innerHeight) {
      top = Math.max(8, window.innerHeight - popHeight - 8)
    }
    pop.style.left = `${Math.max(8, left)}px`
    pop.style.top = `${top}px`
    pop.style.bottom = 'auto'
  }

  // Open the project as its OWN native window (Tauri) or a new browser tab
  // (dev). One seam — Tauri invoke if available, else `window.open`. The
  // popover does NOT close on this branch: in the Tauri case a new Ship
  // window opens (the hub stays put); in browser-dev a new tab opens.
  // Switching the active project (the row's primary click) is unchanged.
  const openProjectAsWindow = async (id: string): Promise<void> => {
    try {
      const w = window as Window & {
        __TAURI__?: { core?: { invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> }; invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> }
        __TAURI_INTERNALS__?: { invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> }
      }
      const invoke = w.__TAURI__?.core?.invoke ?? w.__TAURI__?.invoke ?? w.__TAURI_INTERNALS__?.invoke
      if (invoke) {
        await invoke('open_space', { appId: id })
      } else {
        window.open(`/space/${encodeURIComponent(id)}`, '_blank', 'noopener,noreferrer')
      }
    } catch (err) {
      notify({
        tone: 'destructive',
        title: `Open failed: ${(err as Error).message ?? err}`,
      })
    }
  }

  const onPopoverClick = (e: Event): void => {
    const target = e.target as HTMLElement | null
    if (!target) return
    if (target.closest('[data-rail-add]')) {
      void addProjectFlow()
      return
    }
    // "Open in its own window" button — per-row affordance, beats the
    // primary click which still swaps the active project inline. Check
    // this BEFORE the row fallback so the button wins.
    const openBtn = target.closest('[data-space-open]') as HTMLElement | null
    if (openBtn?.dataset.spaceOpen) {
      e.stopPropagation()
      void openProjectAsWindow(openBtn.dataset.spaceOpen)
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
