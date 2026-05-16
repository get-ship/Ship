// First-run setup wizard hook for shell.
//
// On boot, check localStorage for setup completion. If incomplete AND
// the user is landing on a path other than `/setup` (or one of the
// always-allowed support paths), navigate to `/setup` so the wizard
// view (`views/setup/`) takes over the viewport.
//
// The wizard itself writes `ship-setup-completed=1` when the user
// finishes or skips, after which we never redirect again. To force
// the wizard back, clear that key (Settings → About → "Run setup
// again" is wired to this).

const COMPLETED_KEY = 'ship-setup-completed'
const SETUP_PATH = '/setup'

// Paths the wizard never redirects away from — devtools, theme studio,
// settings (so users who skipped can still get back in), and the
// wizard itself.
const ALLOWED_WHEN_PENDING = new Set<string>(['/setup', '/settings', '/ship-ui-studio'])

function setupCompleted(): boolean {
  try {
    return localStorage.getItem(COMPLETED_KEY) === '1'
  } catch {
    // No storage → don't trap the user in a redirect loop.
    return true
  }
}

function isAllowed(pathname: string): boolean {
  if (pathname === SETUP_PATH) return true
  for (const allowed of ALLOWED_WHEN_PENDING) {
    if (pathname === allowed || pathname.startsWith(allowed + '/')) return true
  }
  return false
}

/**
 * Force the wizard back on the user's next nav. Called from Settings →
 * About → "Run setup again" via window.dispatchEvent.
 */
export function resetSetupWizard(): void {
  try {
    localStorage.removeItem(COMPLETED_KEY)
    localStorage.removeItem('ship-setup-state')
  } catch { /* best-effort */ }
}

/**
 * Wire the first-run check. Call once at shell boot, before mountRoute.
 * If setup hasn't been completed and the user isn't on an allowed
 * support path, this rewrites the URL to `/setup` so the next mount
 * lands on the wizard.
 *
 * Returns `true` when a redirect was applied (caller can use this as a
 * "skip the original mountRoute" signal if needed).
 */
export function ensureSetupOrRedirect(): boolean {
  if (setupCompleted()) return false
  if (isAllowed(location.pathname)) return false
  history.replaceState({}, '', SETUP_PATH)
  return true
}

// Settings re-entry: a `ship:setup:rerun` window event clears completion
// and navigates to /setup. Settings → About wires its button to this.
if (typeof window !== 'undefined') {
  window.addEventListener('ship:setup:rerun', () => {
    resetSetupWizard()
    history.pushState({}, '', SETUP_PATH)
    document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: SETUP_PATH } }))
  })
}
