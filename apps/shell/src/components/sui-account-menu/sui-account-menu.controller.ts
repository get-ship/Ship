import {
  controller,
  signal,
  derived,
  effect,
  ref,
  unwrapOr,
  chrome,
} from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import {
  accountDisplayName as deriveDisplayName,
  accountInitials as deriveInitials,
  accountPlanLabel as derivePlanLabel,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity,
  type AccountIdentity,
} from '@ship/sdk-client'
import type { Scope } from './sui-account-menu.sui'

type PopoverEl = HTMLElement & {
  showPopover?: () => void
  hidePopover?: () => void
}

const EMPTY: AccountIdentity = {
  status: 'logged_out',
  user_id: null,
  me: null,
  state_id: null,
}

export default controller({
  // `account` is auto-created from `<data name="account" source="ship.account.state()" live />`
  // in the template — see SUI hard rule #2 (subscriptions in <data>).
  // The controller derives identity + UI state from it.
  scope: ({ account }: ViewScope<Scope>) => {
    const panelOpen = signal(false)
    const busy = signal(false)
    const errorMessage = signal('')

    // Squash the live envelope into a flat identity record so templates
    // don't need to re-do the loading/error/ok dance per access.
    const identity = derived<AccountIdentity>(() => readAccountIdentity(unwrapOr(account, null)))

    // Resolved-identity label only. The live-source load states (loading /
    // error) are render states owned by the template via {#while-loading} /
    // {#while-error} — the controller must not re-derive them from
    // `account().kind` (sui-controller-no-load-state).
    const triggerLabel = derived(() => {
      const id = identity()
      if (id.status === 'logged_in') return deriveDisplayName(id) || 'Account'
      if (id.status === 'pending') return 'Completing…'
      return 'Sign in'
    })
    const triggerAria = derived(() => {
      const label = triggerLabel()
      return label === 'Sign in' ? 'Sign in to Ship Cloud' : `Account: ${label}`
    })
    const initials = derived(() => deriveInitials(identity()) || '··')
    const displayName = derived(() => deriveDisplayName(identity()))
    const planLabel = derived(() => derivePlanLabel(identity()))

    const internals: { host: HTMLElement | null; panel: PopoverEl | null } = {
      host: null,
      panel: null,
    }

    const triggerBtn = ref<HTMLButtonElement>(null)
    const panelEl = ref<HTMLDivElement>(null)

    function togglePanel(): void {
      const panel = internals.panel
      if (!panel) return
      if (panelOpen()) panel.hidePopover?.()
      else panel.showPopover?.()
    }

    function closePanel(): void {
      internals.panel?.hidePopover?.()
    }

    function openSettings(): void {
      closePanel()
      chrome.navigate('/settings#account')
    }

    function openBillingAction(): void {
      closePanel()
      openAccountBilling()
    }

    async function onSignIn(): Promise<void> {
      if (busy()) return
      busy.set(true)
      errorMessage.set('')
      try {
        await accountSignIn()
        // Leave panel open with the pending state visible; the live signal
        // will flip to logged_in once the gateway-side flow completes.
      } catch (e) {
        errorMessage.set(e instanceof Error ? e.message : String(e))
      } finally {
        busy.set(false)
      }
    }

    async function onSignOut(): Promise<void> {
      if (busy()) return
      busy.set(true)
      errorMessage.set('')
      try {
        await accountSignOut()
        closePanel()
      } catch (e) {
        errorMessage.set(e instanceof Error ? e.message : String(e))
      } finally {
        busy.set(false)
      }
    }

    return {
      identity,
      panelOpen,
      busy,
      errorMessage,
      triggerLabel,
      triggerAria,
      initials,
      displayName,
      planLabel,
      togglePanel,
      openSettings,
      openBillingAction,
      onSignIn,
      onSignOut,
      triggerBtn,
      panelEl,
      _internals: internals,
    }
  },

  mount(state, host) {
    void EMPTY

    state._internals.host = host

    // Sync panel ref into internals + listen for native open/close so
    // panelOpen mirrors the popover state.
    const stopPanelSync = effect(() => {
      const panel = state.panelEl
      state._internals.panel = panel
      if (!panel) return
      const onToggle = (e: Event) => {
        const newState = (e as ToggleEvent).newState
        const open = newState === 'open'
        if (state.panelOpen !== open) state.panelOpen = open
        if (open) anchorPanel()
      }
      panel.addEventListener('toggle', onToggle)
      return () => panel.removeEventListener('toggle', onToggle)
    })

    function anchorPanel(): void {
      const panel = state.panelEl
      const trigger = state.triggerBtn
      if (!panel || !trigger) return
      const r = trigger.getBoundingClientRect()
      panel.style.top = `${r.bottom + 4}px`
      panel.style.left = `${Math.max(8, r.right - panel.offsetWidth)}px`
    }

    // Close on outside click — native popover="auto" handles light-dismiss,
    // but escape-key also closes via the platform default.
    function onPanelKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault()
        state.panelEl?.hidePopover?.()
      }
    }
    const stopKeyBind = effect(() => {
      const panel = state.panelEl
      if (!panel) return
      panel.addEventListener('keydown', onPanelKeydown)
      return () => panel.removeEventListener('keydown', onPanelKeydown)
    })

    host.dispatchEvent(
      new CustomEvent('sui:account:menu-ready', { bubbles: true, composed: true }),
    )

    return () => {
      stopPanelSync()
      stopKeyBind()
    }
  },
})
