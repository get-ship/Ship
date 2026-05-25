// <sui-settings> — CLI + studio configuration. First-party CHROME (apps/shell);
// shell renders it at /settings. Relocated from views/settings; the two <data>
// sources are now controller-side live() over @ship/sdk-client.
import { controller, signal, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { Signal, LiveSignal, AccountState, ViewScope } from '@ship/sui-runtime'
import {
  accountDisplayName,
  accountInitials,
  accountPlanLabel,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity,
} from '@ship/sdk-client'
import { ops, ship } from '../../ops'
import type { Scope } from './sui-settings.sui'

function flashSaved(sig: Signal<boolean>, ms = 1600): void {
  sig.set(true)
  const w = window as unknown as Record<string, number | undefined>
  const key = `__shipSetSavedTimer_${(sig as unknown as { __id?: number }).__id ?? Math.random()}`
  const existing = w[key]
  if (typeof existing === 'number') clearTimeout(existing)
  w[key] = setTimeout(() => sig.set(false), ms) as unknown as number
}

async function settingsUpdate(formData: FormData): Promise<{ key: string; value: string }> {
  const key = String(formData.get('key') ?? '')
  const value = String(formData.get('value') ?? '')
  if (!key) throw new Error('settings.update: missing "key" field')
  await ops.settings.set({ key, valueJson: JSON.stringify(value) })
  return { key, value }
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    // Chrome has no ctx.ship — wire the live sources directly (was <data>).
    const settings = live(() => ship.settings.state())
    const account = live(() => ship.account.state())

    const accountBusy = signal(false)
    const accountError = signal('')
    const accountIdentity = derived(() => readAccountIdentity(unwrapOr(account as LiveSignal<AccountState | null>, null)))
    const accountName = derived(() => accountDisplayName(accountIdentity()))
    const accountInitialsSig = derived(() => accountInitials(accountIdentity()) || '··')
    const accountPlan = derived(() => accountPlanLabel(accountIdentity()))

    async function onAccountSignIn(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true); accountError.set('')
      try { await accountSignIn() }
      catch (e) { accountError.set(e instanceof Error ? e.message : String(e)) }
      finally { accountBusy.set(false) }
    }
    async function onAccountSignOut(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true); accountError.set('')
      try { await accountSignOut() }
      catch (e) { accountError.set(e instanceof Error ? e.message : String(e)) }
      finally { accountBusy.set(false) }
    }

    return {
      settings, account,
      advancedOpen: signal(false),
      savedProvider: signal(false),
      savedModel: signal(false),
      savedTerminal: signal(false),
      savedWorktree: signal(false),
      savedRegistry: signal(false),
      savedHarnessDefault: signal(false),
      savedHarnessModel: signal(false),
      savedAutoResume: signal(false),
      savedWatcherDebounce: signal(false),
      savedGitLogLimit: signal(false),
      savedHooksMaxDepth: signal(false),
      flashSaved,
      ship: { settings: { update: settingsUpdate } },
      accountBusy, accountError, accountIdentity, accountName,
      accountInitials: accountInitialsSig, accountPlan,
      onAccountSignIn, onAccountSignOut,
      openBilling: openAccountBilling,
    }
  },
  on: {
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
