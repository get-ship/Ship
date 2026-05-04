import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-field.sui'

let _idSeq = 0

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const hintId = `sui-field-hint-${++_idSeq}`
    const errorId = `sui-field-error-${++_idSeq}`

    // Assign stable IDs to hint and error containers.
    const stopIds = effect(() => {
      const hint = scope.hintEl()
      const error = scope.errorEl()
      if (hint) hint.id = hintId
      if (error) error.id = errorId
    })

    const wireControl = () => {
      const slot = scope.controlSlot() as HTMLSlotElement | null
      const first = slot?.assignedElements({ flatten: true })[0] as HTMLElement | undefined
      if (!first) return
      const inv = scope.invalid()
      const targetId = inv ? errorId : hintId
      const prior = (first.getAttribute('aria-describedby') || '')
        .split(/\s+/)
        .filter(id => id && id !== hintId && id !== errorId)
      const final = [...prior, targetId].join(' ').trim()
      if (final) first.setAttribute('aria-describedby', final)
      if ('invalid' in first) (first as unknown as { invalid: boolean }).invalid = inv
      else first.setAttribute('aria-invalid', inv ? 'true' : 'false')
    }

    // Re-wire whenever invalid changes.
    const stopInvalid = effect(() => {
      scope.invalid() // track dependency
      wireControl()
    })

    // Re-wire when slotted elements change.
    const stopSlot = effect(() => {
      const slot = scope.controlSlot() as HTMLSlotElement | null
      if (!slot) return
      slot.addEventListener('slotchange', wireControl)
      return () => slot.removeEventListener('slotchange', wireControl)
    })

    return () => {
      stopIds()
      stopInvalid()
      stopSlot()
    }
  },
})
