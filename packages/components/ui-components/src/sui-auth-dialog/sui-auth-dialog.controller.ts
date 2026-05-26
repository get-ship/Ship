import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-auth-dialog.sui'

type AuthMode = 'sign-in' | 'sign-up'
type DialogEl = HTMLElement & { show(): void; showModal(): void; close(): void }

export default controller({
  scope: (ctx: ViewScope<Scope>, host) => {
    const dialogRef = ref<DialogEl>(null)
    const formRef = ref<HTMLFormElement>(null)
    const providersSlot = ref<HTMLSlotElement>(null)
    const hasProviders = signal<boolean>(false)

    const defaultTitle = derived<string>(() =>
      ctx.mode() === 'sign-up' ? 'Create your account' : 'Welcome back',
    )
    const submitLabel = derived<string>(() => {
      if (ctx.loading()) {
        return ctx.mode() === 'sign-up' ? 'Creating account…' : 'Signing in…'
      }
      return ctx.mode() === 'sign-up' ? 'Create account' : 'Sign in'
    })
    const otherMode = derived<AuthMode>(() =>
      ctx.mode() === 'sign-up' ? 'sign-in' : 'sign-up',
    )
    const switchLabel = derived<string>(() =>
      otherMode() === 'sign-up' ? 'Create an account' : 'Sign in instead',
    )
    const switchPrompt = derived<string>(() =>
      ctx.mode() === 'sign-up' ? 'Already have an account?' : 'New here?',
    )
    const passwordAutocomplete = derived<string>(() =>
      ctx.mode() === 'sign-up' ? 'new-password' : 'current-password',
    )

    function setMode(next: AuthMode): void {
      if (next === ctx.mode()) return
      ctx.mode.set(next)
      host?.dispatchEvent(
        new CustomEvent('sui-mode-change', {
          detail: { mode: next },
          bubbles: true,
          composed: true,
        }),
      )
    }

    function onTabChange(e: Event): void {
      const detail = (e as CustomEvent<{ name?: string }>).detail
      if (!detail || typeof detail.name !== 'string') return
      const next: AuthMode = detail.name === 'sign-up' ? 'sign-up' : 'sign-in'
      setMode(next)
    }

    function onSubmit(e: Event): void {
      e.preventDefault()
      const form = (e.currentTarget as HTMLFormElement | null) ?? formRef()
      if (!form) return
      const email =
        (form.elements.namedItem('email') as HTMLInputElement | null)?.value ?? ''
      const password =
        (form.elements.namedItem('password') as HTMLInputElement | null)?.value ?? ''
      host?.dispatchEvent(
        new CustomEvent('sui-auth-submit', {
          detail: { mode: ctx.mode(), email, password },
          bubbles: true,
          composed: true,
        }),
      )
    }

    function onSwitchClick(): void {
      setMode(otherMode())
    }

    function onDialogClose(): void {
      // The inner <sui-dialog> dispatches sui-close with composed:true,
      // so the event bubbles to the host naturally — no re-dispatch
      // here. Just sync our own open signal.
      if (ctx.open()) ctx.open.set(false)
    }

    return {
      dialogRef,
      formRef,
      providersSlot,
      hasProviders,
      defaultTitle,
      submitLabel,
      otherMode,
      switchLabel,
      switchPrompt,
      passwordAutocomplete,
      onTabChange,
      onSubmit,
      onSwitchClick,
      onDialogClose,
    }
  },

  mount(state, host) {
    const recomputeProviders = () => {
      const slot = state.providersSlot as HTMLSlotElement | null
      if (!slot) {
        state.$signals.hasProviders.set(false)
        return
      }
      const filled = slot.assignedNodes({ flatten: true }).some((n: Node) => {
        if (n.nodeType === Node.TEXT_NODE) {
          return (n.textContent || '').trim().length > 0
        }
        return n.nodeType === Node.ELEMENT_NODE
      })
      state.$signals.hasProviders.set(filled)
    }

    const stopSlotBind = effect(() => {
      const slot = state.providersSlot as HTMLSlotElement | null
      if (!slot) return
      slot.addEventListener('slotchange', recomputeProviders)
      recomputeProviders()
      return () => slot.removeEventListener('slotchange', recomputeProviders)
    })

    const stopOpenSync = effect(() => {
      const wantOpen = state.open as boolean
      const dlg = state.dialogRef as DialogEl | null
      if (!dlg) return
      if (wantOpen && typeof dlg.showModal === 'function') {
        if (!dlg.hasAttribute('open')) dlg.showModal()
      } else if (!wantOpen && typeof dlg.close === 'function') {
        if (dlg.hasAttribute('open')) dlg.close()
      }
    })

    // Imperative parity helpers — mirror ship-dialog / ship-auth-dialog.
    type AuthEl = HTMLElement & {
      show: () => void
      showModal: () => void
      close: () => void
    }
    const target = host as AuthEl
    target.show = () => state.$signals.open.set(true)
    target.showModal = () => state.$signals.open.set(true)
    target.close = () => state.$signals.open.set(false)

    return () => {
      stopSlotBind()
      stopOpenSync()
    }
  },
})
