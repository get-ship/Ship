/**
 * Focus utilities shared by overlay components (dialog/popover/menu/alert-dialog).
 *
 *   - `focusableWithin(root)` — returns focusable descendants in DOM order
 *   - `trapFocus(root, event)` — wraps Tab/Shift+Tab within `root`
 *   - `FocusReturn` — saves the previously-focused element on enter, restores
 *     on leave.
 *
 * Intentionally no side effects at import time — all state is per-instance.
 */

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'area[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  'iframe',
  'object',
  'embed',
  'summary',
  'audio[controls]',
  'video[controls]',
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex^="-"])',
].join(',')

export function focusableWithin(root: ParentNode): HTMLElement[] {
  const nodes = Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR))
  return nodes.filter((el) => {
    if (el.hasAttribute('disabled')) return false
    if (el.getAttribute('aria-hidden') === 'true') return false
    if (el instanceof HTMLInputElement && el.type === 'hidden') return false
    return true
  })
}

export function trapFocus(root: ParentNode, e: KeyboardEvent): void {
  if (e.key !== 'Tab') return
  const items = focusableWithin(root)
  if (items.length === 0) {
    e.preventDefault()
    return
  }
  const first = items[0]
  const last = items[items.length - 1]
  const active = document.activeElement as HTMLElement | null
  if (e.shiftKey) {
    if (active === first || !root.contains(active!)) {
      e.preventDefault()
      last.focus()
    }
  } else {
    if (active === last) {
      e.preventDefault()
      first.focus()
    }
  }
}

/**
 * Remembers the active element on `save()` and restores focus on `restore()`.
 * Safe to call `restore()` multiple times (no-op after first).
 */
export class FocusReturn {
  private saved: HTMLElement | null = null
  save(): void {
    this.saved = (document.activeElement as HTMLElement) ?? null
  }
  restore(): void {
    const target = this.saved
    this.saved = null
    if (target && typeof target.focus === 'function') target.focus()
  }
}
