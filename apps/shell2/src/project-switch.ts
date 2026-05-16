// Project-switch wiring — keeps chrome in sync with the daemon's
// active-project context.
//
// The project picker in <sui-rail> POSTs /api/v1/projects/{id}/activate
// which fires a kernel-bus `project.switched` event. This module
// subscribes to that event and:
//   1. Calls the host's `remount()` so the current view re-fetches in
//      the new scope (any project-scoped <data live> declaration
//      reruns on a fresh mount).
//   2. Pokes the host's `refreshStatus()` so the branch + cross-project
//      indicator refresh on the next tick.
//
// It also polls /api/v1/sessions/active-across-projects and renders a
// chip into <sui-status-bar>'s `end` slot when ≥1 session is running
// in a project that isn't the active one. Click the chip to switch
// directly to that project (when only one other project has runs) or
// surface the breakdown as a toast.

import type { KernelBridge } from './kernel-bridge'

interface CrossSession {
  id: string
  project_id?: string | null
  project_name?: string | null
  goal?: string | null
}

interface CrossSessionsResp {
  ok?: boolean
  data?: { sessions?: CrossSession[] }
}

const CROSS_SLOT_ID = 'ship-cross-project-indicator'

export interface ProjectSwitchHost {
  /** Status-bar host the indicator chip is appended into. */
  statusBar: HTMLElement | null
  /** Re-mount the currently-displayed view (no-op when nothing mounted). */
  remount(): void
  /** Update branch / online indicators after a switch. */
  refreshStatus(): void | Promise<void>
}

export function initProjectSwitch(
  bridge: KernelBridge,
  host: ProjectSwitchHost,
): void {
  bridge.subscribe('project.switched', (event) => {
    host.remount()
    void host.refreshStatus()
    void updateCrossProjectIndicator(host.statusBar)
    const detail = (event.payload as { project_id?: string } | null) ?? {}
    document.dispatchEvent(new CustomEvent('ship:project:switched', {
      detail,
      bubbles: true,
    }))
  })

  bridge.subscribe('session.', () => {
    void updateCrossProjectIndicator(host.statusBar)
  })
  setInterval(() => void updateCrossProjectIndicator(host.statusBar), 30_000)
  void updateCrossProjectIndicator(host.statusBar)
}

async function updateCrossProjectIndicator(statusBar: HTMLElement | null): Promise<void> {
  if (!statusBar) return
  let activeId: string | null = null
  try {
    const resp = await fetch('/api/v1/projects/active', { credentials: 'same-origin' })
    if (resp.ok) {
      const body = (await resp.json()) as { active?: { id?: string } | null }
      activeId = body.active?.id ?? null
    }
  } catch { /* every running session counts as "elsewhere" when we can't read active */ }

  let sessions: CrossSession[] = []
  try {
    const resp = await fetch('/api/v1/sessions/active-across-projects', {
      credentials: 'same-origin',
    })
    if (!resp.ok) return
    const body = (await resp.json()) as CrossSessionsResp
    sessions = body.data?.sessions ?? []
  } catch { return }

  const others = sessions.filter((s) => s.project_id && s.project_id !== activeId)
  renderCrossIndicator(statusBar, others)
}

function renderCrossIndicator(statusBar: HTMLElement, others: CrossSession[]): void {
  const existing = document.getElementById(CROSS_SLOT_ID) as HTMLElement | null
  if (others.length === 0) {
    existing?.remove()
    return
  }
  const byProject = new Map<string, { name: string; count: number }>()
  for (const s of others) {
    const id = s.project_id ?? '_'
    const entry = byProject.get(id) ?? { name: s.project_name ?? id, count: 0 }
    entry.count++
    byProject.set(id, entry)
  }
  const total = others.length
  const breakdown = Array.from(byProject.values())
    .map(({ name, count }) => `${name}: ${count}`)
    .join(' • ')

  const chip = existing ?? document.createElement('button')
  chip.id = CROSS_SLOT_ID
  chip.setAttribute('slot', 'end')
  chip.setAttribute('type', 'button')
  chip.setAttribute('data-cross-project-count', String(total))
  chip.setAttribute('aria-label', `${total} agent sessions running in other projects`)
  chip.title = `Running elsewhere: ${breakdown}`
  chip.style.cssText = [
    'appearance:none',
    'background:color-mix(in oklch, var(--primary) 14%, transparent)',
    'color:var(--primary)',
    'border:1px solid color-mix(in oklch, var(--primary) 30%, transparent)',
    'border-radius:var(--radius-sm, 4px)',
    'padding:1px 6px',
    'font:inherit',
    'font-size:10.5px',
    'line-height:1.4',
    'cursor:pointer',
    'flex-shrink:0',
  ].join(';')
  chip.textContent = `${total} in other projects`
  if (!existing) {
    chip.addEventListener('click', () => onCrossIndicatorClick(byProject))
    statusBar.appendChild(chip)
  }
}

function onCrossIndicatorClick(byProject: Map<string, { name: string; count: number }>): void {
  // One other project: switch directly. Multiple: surface a breakdown
  // toast so the user can decide. A richer popover-with-list is
  // follow-up work; minimum bar is "user can see and act on the fact
  // that things run elsewhere."
  if (byProject.size === 1) {
    const id = byProject.keys().next().value as string
    void fetch(`/api/v1/projects/${encodeURIComponent(id)}/activate`, {
      method: 'POST',
      credentials: 'same-origin',
    })
    return
  }
  const summary = Array.from(byProject.values())
    .map(({ name, count }) => `${name} (${count})`)
    .join(', ')
  document.dispatchEvent(new CustomEvent('sui:notify', {
    detail: {
      tone: 'info',
      title: 'Agents running in other projects',
      description: summary,
      duration: 6000,
    },
    bubbles: true,
  }))
}
