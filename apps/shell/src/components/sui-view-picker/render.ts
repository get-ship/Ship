// Imperative body rendering for <sui-view-picker>. Data layer +
// drag state machine live in data.ts.

import { type ApiView, type Project, humanize, ZONE_PINNED_END, ZONE_ALL } from './data'

const KNOWN_APP_NAMES = new Set([
  'dashboard', 'pipelines', 'agents', 'skills', 'git', 'collections',
  'canvas', 'settings', 'chat', 'events', 'specs', 'theme',
  'theme-studio', 'setup', 'overview', 'mesh', 'ship-ui-studio',
  'artifacts', 'workspaces',
])

export interface RenderArgs {
  pinned: ApiView[]
  others: ApiView[]
  cursor: number
  pins: string[]
  activeView: string
  projects: Project[]
  activeProjectId: string | null
  dragOverName: string | null
  dragOverZone: number | null
  draggingName: string | null
  onNavigate: (name: string) => void
  onPickProject: (id: string) => void
  onTogglePin: (e: Event, name: string) => void
  onDragStart: (e: DragEvent, name: string, fromPinned: boolean) => void
  onDragOverRow: (e: DragEvent, name: string) => void
  onDropRow: (e: DragEvent, name: string, intoPinned: boolean) => void
  onDragOverZone: (e: DragEvent, zoneId: number) => void
  onDropZone: (e: DragEvent, zoneId: number) => void
  onDragEnd: () => void
  onRowEnter: (idx: number) => void
}

export function renderBody(el: HTMLElement, args: RenderArgs): void {
  el.replaceChildren()
  // Projects section at top — unified picker (Task #54). Shown when
  // ≥1 project is tracked; lets the user switch project without
  // leaving the picker.
  const projectCount = args.projects.length
  if (projectCount > 0) {
    el.appendChild(buildProjectsSection(args))
  } else {
    el.appendChild(buildProjectsEmpty())
  }
  const total = args.pinned.length + args.others.length
  if (total === 0) {
    const empty = document.createElement('div')
    empty.className = 'vp-empty'
    empty.textContent = 'No views match.'
    el.appendChild(empty)
    return
  }
  el.appendChild(buildSection('Pinned to rail', 'section-pinned', args.pinned, projectCount, true, args))
  el.appendChild(buildSection('All views', 'section-all', args.others, projectCount + args.pinned.length, false, args))
}

function buildProjectsEmpty(): HTMLElement {
  const section = document.createElement('div')
  section.setAttribute('part', 'section-projects')
  const header = document.createElement('div')
  header.className = 'vp-section-h'
  const icon = document.createElement('sui-icon')
  icon.setAttribute('name', 'folder-tree')
  icon.setAttribute('size', 'sm')
  header.appendChild(icon)
  header.appendChild(Object.assign(document.createElement('span'), { textContent: 'Projects' }))
  section.appendChild(header)
  const empty = document.createElement('div')
  empty.className = 'vp-empty'
  empty.style.padding = '8px 12px'
  empty.style.textAlign = 'left'
  empty.textContent = 'No projects — run `ship init` or open a workspace.'
  section.appendChild(empty)
  return section
}

function buildProjectsSection(args: RenderArgs): HTMLElement {
  const section = document.createElement('div')
  section.setAttribute('part', 'section-projects')

  const header = document.createElement('div')
  header.className = 'vp-section-h'
  const icon = document.createElement('sui-icon')
  icon.setAttribute('name', 'folder-tree')
  icon.setAttribute('size', 'sm')
  header.appendChild(icon)
  header.appendChild(Object.assign(document.createElement('span'), { textContent: 'Projects' }))
  header.appendChild(Object.assign(document.createElement('span'), {
    className: 'vp-hint', textContent: `${args.projects.length} tracked`,
  }))
  section.appendChild(header)

  args.projects.forEach((p, idx) => {
    const isActive = p.id === args.activeProjectId
    const isCursor = idx === args.cursor
    const row = document.createElement('div')
    row.className = 'vp-row vp-row-project'
      + (isActive ? ' is-active' : '')
      + (isCursor ? ' is-cursor' : '')
    row.setAttribute('role', 'option')
    row.setAttribute('data-project-id', p.id)
    if (isActive) row.setAttribute('aria-current', 'true')

    const swatch = document.createElement('span')
    swatch.className = 'vp-project-swatch'
    if (isActive) swatch.classList.add('is-active')

    const text = document.createElement('span')
    text.className = 'vp-row-text'
    const label = document.createElement('span')
    label.className = 'vp-row-label'
    label.textContent = p.label || p.name
    const sub = document.createElement('span')
    sub.className = 'vp-row-sub'
    sub.textContent = p.path
    text.append(label, sub)

    row.append(swatch, text)
    row.addEventListener('click', () => args.onPickProject(p.id))
    row.addEventListener('mouseenter', () => args.onRowEnter(idx))
    section.appendChild(row)
  })
  return section
}

function buildSection(
  label: string, partName: string, rows: ApiView[],
  indexBase: number, isPinned: boolean, args: RenderArgs,
): HTMLElement {
  const section = document.createElement('div')
  section.setAttribute('part', partName)

  const header = document.createElement('div')
  header.className = 'vp-section-h'
  if (isPinned) {
    const pin = document.createElement('sui-icon')
    pin.setAttribute('name', 'pin'); pin.setAttribute('size', 'sm')
    header.appendChild(pin)
  }
  header.appendChild(Object.assign(document.createElement('span'), { textContent: label }))
  if (isPinned) {
    header.appendChild(Object.assign(document.createElement('span'), {
      className: 'vp-hint', textContent: 'drag to reorder',
    }))
  }
  section.appendChild(header)

  if (rows.length === 0) {
    const empty = document.createElement('div')
    empty.className = 'vp-empty'
    empty.style.padding = '8px 12px'
    empty.style.textAlign = 'left'
    empty.textContent = isPinned ? 'Nothing pinned yet.' : 'No more views.'
    section.appendChild(empty)
  }
  rows.forEach((v, i) => section.appendChild(buildRow(v, indexBase + i, isPinned, args)))

  // End-of-section drop zone — pin-to-end when dragging from all-views
  // into the pinned section; unpin when dragging from pinned into all.
  const zoneId = isPinned ? ZONE_PINNED_END : ZONE_ALL
  const zone = document.createElement('div')
  zone.className = 'vp-drop-zone' + (args.dragOverZone === zoneId ? ' is-drag-over' : '')
  zone.setAttribute('data-zone', String(zoneId))
  zone.addEventListener('dragover', (e) => args.onDragOverZone(e, zoneId))
  zone.addEventListener('drop', (e) => args.onDropZone(e, zoneId))
  section.appendChild(zone)
  return section
}

function buildRow(v: ApiView, idx: number, isPinned: boolean, args: RenderArgs): HTMLElement {
  const label = v.rail?.label ?? humanize(v.name)
  const fallbackIcon = v.rail?.icon ?? 'app-window'
  const known = KNOWN_APP_NAMES.has(v.name)
  const isActive = v.name === args.activeView
  const isCursor = idx === args.cursor
  const isDragOver = args.dragOverName === v.name
  const isDragging = args.draggingName === v.name

  const row = document.createElement('div')
  row.className = [
    'vp-row',
    isCursor ? 'is-cursor' : '',
    isActive ? 'is-active' : '',
    isDragOver ? 'is-drag-over' : '',
    isDragging ? 'is-dragging' : '',
  ].filter(Boolean).join(' ')
  row.setAttribute('role', 'option')
  row.setAttribute('data-view', v.name)
  row.setAttribute('data-section', isPinned ? 'pinned' : 'all')
  if (isActive) row.setAttribute('aria-current', 'true')
  row.setAttribute('draggable', 'true')
  row.setAttribute('tabindex', '-1')

  const icon = document.createElement(known ? 'sui-app-icon' : 'sui-icon')
  if (known) {
    icon.setAttribute('name', v.name); icon.setAttribute('variant', 'glyph'); icon.setAttribute('size', 'md')
  } else {
    icon.setAttribute('name', fallbackIcon); icon.setAttribute('size', 'md')
  }

  const text = document.createElement('span')
  text.className = 'vp-row-text'
  const lbl = document.createElement('span')
  lbl.className = 'vp-row-label'; lbl.textContent = label
  const sub = document.createElement('span')
  sub.className = 'vp-row-sub'
  sub.textContent = isPinned ? 'workspace' : (v.source || v.type || '')
  text.append(lbl, sub)
  row.append(icon, text)

  row.addEventListener('click', () => args.onNavigate(v.name))
  row.addEventListener('mouseenter', () => args.onRowEnter(idx))
  row.addEventListener('dragstart', (e) => args.onDragStart(e, v.name, isPinned))
  row.addEventListener('dragover', (e) => args.onDragOverRow(e, v.name))
  row.addEventListener('drop', (e) => args.onDropRow(e, v.name, isPinned))
  row.addEventListener('dragend', () => args.onDragEnd())
  return row
}
