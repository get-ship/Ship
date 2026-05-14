import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-auth-dialog.sui'

type AuthMode = 'sign-in' | 'sign-up'
type DialogEl = HTMLElement & { show(): void; showModal(): void; close(): void }

export default controller<Scope>({
  scope: (state, host) => {
    const s = state as unknown as {
      open: Signal<boolean>
      mode: Signal<AuthMode>
      loading: Signal<boolean>
    }

    const dialogRef = ref<DialogEl>(null)
    const formRef = ref<HTMLFormElement>(null)
    const providersSlot = ref<HTMLSlotElement>(null)
    const hasProviders = signal<boolean>(false)

    const defaultTitle = derived<string>(() =>
      s.mode() === 'sign-up' ? 'Create your account' : 'Welcome back',
    )
    const submitLabel = derived<string>(() => {
      if (s.loading()) {
        return s.mode() === 'sign-up' ? 'Creating account…' : 'Signing in…'
      }
      return s.mode() === 'sign-up' ? 'Create account' : 'Sign in'
    })
    const otherMode = derived<AuthMode>(() =>
      s.mode() === 'sign-up' ? 'sign-in' : 'sign-up',
    )
    const switchLabel = derived<string>(() =>
      otherMode() === 'sign-up' ? 'Create an account' : 'Sign in instead',
    )
    const switchPrompt = derived<string>(() =>
      s.mode() === 'sign-up' ? 'Already have an account?' : 'New here?',
    )
    const passwordAutocomplete = derived<string>(() =>
      s.mode() === 'sign-up' ? 'new-password' : 'current-password',
    )

    function setMode(next: AuthMode): void {
      if (next === s.mode()) return
      s.mode.set(next)
      host.dispatchEvent(
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
      host.dispatchEvent(
        new CustomEvent('sui-auth-submit', {
          detail: { mode: s.mode(), email, password },
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
      if (s.open()) s.open.set(false)
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
    const scope = state as unknown as Scope & {
      open: Signal<boolean>
      dialogRef: Signal<DialogEl | null>
      providersSlot: Signal<HTMLSlotElement | null>
      hasProviders: Signal<boolean>
    }

    const recomputeProviders = () => {
      const slot = scope.providersSlot()
      if (!slot) {
        scope.hasProviders.set(false)
        return
      }
      const filled = slot.assignedNodes({ flatten: true }).some((n) => {
        if (n.nodeType === Node.TEXT_NODE) {
          return (n.textContent || '').trim().length > 0
        }
        return n.nodeType === Node.ELEMENT_NODE
      })
      scope.hasProviders.set(filled)
    }

    const stopSlotBind = effect(() => {
      const slot = scope.providersSlot()
      if (!slot) return
      slot.addEventListener('slotchange', recomputeProviders)
      recomputeProviders()
      return () => slot.removeEventListener('slotchange', recomputeProviders)
    })

    const stopOpenSync = effect(() => {
      const wantOpen = scope.open()
      const dlg = scope.dialogRef()
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
    target.show = () => scope.open.set(true)
    target.showModal = () => scope.open.set(true)
    target.close = () => scope.open.set(false)

    return () => {
      stopSlotBind()
      stopOpenSync()
    }
  },
})
