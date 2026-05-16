// Chrome IPC registry — listens for the typed wrapper events emitted by
// `@ship/sui-runtime`'s `chrome.*` API and applies them to shell-owned
// chrome surfaces.
//
// Per-view claims are tracked by `viewId` (stamped onto the event detail
// by the wrapper from `window.__suiCurrentViewId`). When shell2 disposes
// the previous mount it dispatches `sui:view:unmount` with the outgoing
// viewId; the registry then walks each claim map and clears whatever the
// view set so chrome doesn't leak stale state into the next view.
//
// Slot rendering responsibilities here:
//   - breadcrumb  → write into `#ship-breadcrumb-slot` (created lazily)
//   - status      → tracked-only; sui-status-bar listens to the same
//                   events for actual paint
//   - theme       → CSS custom property writes on document.body
//   - focus mode  → body.focus-mode toggle (push/pop stack)
//   - modal queue → lazy sui-dialog + per-request action buttons

import { renderBreadcrumb, ensureBreadcrumbHost } from './breadcrumb'
import { openModalRequest, type ModalRequestDetail } from './modal-queue'

interface BreadcrumbItem { label: string; href?: string }
interface StatusValue { label: string; tone?: string; icon?: string }

interface BreadcrumbSetDetail { items: BreadcrumbItem[]; viewId?: string }
interface StatusSetDetail { key: string; value: StatusValue; viewId?: string }
interface StatusClearDetail { key: string; viewId?: string }
interface ThemeSetDetail { token: string; value: string; viewId?: string }
interface ThemeClearDetail { token: string; viewId?: string }
interface ViewUnmountDetail { viewId: string }
interface ScopedDetail { viewId?: string }

interface ViewClaims {
  breadcrumb: boolean
  statusKeys: Set<string>
  themeTokens: Set<string>
  focusPushes: number
}

const SHELL_OWNER = '__shell__'
const claims = new Map<string, ViewClaims>()
const focusStack: string[] = []
let lastBreadcrumbOwner: string | null = null

function ownerOf(viewId: string | undefined): string {
  return viewId ?? SHELL_OWNER
}

function claimsFor(owner: string): ViewClaims {
  let entry = claims.get(owner)
  if (!entry) {
    entry = { breadcrumb: false, statusKeys: new Set(), themeTokens: new Set(), focusPushes: 0 }
    claims.set(owner, entry)
  }
  return entry
}

function detail<T>(e: Event): T {
  return ((e as CustomEvent).detail ?? {}) as T
}

export function initChromeRegistry(): void {
  if (typeof document === 'undefined') return

  ensureBreadcrumbHost()

  document.addEventListener('sui:breadcrumb:set', (e: Event) => {
    const d = detail<BreadcrumbSetDetail>(e)
    const owner = ownerOf(d.viewId)
    if (d.items.length === 0) {
      renderBreadcrumb([])
      const prev = lastBreadcrumbOwner ? claims.get(lastBreadcrumbOwner) : null
      if (prev) prev.breadcrumb = false
      lastBreadcrumbOwner = null
      return
    }
    if (lastBreadcrumbOwner && lastBreadcrumbOwner !== owner) {
      const prev = claims.get(lastBreadcrumbOwner)
      if (prev) prev.breadcrumb = false
    }
    renderBreadcrumb(d.items)
    claimsFor(owner).breadcrumb = true
    lastBreadcrumbOwner = owner
  })

  document.addEventListener('sui:status:set', (e: Event) => {
    const d = detail<StatusSetDetail>(e)
    if (!d.key) return
    claimsFor(ownerOf(d.viewId)).statusKeys.add(d.key)
  })

  document.addEventListener('sui:status:clear', (e: Event) => {
    const d = detail<StatusClearDetail>(e)
    if (!d.key) return
    claimsFor(ownerOf(d.viewId)).statusKeys.delete(d.key)
  })

  document.addEventListener('sui:theme:set', (e: Event) => {
    const d = detail<ThemeSetDetail>(e)
    if (!d.token) return
    document.body.style.setProperty(d.token, d.value)
    claimsFor(ownerOf(d.viewId)).themeTokens.add(d.token)
  })

  document.addEventListener('sui:theme:clear', (e: Event) => {
    const d = detail<ThemeClearDetail>(e)
    if (!d.token) return
    document.body.style.removeProperty(d.token)
    const c = claims.get(ownerOf(d.viewId))
    c?.themeTokens.delete(d.token)
  })

  document.addEventListener('sui:focus:enter', (e: Event) => {
    const owner = ownerOf(detail<ScopedDetail>(e).viewId)
    focusStack.push(owner)
    claimsFor(owner).focusPushes += 1
    applyFocusMode()
  })

  document.addEventListener('sui:focus:exit', (e: Event) => {
    const owner = ownerOf(detail<ScopedDetail>(e).viewId)
    popFocus(owner)
    applyFocusMode()
  })

  document.addEventListener('sui:modal:request', (e: Event) => {
    const d = detail<ModalRequestDetail>(e)
    if (typeof d.id !== 'number') return
    void openModalRequest(d)
  })

  document.addEventListener('sui:view:unmount', (e: Event) => {
    const { viewId } = detail<ViewUnmountDetail>(e)
    if (!viewId) return
    releaseView(viewId)
  })
}

function popFocus(owner: string): void {
  const idx = focusStack.lastIndexOf(owner)
  if (idx >= 0) focusStack.splice(idx, 1)
  const c = claims.get(owner)
  if (c && c.focusPushes > 0) c.focusPushes -= 1
}

function applyFocusMode(): void {
  document.body.classList.toggle('focus-mode', focusStack.length > 0)
}

function releaseView(viewId: string): void {
  const c = claims.get(viewId)
  if (!c) return
  if (c.breadcrumb && lastBreadcrumbOwner === viewId) {
    renderBreadcrumb([])
    lastBreadcrumbOwner = null
  }
  for (const key of c.statusKeys) {
    document.dispatchEvent(
      new CustomEvent('sui:status:clear', { detail: { key }, bubbles: true }),
    )
  }
  for (const token of c.themeTokens) {
    document.body.style.removeProperty(token)
  }
  if (c.focusPushes > 0) {
    for (let i = 0; i < c.focusPushes; i += 1) {
      const idx = focusStack.lastIndexOf(viewId)
      if (idx >= 0) focusStack.splice(idx, 1)
    }
    applyFocusMode()
  }
  claims.delete(viewId)
}
