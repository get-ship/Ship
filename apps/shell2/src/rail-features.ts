//! Rail features — project switcher, session badge, launcher/create.
//!
//! The rail component is a passive container. All feature logic (API
//! calls, DOM wiring, event dispatching) lives here so the component
//! stays under the 300-line file cap and remains purely presentational.

interface Project {
  id: string
  name: string
  path: string
  label?: string
}

let projectOpen = false

function getRail(): HTMLElement | null {
  return document.getElementById('ship-rail')
}

function getCommandBar(): (HTMLElement & {
  show(mode?: string, query?: string): void
  hide(): void
  toggle(mode?: string): void
  setMode(mode: string): void
}) | null {
  return document.getElementById('ship-command-bar') as any
}

function notifyToast(title: string, tone: string): void {
  document.dispatchEvent(new CustomEvent('sui:notify', {
    detail: { tone, title },
    bubbles: true,
  }))
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

function updateProjectDisplay(
  projects: Project[],
  activeId: string | undefined,
  badge: HTMLElement | null,
  list: HTMLElement | null,
): void {
  const cur = projects.find((p) => p.id === activeId) ?? projects[0] ?? null
  if (badge) {
    badge.textContent = cur ? (cur.label ?? cur.name).charAt(0).toUpperCase() : ''
    badge.title = cur ? `Project: ${cur.label ?? cur.name}` : 'No project'
  }
  if (list) {
    if (projects.length === 0) {
      list.innerHTML = '<div class="project-popover-empty">No projects tracked.</div>'
      return
    }
    list.innerHTML = projects.map((p) => `
      <div class="project-popover-row${p.id === activeId ? ' active' : ''}" data-project-id="${p.id}">
        <span class="project-popover-row-name">${escapeHtml(p.label ?? p.name)}</span>
        <span class="project-popover-row-path">${escapeHtml(p.path)}</span>
      </div>
    `).join('')
  }
}

async function loadProjects(): Promise<void> {
  const rail = getRail()
  if (!rail) return
  const badge = rail.querySelector('[data-rail="project-badge"]') as HTMLElement | null
  const list = rail.querySelector('[data-rail="project-list"]') as HTMLElement | null
  try {
    const resp = await fetch('/api/v1/projects', { credentials: 'same-origin' })
    if (!resp.ok) return
    const body = await resp.json() as { projects: Project[]; active?: string }
    updateProjectDisplay(body.projects, body.active, badge, list)
  } catch { /* empty */ }
}

async function switchProject(id: string): Promise<void> {
  const rail = getRail()
  if (!rail) return
  const popover = rail.querySelector('[data-rail="project-popover"]') as HTMLElement | null
  try {
    await fetch('/api/v1/projects/switch', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id }),
    })
    projectOpen = false
    popover?.setAttribute('hidden', '')
    document.dispatchEvent(new CustomEvent('ship:project:change', { detail: { id }, bubbles: true }))
  } catch { /* empty */ }
  void loadProjects()
}

async function addProject(path: string): Promise<void> {
  try {
    const resp = await fetch('/api/v1/projects', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path }),
    })
    if (!resp.ok) {
      const err = await resp.text().catch(() => '')
      notifyToast(`Add failed: ${err || resp.status}`, 'warning')
      return
    }
    void loadProjects()
    notifyToast('Project added.', 'success')
  } catch { /* empty */ }
}

async function pollSessions(): Promise<void> {
  try {
    const resp = await fetch('/api/v1/pty-sessions', { credentials: 'same-origin' })
    if (!resp.ok) return
    const body = await resp.json() as { sessions: Array<{ status: string }> }
    const count = body.sessions.filter((s) => s.status !== 'ended').length
    const badge = document.querySelector<HTMLElement>('sui-rail-item[data-view="mesh"]')
    if (badge) badge.setAttribute('badge', count > 0 ? String(count) : '')
  } catch { /* leave previous count */ }
}

export function initRailFeatures(): void {
  const rail = getRail()
  if (!rail) return

  const projectBadge = rail.querySelector('[data-rail="project-badge"]')
  const projectPopover = rail.querySelector('[data-rail="project-popover"]')
  const projectList = rail.querySelector('[data-rail="project-list"]')
  const addProjectRow = rail.querySelector('[data-rail="add-project"]')
  const launcherBtn = rail.querySelector('[data-rail="launcher"]')
  const createBtn = rail.querySelector('[data-rail="create"]')

  void loadProjects()
  void pollSessions()
  setInterval(() => void pollSessions(), 15000)

  projectBadge?.addEventListener('click', () => {
    projectOpen = !projectOpen
    if (projectPopover) {
      projectOpen ? projectPopover.removeAttribute('hidden') : projectPopover.setAttribute('hidden', '')
    }
  })

  projectList?.addEventListener('click', (e) => {
    const row = (e.target as HTMLElement).closest('[data-project-id]') as HTMLElement | null
    if (row?.dataset.projectId) void switchProject(row.dataset.projectId)
  })

  addProjectRow?.addEventListener('click', () => {
    const path = prompt('Path to project directory:')
    if (path) void addProject(path)
  })

  launcherBtn?.addEventListener('click', () => {
    getCommandBar()?.toggle('launcher')
    projectOpen = false
    projectPopover?.setAttribute('hidden', '')
  })

  createBtn?.addEventListener('click', () => {
    getCommandBar()?.show('create')
    projectOpen = false
    projectPopover?.setAttribute('hidden', '')
  })

  document.addEventListener('click', (e) => {
    if (!projectOpen) return
    const target = e.target as HTMLElement | null
    if (!target || !rail.contains(target)) {
      projectOpen = false
      projectPopover?.setAttribute('hidden', '')
    }
  }, true)
}
