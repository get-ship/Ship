// Settings view controller — owns the `settings` live signal directly.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so the compiler-emitted `<data live>` path (which closes over
// `ctx.ship`) sticks on `loading`. Wire the live source directly to
// `/_sui/data?expr=ship.settings.state()` instead.
//
// Per-row save state: `wireFormMutation` (in packages/sui-runtime) writes
// `scope.$mutation` for *every* form, so a single shared `$mutation`
// signal binds all save badges together — and worse, its initial
// `{ kind: 'ok' }` state makes every badge render "Saved" before the
// user has done anything. We side-step that by keeping one local
// boolean per form row, flipped to true on the form's `success`
// CustomEvent (dispatched per-form by wireFormMutation) and reset by a
// short timer. Each row's `s-save-badge` binds `class:visible` to its
// own signal instead of the shared `$mutation`.

import { controller, signal, derived, unwrapOr } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import {
  accountDisplayName,
  accountInitials,
  accountPlanLabel,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity,
} from '@ship/sdk-client'
import type { Scope } from './page.sui'

// Flip the given signal to true for `ms`, then back to false. Each call
// supersedes the previous timer so rapid saves don't stack.
function flashSaved(sig: Signal<boolean>, ms = 1600): void {
  sig.set(true)
  const w = window as unknown as Record<string, number | undefined>
  const key = `__shipSetSavedTimer_${(sig as unknown as { __id?: number }).__id ?? Math.random()}`
  const existing = w[key]
  if (typeof existing === 'number') clearTimeout(existing)
  w[key] = setTimeout(() => sig.set(false), ms) as unknown as number
}

// View-host doesn't put a `ship` SDK client in scope, so
// `wireFormMutation` can't resolve `ship.settings.update`. Mount our
// own minimal `ship.settings.update` shim that PUTs to the
// daemon-backed `/api/v1/settings` endpoint with `{ key, value }`.
// wireFormMutation always passes a FormData; pull `key` and `value`.
async function settingsUpdate(formData: FormData): Promise<{ key: string; value: string }> {
  const key = String(formData.get('key') ?? '')
  const value = String(formData.get('value') ?? '')
  if (!key) throw new Error('settings.update: missing "key" field')
  const res = await fetch('/api/v1/settings', {
    method: 'PUT',
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key, value }),
  })
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText)
    throw new Error(`settings.update failed (${res.status}): ${text}`)
  }
  return { key, value }
}

export default controller<Scope>({
  // `settings` and `account` come from the <data> elements in the template
  // (see SUI hard rule #2). Destructure them from the auto-generated scope.
  scope: ({ account }) => {
    const accountBusy = signal(false)
    const accountError = signal('')
    const accountIdentity = derived(() =>
      readAccountIdentity(unwrapOr(account, null)),
    )
    const accountName = derived(() => accountDisplayName(accountIdentity()))
    const accountInitialsSig = derived(
      () => accountInitials(accountIdentity()) || '··',
    )
    const accountPlan = derived(() => accountPlanLabel(accountIdentity()))

    async function onAccountSignIn(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true)
      accountError.set('')
      try {
        await accountSignIn()
      } catch (e) {
        accountError.set(e instanceof Error ? e.message : String(e))
      } finally {
        accountBusy.set(false)
      }
    }
    async function onAccountSignOut(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true)
      accountError.set('')
      try {
        await accountSignOut()
      } catch (e) {
        accountError.set(e instanceof Error ? e.message : String(e))
      } finally {
        accountBusy.set(false)
      }
    }

    return {
      advancedOpen: signal(false),
      savedProvider: signal(false),
      savedModel: signal(false),
      savedTerminal: signal(false),
      savedWorktree: signal(false),
      savedRegistry: signal(false),
      flashSaved,
      // Local SDK facade so `<form mutation="ship.settings.update">`
      // resolves through `resolveChain` in wireFormMutation.
      ship: { settings: { update: settingsUpdate } },

      // Account section — same live SSE source (declared in template) as
      // <sui-account-menu> so the two surfaces stay in lockstep without
      // duplicating state.
      accountBusy,
      accountError,
      accountIdentity,
      accountName,
      accountInitials: accountInitialsSig,
      accountPlan,
      onAccountSignIn,
      onAccountSignOut,
      openBilling: openAccountBilling,
    }
  },
  on: {
    // Settings → About → Run setup again. Clears the wizard's
    // completion flag and dispatches `ship:setup:rerun` so shell's
    // setup-wizard hook routes to /setup. Standalone callers (no
    // shell) get a direct location assignment as a fallback.
    rerunSetup: () => {
      try {
        localStorage.removeItem('ship-setup-completed')
        localStorage.removeItem('ship-setup-state')
      } catch { /* best-effort */ }
      window.dispatchEvent(new CustomEvent('ship:setup:rerun'))
      if (!window.location.pathname.startsWith('/setup')) {
        window.location.href = '/setup'
      }
    },
  },
})
