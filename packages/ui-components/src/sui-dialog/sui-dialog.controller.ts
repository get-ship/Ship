import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-dialog.sui'
import { FocusReturn, acquireScrollLock, releaseScrollLock, focusableWithin } from '../overlay-focus'

let _seq = 0

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const uid = ++_seq
    const headerId = `sui-dialog-header-${uid}`
    const bodyId = `sui-dialog-body-${uid}`

    const dialogEl = scope.dialogEl() as HTMLDialogElement | null
    const closeBtn = scope.closeBtn() as HTMLButtonElement | null
    const sentinelStart = scope.sentinelStart() as HTMLElement | null
    const sentinelEnd = scope.sentinelEnd() as HTMLElement | null
    const headerEl = scope.headerEl() as HTMLElement | null
    const bodyEl = scope.bodyEl() as HTMLElement | null

    if (!dialogEl) return

    if (headerEl) headerEl.id = headerId
    if (bodyEl) bodyEl.id = bodyId
    dialogEl.setAttribute('aria-labelledby', headerId)
    dialogEl.setAttribute('aria-describedby', bodyId)

    let hasScrollLock = false
    const focusReturn = new FocusReturn()

    function updateRole() {
      dialogEl!.setAttribute('role', scope.roleOverride() || 'dialog')
      dialogEl!.setAttribute('aria-modal', 'true')
    }
    updateRole()

    function collectFocusables(): HTMLElement[] {
      const shadowFocusables = focusableWithin(dialogEl!).filter(
        el => !el.classList.contains('sentinel'),
      )
      const lightFocusables = focusableWithin(host)
      return [...lightFocusables, ...shadowFocusables]
    }

    function focusInitial() {
      const autofocus =
        host.querySelector<HTMLElement>('[autofocus]') ??
        dialogEl!.querySelector<HTMLElement>('[autofocus]')
      if (autofocus) { autofocus.focus(); return }
      const focusables = collectFocusables()
      const primary = focusables.find(el => el !== closeBtn) ?? focusables[0]
      if (primary) primary.focus()
      else dialogEl!.focus()
    }

    function doOpen() {
      focusReturn.save()
      try { dialogEl!.showModal() } catch { /* jsdom may not support */ }
      if (!hasScrollLock) { acquireScrollLock(); hasScrollLock = true }
      queueMicrotask(() => focusInitial())
      host.dispatchEvent(new CustomEvent('sui-open', { bubbles: true, composed: true }))
    }

    function doClose() {
      if ((dialogEl as HTMLDialogElement).open) dialogEl!.close()
    }

    const onNativeClose = () => {
      if (scope.open()) scope.open.set(false)
      if (hasScrollLock) { releaseScrollLock(); hasScrollLock = false }
      focusReturn.restore()
      host.dispatchEvent(new CustomEvent('sui-close', { bubbles: true, composed: true }))
    }

    const onBackdropClick = (e: MouseEvent) => {
      if (!scope.dismissible()) return
      if (e.target === dialogEl) scope.open.set(false)
    }

    const onDismiss = () => {
      host.dispatchEvent(new CustomEvent('sui-dismiss', { bubbles: true, composed: true }))
      scope.open.set(false)
    }

    const onSentinelStartFocus = () => {
      const items = collectFocusables()
      if (items.length) items[items.length - 1].focus()
    }
    const onSentinelEndFocus = () => {
      const items = collectFocusables()
      if (items.length) items[0].focus()
    }

    dialogEl.addEventListener('close', onNativeClose)
    dialogEl.addEventListener('click', onBackdropClick)
    closeBtn?.addEventListener('click', onDismiss)
    sentinelStart?.addEventListener('focus', onSentinelStartFocus)
    sentinelEnd?.addEventListener('focus', onSentinelEndFocus)

    const stopOpenSync = effect(() => {
      const wantOpen = scope.open()
      host.setAttribute('data-state', wantOpen ? 'open' : 'closed')
      if (wantOpen && !dialogEl.open) doOpen()
      else if (!wantOpen && dialogEl.open) doClose()
    })

    const stopRoleSync = effect(() => { updateRole() })

    ;(host as HTMLElement & { show(): void; showModal(): void; close(): void }).show = () =>
      scope.open.set(true)
    ;(host as HTMLElement & { showModal(): void }).showModal = () => scope.open.set(true)
    ;(host as HTMLElement & { close(): void }).close = () => scope.open.set(false)

    return () => {
      if (hasScrollLock) releaseScrollLock()
      dialogEl.removeEventListener('close', onNativeClose)
      dialogEl.removeEventListener('click', onBackdropClick)
      closeBtn?.removeEventListener('click', onDismiss)
      sentinelStart?.removeEventListener('focus', onSentinelStartFocus)
      sentinelEnd?.removeEventListener('focus', onSentinelEndFocus)
      stopOpenSync()
      stopRoleSync()
    }
  },
})
