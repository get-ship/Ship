import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-radio-group.sui'

type RadioEl = HTMLElement & { focusInput?(): void }

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'radiogroup')

    // Light-DOM hidden input for form participation.
    const hiddenInput = document.createElement('input')
    hiddenInput.type = 'hidden'
    host.appendChild(hiddenInput)

    const getRadios = (): RadioEl[] =>
      Array.from(host.querySelectorAll(':scope > sui-radio')) as RadioEl[]

    const enabledRadios = (): RadioEl[] =>
      getRadios().filter(r => !r.hasAttribute('disabled') && !(scope.disabled() as boolean))

    const syncRadios = () => {
      const currentValue = scope.value() as string
      const isGroupDisabled = scope.disabled() as boolean
      const groupTone = scope.tone() as string
      const groupSize = scope.size() as string

      for (const radio of getRadios()) {
        radio.toggleAttribute('checked', radio.getAttribute('value') === currentValue)
        if (isGroupDisabled) radio.setAttribute('disabled', '')
        if (groupTone && !radio.hasAttribute('tone')) radio.setAttribute('tone', groupTone)
        if (groupSize && !radio.hasAttribute('size')) radio.setAttribute('size', groupSize)
      }

      hiddenInput.name = scope.name() as string
      hiddenInput.value = currentValue
    }

    const stopSync = effect(() => {
      scope.value()
      scope.disabled()
      scope.tone()
      scope.size()
      syncRadios()
    })

    const stopLabel = effect(() => {
      const label = scope.label() as string
      if (label) host.setAttribute('aria-label', label)
      else host.removeAttribute('aria-label')
    })

    const stopRequired = effect(() => {
      const required = scope.required() as boolean
      if (required) host.setAttribute('aria-required', 'true')
      else host.removeAttribute('aria-required')
    })

    const onSelect = (e: Event) => {
      const ce = e as CustomEvent
      ce.stopPropagation()
      const newVal = ce.detail.value as string
      if (newVal === (scope.value() as string)) return
      scope.value.set(newVal)
      syncRadios()
      host.dispatchEvent(new CustomEvent('sui-change', {
        detail: { value: newVal },
        bubbles: true,
        composed: true,
      }))
    }

    const onNav = (e: Event) => {
      const ce = e as CustomEvent
      ce.stopPropagation()
      const enabled = enabledRadios()
      if (!enabled.length) return

      const current = (ce.composedPath() as Element[]).find(
        el => el.tagName?.toLowerCase() === 'sui-radio',
      ) as RadioEl | null
      const idx = current ? enabled.indexOf(current) : -1
      const next = ce.detail.direction === 'next'
        ? enabled[(idx + 1) % enabled.length]!
        : enabled[(idx - 1 + enabled.length) % enabled.length]!

      const nextVal = next.getAttribute('value') ?? ''
      scope.value.set(nextVal)
      syncRadios()
      next.focusInput?.()

      host.dispatchEvent(new CustomEvent('sui-change', {
        detail: { value: nextVal },
        bubbles: true,
        composed: true,
      }))
    }

    host.addEventListener('sui-radio-select', onSelect)
    host.addEventListener('sui-radio-nav', onNav)

    return () => {
      stopSync()
      stopLabel()
      stopRequired()
      host.removeEventListener('sui-radio-select', onSelect)
      host.removeEventListener('sui-radio-nav', onNav)
      hiddenInput.remove()
    }
  },
})
