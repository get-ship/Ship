import { controller, effect, signal, ref, bindNativeEvent } from '@ship/sui-runtime'
import type { Scope } from './sui-view-picker.sui'
import {
  type ApiView, type Project,
  fetchPins, fetchProject, fetchViews,
  makeDrag, partition, savePins,
} from './data'
import { renderBody } from './render'

interface PopoverEl extends HTMLElement {
  show?: () => void; hide?: () => void; toggle?: () => void
}

export default controller<Scope>({
  scope: () => ({
    popoverEl: ref<PopoverEl>(),
    rootEl: ref<HTMLElement>(),
    projectNameEl: ref<HTMLElement>(),
    projectPathEl: ref<HTMLElement>(),
    filterInput: ref<HTMLInputElement>(),
    countEl: ref<HTMLElement>(),
    pagerEl: ref<HTMLElement>(),
    bodyEl: ref<HTMLElement>(),
    installLink: ref<HTMLAnchorElement>(),
  }),

  mount(state, host) {
    const scope = state as unknown as Scope

    const views = signal<ApiView[]>([])
    const pins = signal<string[]>([])
    const project = signal<Project | null>(null)
    const filter = signal('')
    const cursor = signal(0)
    const activeView = signal<string>(routeToViewName(location.pathname))
    const dragState = { name: null, overName: null, overZone: null, fromPinned: false }
    let filterTimer: ReturnType<typeof setTimeout> | null = null
    let loaded = false

    const sUrl = (): string => scope.settingsUrl()
    const sKey = (): string => scope.pinsKey()

    async function load(): Promise<void> {
      const [p, pr, vs] = await Promise.all([
        fetchPins(sUrl(), sKey()), fetchProject(scope.projectsUrl()), fetchViews(scope.viewsUrl()),
      ])
      pins.set(p); project.set(pr); views.set(vs); loaded = true
    }

    function part(): { pinned: ApiView[]; others: ApiView[] } {
      return partition({ views: views(), pins: pins(), filter: filter() })
    }

    const navigate = (name: string): void => {
      host.emit!('sui:view-picker:navigate', { name })
      const path = '/' + (name === 'dashboard' ? '' : name)
      try { history.pushState({}, '', path) } catch { /* SSR-safe */ }
      document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
      ;(window as unknown as { ship?: { navigate?: (p: string) => void } }).ship?.navigate?.(path)
      scope.open.set(false)
    }

    const togglePin = async (name: string): Promise<void> => {
      const cur = pins()
      const next = cur.includes(name) ? cur.filter((n) => n !== name) : [...cur, name]
      pins.set(next)
      await savePins(sUrl(), sKey(), next)
      host.emit!('sui:view-picker:pin-changed', { pins: next })
    }

    const drag = makeDrag({
      state: dragState,
      pins: () => pins(),
      setPins: (next) => pins.set(next),
      persist: (next) => savePins(sUrl(), sKey(), next),
      repaint: () => paintBody(),
      emit: (next) => host.emit!('sui:view-picker:pin-changed', { pins: next }),
    })

    function paintBody(): void {
      const bodyEl = scope.bodyEl()
      if (!bodyEl) return
      const { pinned, others } = part()
      renderBody(bodyEl, {
        pinned, others, cursor: cursor(), pins: pins(), activeView: activeView(),
        dragOverName: dragState.overName, dragOverZone: dragState.overZone,
        draggingName: dragState.name,
        onNavigate: navigate,
        onTogglePin: (e, name) => { e.stopPropagation(); void togglePin(name) },
        onDragStart: drag.onDragStart, onDragOverRow: drag.onDragOverRow,
        onDropRow: drag.onDropRow, onDragOverZone: drag.onDragOverZone,
        onDropZone: drag.onDropZone, onDragEnd: drag.onDragEnd,
        onRowEnter: (idx) => cursor.set(idx),
      })
    }

    const stopHeader = effect(() => {
      const p = project()
      const n = scope.projectNameEl(); const pt = scope.projectPathEl()
      if (n) n.textContent = p?.label ?? p?.name ?? 'Ship'
      if (pt) pt.textContent = p?.path ?? ''
    })
    const stopInstall = effect(() => {
      const a = scope.installLink()
      if (a) a.setAttribute('href', scope.installHref())
    })
    const stopBody = effect(() => {
      const { pinned, others } = part()
      const total = pinned.length + others.length
      const c = scope.countEl(); const pg = scope.pagerEl()
      if (c) c.textContent = String(total)
      if (pg) pg.textContent = `${total} view${total === 1 ? '' : 's'}`
      paintBody()
    })

    // ── filter input ────────────────────────────────────────────
    const fInput = scope.filterInput()
    if (fInput) {
      bindNativeEvent(fInput, 'input', () => {
        if (filterTimer) clearTimeout(filterTimer)
        filterTimer = setTimeout(() => { filter.set(fInput.value); cursor.set(0) }, 60)
      })
      bindNativeEvent(fInput, 'keydown', (e) => {
        const k = (e as KeyboardEvent).key
        if (k === 'ArrowDown' || k === 'ArrowUp' || k === 'Enter' || k === 'Escape') onKey(e as KeyboardEvent)
      })
    }

    // ── install link ────────────────────────────────────────────
    const iLink = scope.installLink()
    if (iLink) bindNativeEvent(iLink, 'click', (e) => {
      e.preventDefault()
      host.emit!('sui:view-picker:install', { href: scope.installHref() })
      navigate(scope.installHref().replace(/^\//, ''))
    })

    // ── open/close sync (host.open ↔ composed sui-popover) ──────
    let wasOpen = false
    const stopOpen = effect(() => {
      const o = scope.open()
      const pop = scope.popoverEl()
      if (pop) {
        if (o && !pop.hasAttribute('open')) pop.setAttribute('open', '')
        else if (!o && pop.hasAttribute('open')) pop.removeAttribute('open')
      }
      if (o && !wasOpen) {
        cursor.set(0); filter.set('')
        if (fInput) fInput.value = ''
        if (!loaded) void load()
        requestAnimationFrame(() => fInput?.focus())
        host.emit!('sui:view-picker:open', null)
      } else if (!o && wasOpen) {
        host.emit!('sui:view-picker:close', null)
      }
      wasOpen = o
    })

    const pop = scope.popoverEl()
    if (pop) bindNativeEvent(pop, 'sui-toggle', (e) => {
      const open = !!(e as CustomEvent).detail?.open
      if (open !== scope.open()) scope.open.set(open)
    })

    host.expose!({
      open: () => scope.open.set(true),
      close: () => scope.open.set(false),
      toggle: () => scope.open.set(!scope.open()),
      refresh: () => load(),
    })

    // ── keyboard ────────────────────────────────────────────────
    function onKey(e: KeyboardEvent): void {
      if (!scope.open()) return
      if (e.key === 'Escape') { e.preventDefault(); scope.open.set(false); return }
      if (e.key === '/' && document.activeElement !== fInput) {
        e.preventDefault(); fInput?.focus(); return
      }
      const { pinned, others } = part()
      const visible = [...pinned, ...others]
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        cursor.set(Math.min(visible.length - 1, cursor() + 1))
        requestAnimationFrame(() => {
          scope.bodyEl()?.querySelector('.vp-row.is-cursor')?.scrollIntoView({ block: 'nearest' })
        })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        cursor.set(Math.max(0, cursor() - 1))
      } else if (e.key === 'Enter') {
        const row = visible[cursor()]
        if (row) { e.preventDefault(); navigate(row.name) }
      }
    }
    bindNativeEvent(document, 'keydown', onKey as EventListener)

    const onNav = (): void => activeView.set(routeToViewName(location.pathname))
    bindNativeEvent(document, 'sui:navigate', onNav)
    bindNativeEvent(window, 'popstate', onNav)

    return () => {
      stopHeader(); stopInstall(); stopBody(); stopOpen()
      if (filterTimer) clearTimeout(filterTimer)
    }
  },
})

function routeToViewName(path: string): string {
  const seg = path.split('/').filter(Boolean)[0]
  return seg ?? 'dashboard'
}
