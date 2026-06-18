// <sui-nav-top> controller — exposes the account chip state to the
// template. The chip reads `ship.account.state()` (the actual RPC
// surface — Connect-RPC `AccountService.Snapshot` via the typed SDK
// client) and derives a human label from the auth state. No
// hand-rolled `window.ship` global, no KernelBridge.

import { controller, signal, effect, shared, type ViewScope } from '@ship/sui-runtime'
import { getShipClient } from '@ship/sdk-client'
import type { Scope } from './sui-nav-top.sui'

interface AccountState {
  /** The raw auth state from `ship.account.state()`. The SDK exposes
   *  it as `LiveSource<AccountState>` where `AccountState.status`
   *  is `'logged_out' | 'pending' | 'logged_in'`. */
  status?: 'logged_out' | 'pending' | 'logged_in' | string
  email?: string
  handle?: string
}

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => {
    const ship = getShipClient()
    const accountState = shared(signal<string>('logged_out'))
    const accountLabel = shared(signal<string>('Signed out'))

    // Subscribe to the SDK's LiveSource and reflect it into the
    // template signals. `effect` cleans up on host unmount; the
    // LiveSource handles its own subscription.
    effect(() => {
      const src = ship.account.state()
      // `LiveSource` exposes either a `subscribe(cb)` (push) or a
      // `value` field (snapshot). The SDK uses both shapes depending
      // on the underlying transport — read the snapshot first, then
      // attach a push listener for deltas.
      const apply = (s: AccountState) => {
        const status = (s?.status as string) ?? 'logged_out'
        accountState.set(status)
        switch (status) {
          case 'logged_in':
            accountLabel.set(s.email ?? s.handle ?? 'Signed in')
            break
          case 'pending':
            accountLabel.set('Connecting...')
            break
          case 'logged_out':
          default:
            accountLabel.set('Sign in')
        }
      }
      const initial = src.value as AccountState | undefined
      if (initial) apply(initial)
      // Push subscription — best-effort; if the transport doesn't
      // expose `subscribe`, the snapshot read is enough for the
      // nav-top's needs (the chip doesn't need real-time updates).
      const sub = (src as { subscribe?: (cb: (v: unknown) => void) => () => void }).subscribe
      if (typeof sub === 'function') {
        return sub((v) => apply(v as AccountState))
      }
    })

    return { accountState, accountLabel }
  },
})
