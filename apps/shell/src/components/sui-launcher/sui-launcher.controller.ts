import { chrome, controller, effect, signal, bindNativeEvent } from '@ship/sui-runtime'
import type { Scope } from './sui-launcher.sui'
import { type ApiView, type Project, humanize, renderHead, renderFoot, renderBody } from './render'
import { ops } from '../../ops'

const SETTING_KEY = 'shell.rail.pinned'
const DEFAULT_PINS = ['dashboard', 'pipelines', 'agents', 'skills', 'git']

async function fetchPins(): Promise<string[]> {
  try {
    const body = JSON.parse((await ops.settings.state({})).json) as {
      data?: Record<string, unknown>
    }
    const raw = body.data?.[SETTING_KEY]
    if (Array.isArray(raw) && raw.every((v) => typeof v === 'string')) return raw as string[]
    return DEFAULT_PINS
  } catch { return DEFAULT_PINS }
}

async function savePins(pins: string[]): Promise<void> {
  try {
    await ops.settings.set({ key: SETTING_KEY, valueJson: JSON.stringify(pins) })
  } catch { /* best-effort */ }
}

async function fetchViews(): Promise<ApiView[]> {
  try {
    const body = JSON.parse((await ops.discovery.list({})).json) as { views?: ApiView[] }
    return body.views ?? []
  } catch { return [] }
}

async function fetchProject(): Promise<Project | null> {
  try {
    const body = JSON.parse((await ops.project.listProjects({})).json) as {
      projects: Project[]
      active?: string
    }
    return body.projects.find((p) => p.id === body.active) ?? body.projects[0] ?? null
  } catch { return null }
}

function matchView(v: ApiView, filterVal: string): boolean {
  const f = filterVal.trim().toLowerCase()
  if (!f) return true
  const label = (v.rail?.label ?? humanize(v.name)).toLowerCase()
  return label.includes(f) || v.source.toLowerCase().includes(f)
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const views = signal<ApiView[]>([])
    const pins = signal<string[]>([])
    const project = signal<Project | null>(null)
    const filter = signal('')
    const cursor = signal(0)
    let dragName: string | null = null
    let dragOver: string | null = null
    let filterTimer: ReturnType<typeof setTimeout> | null = null

    const headEl = scope.headEl() as HTMLElement | null
    const bodyEl = scope.bodyEl() as HTMLElement | null
    const footEl = scope.footEl() as HTMLElement | null
    const countEl = scope.countEl() as HTMLElement | null
    const filterInput = scope.filterInput() as HTMLInputElement | null
    const maskEl = scope.maskEl() as HTMLElement | null

    // ── foot is static — render once ─────────────────────────
    if (footEl) renderFoot(footEl)

    // ── reactive head render ──────────────────────────────────
    const stopHead = effect(() => {
      if (headEl) renderHead(headEl, project())
    })

    // ── reactive body + count render ──────────────────────────
    const stopBody = effect(() => {
      const allViews = views()
      const allPins = pins()
      const f = filter()
      const cur = cursor()
      const pinned = allPins
        .map((n) => allViews.find((v) => v.name === n))
        .filter((v): v is ApiView => !!v && matchView(v, f))
      const pinSet = new Set(allPins)
      const others = allViews.filter((v) => !pinSet.has(v.name) && matchView(v, f))
      const total = pinned.length + others.length
      if (countEl) countEl.textContent = String(total)
      if (bodyEl) {
        renderBody(bodyEl, pinned, others, cur, allPins,
          (name) => navigate(name),
          (e, name) => void togglePin(e, name),
          (e, name) => onDragStart(e, name),
          (e, name) => onDragOver(e, name),
          (e, name) => void onDrop(e, name),
          () => onDragEnd(),
          dragOver, dragName,
        )
      }
    })

    // ── open/close effect ─────────────────────────────────────
    let wasOpen = false
    const stopOpen = effect(() => {
      const isOpen = scope.open()
      if (isOpen && !wasOpen) {
        cursor.set(0)
        filter.set('')
        void load()
      } else if (!isOpen && wasOpen) {
        filter.set('')
        if (filterInput) filterInput.value = ''
      }
      wasOpen = isOpen
    })

    // ── mask click ────────────────────────────────────────────
    if (maskEl) bindNativeEvent(maskEl, 'click', () => doClose())

    // ── filter input ──────────────────────────────────────────
    if (filterInput) bindNativeEvent(filterInput, 'input', () => {
      if (filterTimer) clearTimeout(filterTimer)
      filterTimer = setTimeout(() => {
        filter.set(filterInput.value)
        cursor.set(0)
      }, 80)
    })
    if (filterInput) bindNativeEvent(filterInput, 'keydown', (e) => {
      if (['ArrowDown', 'ArrowUp', 'Enter', 'Escape'].includes((e as KeyboardEvent).key)) onGlobalKey(e as KeyboardEvent)
    })

    // ── global key handler ────────────────────────────────────
    function onGlobalKey(e: KeyboardEvent): void {
      if (!scope.open()) return
      if (e.key === 'Escape') { e.preventDefault(); doClose(); return }
      if (e.key === '/' && document.activeElement !== filterInput) {
        e.preventDefault()
        filterInput?.focus()
        return
      }
      const allViews = views()
      const allPins = pins()
      const f = filter()
      const pinned = allPins.map((n) => allViews.find((v) => v.name === n)).filter((v): v is ApiView => !!v && matchView(v, f))
      const pinSet = new Set(allPins)
      const others = allViews.filter((v) => !pinSet.has(v.name) && matchView(v, f))
      const visible = [...pinned, ...others]
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        cursor.set(Math.min(visible.length - 1, cursor() + 1))
        scrollCursor()
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        cursor.set(Math.max(0, cursor() - 1))
        scrollCursor()
      } else if (e.key === 'Enter') {
        const row = visible[cursor()]
        if (!row) return
        e.preventDefault()
        if (e.metaKey || e.ctrlKey) {
          host.emit!('sui:launcher:open-pane', { name: row.name })
        }
        navigate(row.name)
      }
    }
    bindNativeEvent(document, 'keydown', onGlobalKey as EventListener)

    function scrollCursor(): void {
      requestAnimationFrame(() => {
        bodyEl?.querySelector('.launcher-row.is-cursor')?.scrollIntoView({ block: 'nearest' })
      })
    }

    // ── data loading ──────────────────────────────────────────
    async function load(): Promise<void> {
      const [loadedPins, loadedProject] = await Promise.all([
        fetchPins(),
        fetchProject(),
        fetchViews().then((v) => views.set(v)),
      ])
      pins.set(loadedPins)
      project.set(loadedProject)
      requestAnimationFrame(() => filterInput?.focus())
    }

    // ── actions ───────────────────────────────────────────────
    function navigate(name: string): void {
      const path = '/' + (name === 'dashboard' ? '' : name)
      chrome.navigate(path)
      doClose()
    }

    async function togglePin(e: Event, name: string): Promise<void> {
      e.stopPropagation()
      const cur = pins()
      const next = cur.includes(name) ? cur.filter((n) => n !== name) : [...cur, name]
      pins.set(next)
      await savePins(next)
      host.emit!('sui:rail:pins-changed', { pins: next })
    }

    function doClose(): void {
      scope.open.set(false)
    }

    // ── drag ──────────────────────────────────────────────────
    function onDragStart(e: DragEvent, name: string): void {
      dragName = name
      e.dataTransfer?.setData('text/plain', name)
      if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move'
    }
    function onDragOver(e: DragEvent, name: string): void {
      if (!dragName) return
      e.preventDefault()
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
      dragOver = name
    }
    async function onDrop(e: DragEvent, target: string): Promise<void> {
      e.preventDefault()
      const src = dragName; dragName = null; dragOver = null
      if (!src || src === target) return
      const cur = pins()
      if (!cur.includes(src) || !cur.includes(target)) return
      const without = cur.filter((n) => n !== src)
      const idx = without.indexOf(target)
      const next = [...without.slice(0, idx), src, ...without.slice(idx)]
      pins.set(next)
      await savePins(next)
      host.emit!('sui:rail:pins-changed', { pins: next })
    }
    function onDragEnd(): void { dragName = null; dragOver = null }

    // ── external open/close events ────────────────────────────
    const onExtOpen = () => scope.open.set(true)
    const onExtClose = () => doClose()
    bindNativeEvent(document, 'sui:launcher:open', onExtOpen)
    bindNativeEvent(document, 'sui:launcher:close', onExtClose)

    return () => {
      stopHead(); stopBody(); stopOpen()
      if (filterTimer) clearTimeout(filterTimer)
    }
  },
})
