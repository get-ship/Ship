import { bindAttr, bindNativeEvent, controller, onCleanup } from '@ship/ui-runtime'
import type { Scope } from './sui-input.sui'

let _idSeq = 0

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const inputEl = scope.inputEl()
    if (!inputEl) return
    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      clear: () => { inputEl.value = ''; scope.value.set('') },
      reportValidity: () => inputEl.reportValidity(),
    })
    bindNativeEvent(inputEl, 'input', () =>
      host.emit!('sui-input', { value: inputEl.value }))
    bindNativeEvent(inputEl, 'change', () =>
      host.emit!('sui-input-change', { value: inputEl.value }))
    const errId = `sui-input-err-${++_idSeq}`
    onCleanup(bindAttr(inputEl, 'aria-describedby', () => {
      const parts: string[] = []
      if (scope.invalid() && scope.errorMessage()) {
        const errRef = (scope as unknown as { errorEl?: () => HTMLSpanElement | null }).errorEl
        const errorEl = typeof errRef === 'function' ? errRef() : null
        if (errorEl) { errorEl.id = errId; parts.push(errId) }
      }
      const extra = scope.describedBy()
      if (extra) parts.push(extra)
      return parts.length > 0 ? parts.join(' ') : null
    }))
  },
})
