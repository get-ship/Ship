// Imperative rendering helpers for <sui-launcher>.
// Separated to keep sui-launcher.ts under the 300-line file cap.

export interface ApiView {
  name: string
  type: string
  source: string
  rail?: { label: string; icon: string }
}

export interface Project {
  id: string
  name: string
  path: string
  label?: string
}

const KNOWN_APP_NAMES = new Set([
  'dashboard', 'pipelines', 'agents', 'skills', 'git', 'collections',
  'canvas', 'settings', 'chat', 'events', 'specs', 'theme',
  'theme-studio', 'setup', 'overview', 'mesh',
])

export function humanize(id: string): string {
  return id
    .replace(/-/g, ' ')
    .split(' ')
    .map((w) => (w ? w[0]!.toUpperCase() + w.slice(1) : w))
    .join(' ')
}

export function renderHead(el: HTMLElement, project: Project | null): void {
  el.innerHTML = ''
  const icon = document.createElement('sui-icon')
  icon.setAttribute('name', 'settings')
  icon.setAttribute('size', 'md')
  icon.setAttribute('aria-hidden', 'true')
  const title = document.createElement('span')
  title.className = 'launcher-title'
  title.style.cssText = 'font-weight:600;font-size:14px;flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;'
  title.textContent = project?.label ?? project?.name ?? 'Ship'
  const ws = document.createElement('span')
  ws.className = 'launcher-ws'
  ws.style.cssText = 'font-size:11px;color:var(--muted-foreground,#888);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:160px;'
  ws.textContent = project?.path ?? ''
  el.append(icon, title, ws)
}

export function renderFoot(el: HTMLElement): void {
  el.innerHTML = ''
  const add = document.createElement('a')
  add.className = 'launcher-add'
  add.href = '#'
  add.style.cssText = 'display:inline-flex;align-items:center;gap:4px;color:var(--muted-foreground,#888);text-decoration:none;font-size:12px;'
  add.addEventListener('click', (e) => e.preventDefault())
  const addIcon = document.createElement('sui-icon')
  addIcon.setAttribute('name', 'plus')
  addIcon.setAttribute('size', 'sm')
  add.appendChild(addIcon)
  add.appendChild(Object.assign(document.createElement('span'), { textContent: 'Install view' }))
  const spacer = Object.assign(document.createElement('span'), { style: 'flex:1' })
  el.append(add, spacer)
  for (const k of ['⌘', '⇧', 'P']) {
    const kbd = document.createElement('sui-kbd')
    kbd.textContent = k
    el.appendChild(kbd)
  }
}

export function renderBody(
  el: HTMLElement,
  pinned: ApiView[],
  others: ApiView[],
  cursor: number,
  pins: string[],
  onNavigate: (name: string) => void,
  onTogglePin: (e: Event, name: string) => void,
  onDragStart: (e: DragEvent, name: string) => void,
  onDragOver: (e: DragEvent, name: string) => void,
  onDrop: (e: DragEvent, name: string) => void,
  onDragEnd: () => void,
  dragOverName: string | null,
  draggingName: string | null,
): void {
  el.innerHTML = ''
  const total = pinned.length + others.length
  if (total === 0) {
    const empty = document.createElement('div')
    empty.style.cssText = 'padding:24px;text-align:center;color:var(--muted-foreground,#888);font-size:13px;'
    empty.textContent = 'No views match.'
    el.appendChild(empty)
    return
  }
  if (pinned.length > 0) {
    el.appendChild(buildSection('Pinned to rail', pinned, 0, true, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName))
  }
  if (others.length > 0) {
    el.appendChild(buildSection('All views', others, pinned.length, false, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName))
  }
}

function buildSection(
  label: string,
  rows: ApiView[],
  indexBase: number,
  isPinned: boolean,
  cursor: number,
  pins: string[],
  onNavigate: (name: string) => void,
  onTogglePin: (e: Event, name: string) => void,
  onDragStart: (e: DragEvent, name: string) => void,
  onDragOver: (e: DragEvent, name: string) => void,
  onDrop: (e: DragEvent, name: string) => void,
  onDragEnd: () => void,
  dragOverName: string | null,
  draggingName: string | null,
): HTMLElement {
  const section = document.createElement('div')
  section.style.cssText = 'padding-bottom:4px;'

  const header = document.createElement('div')
  header.style.cssText = 'display:flex;align-items:center;gap:4px;padding:8px 12px 4px;font-size:11px;font-weight:600;color:var(--muted-foreground,#888);text-transform:uppercase;letter-spacing:.04em;'
  if (isPinned) {
    const pinIcon = document.createElement('sui-icon')
    pinIcon.setAttribute('name', 'pin')
    pinIcon.setAttribute('size', 'sm')
    header.appendChild(pinIcon)
  }
  header.appendChild(Object.assign(document.createElement('span'), { textContent: label }))
  const spacer = Object.assign(document.createElement('span'), { style: 'flex:1' })
  header.appendChild(spacer)
  if (isPinned) {
    header.appendChild(
      Object.assign(document.createElement('span'), {
        textContent: 'drag to reorder',
        style: 'font-size:10px;font-weight:400;color:var(--muted-foreground,#888);',
      }),
    )
  }
  section.appendChild(header)

  rows.forEach((v, i) => {
    section.appendChild(
      buildRow(v, indexBase + i, isPinned, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName),
    )
  })
  return section
}

function buildRow(
  v: ApiView,
  idx: number,
  isPinned: boolean,
  cursor: number,
  pins: string[],
  onNavigate: (name: string) => void,
  onTogglePin: (e: Event, name: string) => void,
  onDragStart: (e: DragEvent, name: string) => void,
  onDragOver: (e: DragEvent, name: string) => void,
  onDrop: (e: DragEvent, name: string) => void,
  onDragEnd: () => void,
  dragOverName: string | null,
  draggingName: string | null,
): HTMLElement {
  const label = v.rail?.label ?? humanize(v.name)
  const fallbackIcon = v.rail?.icon ?? 'app-window'
  const known = KNOWN_APP_NAMES.has(v.name)
  const pinned = pins.includes(v.name)
  const isCursor = idx === cursor
  const isDragOver = dragOverName === v.name
  const isDragging = draggingName === v.name

  const row = document.createElement('div')
  row.className = ['launcher-row', isCursor ? 'is-cursor' : '', isDragOver ? 'is-drag-over' : '', isDragging ? 'is-dragging' : ''].filter(Boolean).join(' ')
  row.setAttribute('role', 'button')
  row.setAttribute('tabindex', '-1')
  if (isPinned) row.setAttribute('draggable', 'true')
  row.style.cssText = [
    'display:flex;align-items:center;gap:var(--space-2,8px);padding:6px 12px;cursor:pointer;',
    isCursor ? 'background:var(--accent,color-mix(in oklch,var(--foreground)8%,transparent));' : '',
    isDragOver ? 'outline:2px solid var(--primary,#3b82f6);outline-offset:-2px;' : '',
    isDragging ? 'opacity:0.5;' : '',
  ].join('')

  const iconEl = document.createElement(known ? 'sui-icon' : 'sui-icon')
  iconEl.setAttribute('name', known ? fallbackIcon : fallbackIcon)
  iconEl.setAttribute('size', 'md')

  const nameWrap = document.createElement('span')
  nameWrap.style.cssText = 'flex:1;min-width:0;display:flex;flex-direction:column;gap:1px;'
  const labelEl = document.createElement('span')
  labelEl.style.cssText = 'font-size:13px;color:var(--foreground,#111);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;'
  labelEl.textContent = label
  const pathEl = document.createElement('span')
  pathEl.style.cssText = 'font-size:11px;color:var(--muted-foreground,#888);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;'
  pathEl.textContent = v.source
  nameWrap.append(labelEl, pathEl)

  const pinBtn = document.createElement('button')
  pinBtn.className = 'launcher-pin' + (pinned ? ' is-pinned' : '')
  pinBtn.type = 'button'
  pinBtn.setAttribute('aria-label', pinned ? 'Unpin from rail' : 'Pin to rail')
  pinBtn.title = pinned ? 'Unpin from rail' : 'Pin to rail'
  pinBtn.style.cssText = 'appearance:none;background:transparent;border:none;padding:2px;color:var(--muted-foreground,#888);cursor:pointer;display:inline-flex;align-items:center;opacity:' + (pinned ? '1' : '0') + ';flex-shrink:0;'
  const pinIcon = document.createElement('sui-icon')
  pinIcon.setAttribute('name', 'pin')
  pinIcon.setAttribute('size', 'sm')
  pinBtn.appendChild(pinIcon)

  row.append(iconEl, nameWrap, pinBtn)

  row.addEventListener('click', () => onNavigate(v.name))
  row.addEventListener('mouseenter', () => {
    // cursor update via event is handled in controller
  })
  pinBtn.addEventListener('click', (e: MouseEvent) => {
    e.stopPropagation()
    onTogglePin(e, v.name)
  })
  if (isPinned) {
    row.addEventListener('dragstart', (e: DragEvent) => onDragStart(e, v.name))
    row.addEventListener('dragover', (e: DragEvent) => onDragOver(e, v.name))
    row.addEventListener('drop', (e: DragEvent) => onDrop(e, v.name))
    row.addEventListener('dragend', () => onDragEnd())
  }

  // Show pin button on row hover
  row.addEventListener('mouseenter', () => { pinBtn.style.opacity = '1' })
  row.addEventListener('mouseleave', () => { if (!pinned) pinBtn.style.opacity = '0' })

  return row
}
