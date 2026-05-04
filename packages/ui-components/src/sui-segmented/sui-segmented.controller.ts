import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-segmented.sui'

interface OptionEl extends HTMLElement {
  value?: string
  icon?: string
  disabled?: boolean
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'radiogroup')

    // Light-DOM hidden input for form participation.
    const hiddenInput = document.createElement('input')
    hiddenInput.type = 'hidden'
    host.appendChild(hiddenInput)

    const getOptions = (): OptionEl[] =>
      Array.from(host.querySelectorAll(':scope > sui-segmented-option')) as OptionEl[]

    const cssEscape = (s: string): string =>
      typeof CSS !== 'undefined' && CSS.escape ? CSS.escape(s) : s.replace(/[^\w-]/g, '\\$&')

    const select = (value: string) => {
      if (scope.disabled() as boolean) return
      if ((scope.value() as string) === value) return
      scope.value.set(value)
      hiddenInput.name = scope.name() as string
      hiddenInput.value = value
      host.dispatchEvent(new CustomEvent('sui-change', {
        detail: { value },
        bubbles: true,
        composed: true,
      }))
    }

    const renderButtons = () => {
      const container = scope.containerEl() as HTMLElement | null
      if (!container) return
      const options = getOptions()
      const currentValue = scope.value() as string
      const isGroupDisabled = scope.disabled() as boolean

      // Sync hidden input name.
      hiddenInput.name = scope.name() as string
      hiddenInput.value = currentValue

      // Rebuild buttons only when count changes; otherwise sync attrs.
      const existing = Array.from(container.querySelectorAll<HTMLButtonElement>('.btn'))
      if (existing.length !== options.length) {
        container.innerHTML = ''
        for (const opt of options) {
          const btn = document.createElement('button')
          btn.className = 'btn'
          btn.setAttribute('role', 'radio')
          btn.type = 'button'
          container.appendChild(btn)
          if (opt.getAttribute('icon')) {
            const iconWrap = document.createElement('span')
            iconWrap.className = 'btn-icon'
            iconWrap.setAttribute('aria-hidden', 'true')
            const icon = document.createElement('sui-icon')
            icon.setAttribute('name', opt.getAttribute('icon') ?? '')
            icon.setAttribute('size', 'sm')
            iconWrap.appendChild(icon)
            btn.appendChild(iconWrap)
          }
          const label = document.createTextNode(opt.textContent?.trim() ?? '')
          btn.appendChild(label)
        }
      }

      // Sync state on all buttons.
      const buttons = Array.from(container.querySelectorAll<HTMLButtonElement>('.btn'))
      options.forEach((opt, i) => {
        const btn = buttons[i]
        if (!btn) return
        const optValue = opt.getAttribute('value') ?? ''
        const isOptDisabled = opt.hasAttribute('disabled')
        btn.setAttribute('data-value', optValue)
        btn.setAttribute('aria-checked', optValue === currentValue ? 'true' : 'false')
        btn.setAttribute('aria-disabled', (isGroupDisabled || isOptDisabled) ? 'true' : 'false')
        btn.disabled = isGroupDisabled || isOptDisabled
        btn.tabIndex = optValue === currentValue ? 0 : -1
      })
    }

    const onKeyDown = (e: KeyboardEvent) => {
      const options = getOptions()
      const enabled = options.filter(o => !o.hasAttribute('disabled') && !(scope.disabled() as boolean))
      if (!enabled.length) return
      const values = enabled.map(o => o.getAttribute('value') ?? '')
      const i = Math.max(0, values.indexOf(scope.value() as string))
      let next = i

      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
        e.preventDefault()
        next = (i + 1) % values.length
      } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
        e.preventDefault()
        next = (i - 1 + values.length) % values.length
      } else if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        const focused = (host.shadowRoot as ShadowRoot).activeElement as HTMLButtonElement | null
        const focusedValue = focused?.getAttribute('data-value')
        if (focusedValue) select(focusedValue)
        return
      } else {
        return
      }

      const targetValue = values[next] ?? ''
      select(targetValue)
      queueMicrotask(() => {
        const btn = host.shadowRoot?.querySelector<HTMLButtonElement>(
          `.btn[data-value="${cssEscape(targetValue)}"]`,
        )
        btn?.focus()
      })
    }

    const onClick = (e: MouseEvent) => {
      const btn = (e.target as Element).closest<HTMLButtonElement>('.btn')
      if (!btn) return
      const optValue = btn.getAttribute('data-value')
      if (optValue !== null) select(optValue)
    }

    const stopRender = effect(() => {
      scope.value()
      scope.disabled()
      renderButtons()
    })

    // Watch for light-DOM child additions/mutations (new options, attribute changes).
    // Watch only for option additions/removals; attribute changes on options are
    // covered by the reactive effect tracking scope.value() / scope.disabled().
    // Using `attributes: true` here would create an infinite loop because
    // renderButtons() mutates the hidden input's `value` attribute, which is a
    // host subtree descendant and would re-fire the observer.
    const observer = new MutationObserver(() => renderButtons())
    observer.observe(host, { childList: true, subtree: false })

    const stopContainer = effect(() => {
      const container = scope.containerEl() as HTMLElement | null
      if (!container) return
      container.addEventListener('click', onClick)
      container.addEventListener('keydown', onKeyDown)
      return () => {
        container.removeEventListener('click', onClick)
        container.removeEventListener('keydown', onKeyDown)
      }
    })

    return () => {
      stopRender()
      stopContainer()
      observer.disconnect()
      hiddenInput.remove()
    }
  },
})
