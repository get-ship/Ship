<controller>
import { controller, derived, effect, ref, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './sui-auth-dialog.sail'

type AuthMode = 'sign-in' | 'sign-up'
type DialogEl = HTMLElement & { show(): void; showModal(): void; close(): void }

export default controller((ctx: ViewScope<Scope>, host) => {

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
        new CustomEvent('sail-mode-change', {
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
        new CustomEvent('sail-auth-submit', {
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
})
</controller>

<!--
  <sui-auth-dialog> — SUI port of <ship-auth-dialog>.

  Domain wrapper for authentication modals. Composes <dialog> as the
  modal chrome, <tabs> + <tab> to switch between sign-in /
  sign-up, and <button> for the primary submit. Bare <input>s are
  retained for the email + password fields to match the lit version's
  field treatment; <input> is value-bound, while these are a one-shot
  read on form submit.

  Prop / event / slot parity vs <ship-auth-dialog>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  open (bool, false)           open (bool, false,       ✓ reflects
                                  reflect)
  mode (sign-in|sign-up)       mode (enum, 'sign-in',   ✓ reflects
                                  reflect)
  loading (bool, false)        loading (bool, false,    ✓ reflects
                                  reflect)
  backdrop (default|blur|      backdrop (enum, 'blur',  ✓ forwarded to
    solid)                       reflect)                 sui-dialog
  ─────────────────────────────────────────────────────────────────────────────
  ship-auth-submit             sui-auth-submit          renamed per
                                                          convention
  ship-mode-change             sui-mode-change          renamed per
                                                          convention
  ship-close                   sui-close                renamed per
                                                          convention
  ─────────────────────────────────────────────────────────────────────────────
  slots: header, harnesses,    same                     ✓
    terms
  parts: dialog, form, submit  same                     ✓
  methods: show()/showModal()  exposed by controller    ✓ same names
    /close()
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="open" type="bool" default="false" reflect /> -->
<!-- <prop name="mode" values="sign-in|sign-up" default="sign-in" reflect /> -->
<!-- <prop name="loading" type="bool" default="false" reflect /> -->
<!-- <prop name="backdrop" values="default|blur|solid" default="blur" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<dialog
  size="sm"
  backdrop={backdrop()}
  bind:this={dialogRef}
  on:sail-close={onDialogClose}
>
  <div slot="header">
    <slot name="header">{defaultTitle()}</slot>
  </div>

  <div>
    <tabs
      variant="pill"
      active={mode()}
      on:sail-tab-change={onTabChange}
    >
      <tab name="sign-in" label="Sign in"></tab>
      <tab name="sign-up" label="Sign up"></tab>
    </tabs>

    <div>
      <slot name="harnesses" bind:this={providersSlot}></slot>
    </div>

    <if cond={hasProviders()}>
      <div>or</div>
    </if>

    <form on:submit={onSubmit} novalidate bind:this={formRef}>
      <label>
        Email
        <input
          name="email"
          type="email"
          autocomplete="email"
          inputmode="email"
          required
          placeholder="you@company.com"
        />
      </label>
      <label>
        Password
        <input
          name="password"
          type="password"
          autocomplete={passwordAutocomplete()}
          required
          minlength="8"
        />
      </label>

      <div>
        <button
          tone="accent"
          variant="solid"
          size="md"
          type="submit"
          loading={loading()}
          disabled={loading()}
        >{submitLabel()}</button>
        <p><slot name="terms"></slot></p>
      </div>
    </form>

    <p>
      {switchPrompt()}
      <button type="button" on:click={onSwitchClick}>{switchLabel()}</button>
    </p>
  </div>
</dialog>
