import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-input.sui'

let _idSeq = 0

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const errId = `sui-input-err-${++_idSeq}`

    // Proxy imperative API to inner input.
    const h = host as HTMLElement & {
      focus?: () => void
      blur?: () => void
      clear?: () => void
      reportValidity?: () => boolean
    }
    h.focus = () => scope.inputEl()?.focus()
    h.blur = () => scope.inputEl()?.blur()
    h.clear = () => {
      const el = scope.inputEl()
      if (!el) return
      el.value = ''
      scope.value.set('')
    }
    h.reportValidity = () => scope.inputEl()?.reportValidity() ?? true

    // Wire native input + change events → host-dispatched custom events.
    // bind:value in the template already syncs el.value ↔ value signal;
    // we only re-dispatch here.
    const stopInputListener = effect(() => {
      const el = scope.inputEl()
      if (!el) return

      const onNativeInput = () => {
        host.dispatchEvent(new CustomEvent('sui-input', {
          detail: { value: el.value }, bubbles: true, composed: true,
        }))
      }

      const onNativeChange = () => {
        host.dispatchEvent(new CustomEvent('sui-input-change', {
          detail: { value: el.value }, bubbles: true, composed: true,
        }))
      }

      el.addEventListener('input', onNativeInput)
      el.addEventListener('change', onNativeChange)
      return () => {
        el.removeEventListener('input', onNativeInput)
        el.removeEventListener('change', onNativeChange)
      }
    })

    // Wire aria-describedby: link the input to the error span when invalid +
    // errorMessage are both set, composed with any external described-by value.
    const stopDescribed = effect(() => {
      const inputEl = scope.inputEl()
      if (!inputEl) return

      const parts: string[] = []

      if (scope.invalid() && scope.errorMessage()) {
        // errorEl is only set when the {#if invalid() && errorMessage()} branch
        // renders. The Scope type declares it non-optional but at runtime it
        // starts undefined; guard before calling.
        const errorEl: HTMLSpanElement | null =
          typeof (scope as unknown as { errorEl?: () => HTMLSpanElement | null }).errorEl === 'function'
            ? ((scope as unknown as { errorEl: () => HTMLSpanElement | null }).errorEl)()
            : null
        if (errorEl) {
          errorEl.id = errId
          parts.push(errId)
        }
      }

      const extra = scope.describedBy()
      if (extra) parts.push(extra)

      if (parts.length > 0) {
        inputEl.setAttribute('aria-describedby', parts.join(' '))
      } else {
        inputEl.removeAttribute('aria-describedby')
      }
    })

    return () => {
      stopInputListener()
      stopDescribed()
    }
  },
})
