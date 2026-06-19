// <sui-view-picker> — hub-side chrome copy of the shell's view-picker
// controller (apps/shell/src/components/sui-view-picker/sui-view-
// picker.controller.ts).
//
// The shell is being deprecated as the hub composes from packs. The
// hub owns the chrome, so this controller lives under apps/hub/src/
// components/sui-view-picker/ rather than a shared pack. The surface
// is identical to the shell version: popover with project header +
// filter + drag-pinned/all-views sections + install link, all wired
// to the daemon via the same Connect ops the shell called.
//
// Two host-API substitutions, both audit-driven:
//
// 1. The shell called `chrome.navigate(path)` (lines 93 + 245 of the
//    shell source). The hub does not expose a `chrome` object — the
//    bundler entry wires the runtime's RouterPort directly. Import
//    `navigate` from `@ship/deck/route` (the same import the shell
//    uses — see the existing import at the top of the shell source).
//
// 2. The shell called `chrome.notify({ tone, title, description,
//    duration })` on a failed project switch (lines 69-72 of the shell
//    source). The hub's notify port is wired to a document-level
//    `sui:notify` event listener by the bundler entry; dispatching
//    the same event here is the audit-mandated equivalent. Same payload
//    shape, same consumers — no new abstraction.
import { controller, effect, signal, ref, bindNativeEvent } from '@ship/sui-runtime'
import { navigate } from '@ship/deck/route'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-view-picker.sui'
import {
  type ApiView, type Project,
  fetchPins, fetchProject, fetchProjects, fetchViews,
  makeDrag, partition, savePins, switchProject,
} from './data'
import { renderBody } from './render'

interface PopoverEl extends HTMLElement {
  show?: () => void; hide?: () => void; toggle?: () => void
}

/** Hub notify payload. The shape is owned by the bundler entry's
 *  document-level `sui:notify` listener; the audit locked it to these
 *  four fields. Kept inline (no shared type) because the consumer is
 *  a single `CustomEvent` detail — promoting it to a named type would
 *  freeze a payload before the framework settles. */
interface HubNotifyDetail {
  tone: 'info' | 'success' | 'warning' | 'error'
  title: string
  description?: string
  duration?: number
}

/** Dispatch a `sui:notify` event on `document`. The bundler entry
 *  installs a document-level listener that surfaces these as toasts. */
function notify(detail: HubNotifyDetail): void {
  document.dispatchEvent(new CustomEvent<HubNotifyDetail>('sui:notify', { detail }))
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({
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

    const views = signal<ApiView[]>([])
    const pins = signal<string[]>([])
    const project = signal<Project | null>(null)
    const projects = signal<Project[]>([])
    const activeProjectId = signal<string | null>(null)
    const filter = signal('')
    const cursor = signal(0)
    const activeView = signal<string>(routeToViewName(location.pathname))
    const dragState = { name: null, overName: null, overZone: null, fromPinned: false }
    let filterTimer: ReturnType<typeof setTimeout> | null = null
    let loaded = false

    const sUrl = (): string => state.$signals.settingsUrl()
    const sKey = (): string => state.$signals.pinsKey()

    async function load(): Promise<void> {
      const [p, pr, vs, projList] = await Promise.all([
        fetchPins(sUrl(), sKey()),
        fetchProject(state.$signals.projectsUrl()),
        fetchViews(state.$signals.viewsUrl()),
        fetchProjects(state.$signals.projectsUrl()),
      ])
      pins.set(p); project.set(pr); views.set(vs)
      projects.set(projList.projects)
      activeProjectId.set(projList.activeId)
      loaded = true
    }

    async function onPickProject(id: string): Promise<void> {
      // Already-active click is a no-op (popover closes via state.$signals.open).
      if (id === activeProjectId()) { state.$signals.open.set(false); return }
      // Optimistic: highlight + close. Refresh state on success.
      const prev = activeProjectId()
      activeProjectId.set(id)
      state.$signals.open.set(false)
      const ok = await switchProject(id)
      if (!ok) {
        activeProjectId.set(prev)
        // Audit refactor #1: `chrome.notify` → `sui:notify` document event.
        // The hub's bundler entry installs a document-level listener that
        // surfaces these as toasts — same payload shape, same consumer.
        notify({
          tone: 'error',
          title: 'Project switch failed',
          description: 'The daemon rejected the switch. Try again or check shipd logs.',
          duration: 5000,
        })
        return
      }
      host.emit!('sui:view-picker:project-changed', { id })
      // Refresh the picker's own state on next open.
      loaded = false
    }

    function part(): { pinned: ApiView[]; others: ApiView[] } {
      return partition({ views: views(), pins: pins(), filter: filter() })
    }

    const goto = (name: string): void => {
      host.emit!('sui:view-picker:navigate', { name })
      const path = '/' + (name === 'dashboard' ? '' : name)
      // Audit refactor #2: `chrome.navigate(path)` → `navigate(path)`.
      // One affordance: the unified router (history + match + mount + the
      // route-change notification). Replaces the old pushState +
      // sui:navigate + window.ship.navigate hand-roll. See ADR
      // unified-deck-router.
      navigate(path)
      state.$signals.open.set(false)
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
      const bodyEl = state.$signals.bodyEl()
      if (!bodyEl) return
      const { pinned, others } = part()
      renderBody(bodyEl, {
        pinned, others, cursor: cursor(), pins: pins(), activeView: activeView(),
        projects: projects(), activeProjectId: activeProjectId(),
        dragOverName: dragState.overName, dragOverZone: dragState.overZone,
        draggingName: dragState.name,
        onNavigate: goto,
        onPickProject: (id) => { void onPickProject(id) },
        onTogglePin: (e, name) => { e.stopPropagation(); void togglePin(name) },
        onDragStart: drag.onDragStart, onDragOverRow: drag.onDragOverRow,
        onDropRow: drag.onDropRow, onDragOverZone: drag.onDragOverZone,
        onDropZone: drag.onDropZone, onDragEnd: drag.onDragEnd,
        onRowEnter: (idx) => cursor.set(idx),
      })
    }

    const stopHeader = effect(() => {
      const p = project()
      const n = state.$signals.projectNameEl(); const pt = state.$signals.projectPathEl()
      if (n) n.textContent = p?.label ?? p?.name ?? 'Ship'
      if (pt) pt.textContent = p?.path ?? ''
    })
    const stopInstall = effect(() => {
      const a = state.$signals.installLink()
      if (a) a.setAttribute('href', state.$signals.installHref())
    })
    const stopBody = effect(() => {
      const { pinned, others } = part()
      const total = pinned.length + others.length
      const c = state.$signals.countEl(); const pg = state.$signals.pagerEl()
      if (c) c.textContent = String(total)
      if (pg) pg.textContent = `${total} view${total === 1 ? '' : 's'}`
      paintBody()
    })

    // ── filter input ────────────────────────────────────────────
    const fInput = state.$signals.filterInput()
    if (fInput) {
      bindNativeEvent(fInput, 'input', () => {
        clearTimeout(filterTimer)
        filterTimer = setTimeout(() => { filter.set(fInput.value); cursor.set(0) }, 60)
      })
      bindNativeEvent(fInput, 'keydown', (e) => {
        const k = (e as KeyboardEvent).key
        if (k === 'ArrowDown' || k === 'ArrowUp' || k === 'Enter' || k === 'Escape') onKey(e as KeyboardEvent)
      })
    }

    // ── install link ────────────────────────────────────────────
    const iLink = state.$signals.installLink()
    if (iLink) bindNativeEvent(iLink, 'click', (e) => {
      e.preventDefault()
      host.emit!('sui:view-picker:install', { href: state.$signals.installHref() })
      goto(state.$signals.installHref().replace(/^\//, ''))
    })

    // ── open/close sync (host.open ↔ composed sui-popover) ──────
    let wasOpen = false
    const stopOpen = effect(() => {
      const o = state.$signals.open()
      const pop = state.$signals.popoverEl()
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

    // Drag-survives-light-dismiss (task #65). Native `popover="auto"`
    // dismisses on pointerup outside the popover bounds, which happens
    // mid-drag when the user releases over a drop target outside the
    // popover panel. Track drag state at the popover root and veto the
    // close event if it fires while a drag is in flight; re-open via
    // the exposed show() method on the next frame.
    let isDragging = false
    const popRoot = state.$signals.rootEl()
    if (popRoot) {
      bindNativeEvent(popRoot, 'dragstart', () => { isDragging = true })
      bindNativeEvent(popRoot, 'dragend', () => {
        // Defer the flag-clear so a sui-toggle firing in the same task
        // (which is what happens when the popover light-dismisses on the
        // drag's pointerup-outside) still sees isDragging === true.
        setTimeout(() => { isDragging = false }, 0)
      })
    }

    const pop = state.$signals.popoverEl()
    if (pop) bindNativeEvent(pop, 'sui-toggle', (e) => {
      const open = !!(e as CustomEvent).detail?.open
      if (!open && isDragging) {
        // Popover light-dismissed mid-drag — re-open + suppress the
        // state.$signals.open mutation so the rest of the view doesn't tear down.
        requestAnimationFrame(() => { (pop as unknown as { show?: () => void }).show?.() })
        return
      }
      if (open !== state.$signals.open()) state.$signals.open.set(open)
    })

    // Trigger click → toggle open. The composed sui-popover's
    // popovertarget wiring relies on slot.assignedElements() which
    // doesn't follow double-slotted triggers (rail → view-picker →
    // popover trigger slot), so the rail's button click never fires
    // the native popover toggle. Forward the click manually.
    bindNativeEvent(host, 'click', (e) => {
      const path = (e as Event).composedPath()
      const onTrigger = path.some((n) => {
        const el = n as HTMLElement | null
        return el?.getAttribute?.('slot') === 'trigger'
      })
      if (!onTrigger) return
      state.$signals.open.set(!state.$signals.open())
    })

    host.expose!({
      open: () => state.$signals.open.set(true),
      close: () => state.$signals.open.set(false),
      toggle: () => state.$signals.open.set(!state.$signals.open()),
      refresh: () => load(),
    })

    // ── keyboard ────────────────────────────────────────────────
    function onKey(e: KeyboardEvent): void {
      if (!state.$signals.open()) return
      if (e.key === 'Escape') { e.preventDefault(); state.$signals.open.set(false); return }
      if (e.key === '/' && document.activeElement !== fInput) {
        e.preventDefault(); fInput?.focus(); return
      }
      const { pinned, others } = part()
      const projRows = projects().map((p) => ({ kind: 'project' as const, id: p.id }))
      const viewRows = [...pinned, ...others].map((v) => ({ kind: 'view' as const, name: v.name }))
      const visible: Array<{ kind: 'project'; id: string } | { kind: 'view'; name: string }> = [
        ...projRows, ...viewRows,
      ]
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        cursor.set(Math.min(visible.length - 1, cursor() + 1))
        requestAnimationFrame(() => {
          state.$signals.bodyEl()?.querySelector('.vp-row.is-cursor')?.scrollIntoView({ block: 'nearest' })
        })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        cursor.set(Math.max(0, cursor() - 1))
      } else if (e.key === 'Enter') {
        const row = visible[cursor()]
        if (!row) return
        e.preventDefault()
        if (row.kind === 'project') void onPickProject(row.id)
        else goto(row.name)
      }
    }
    bindNativeEvent(document, 'keydown', onKey as EventListener)

    const onNav = (): void => activeView.set(routeToViewName(location.pathname))
    bindNativeEvent(document, 'sui:navigate', onNav)
    bindNativeEvent(window, 'popstate', onNav)

    return () => {
      stopHeader(); stopInstall(); stopBody(); stopOpen()
      clearTimeout(filterTimer)
    }
  },
})

function routeToViewName(path: string): string {
  const seg = path.split('/').filter(Boolean)[0]
  return seg ?? 'dashboard'
}
