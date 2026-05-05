import { controller, signal, effect, ref } from '@ship/ui-runtime'
import type { Signal } from '@ship/ui-runtime'
import type { Scope } from './sui-segmented.sui'

interface OptionEl extends HTMLElement {
  value?: string
  icon?: string
  disabled?: boolean
}

interface Option {
  value: string
  label: string
  icon: string | null
  disabled: boolean
}

const cssEscape = (s: string): string =>
  typeof CSS !== 'undefined' && CSS.escape ? CSS.escape(s) : s.replace(/[^\w-]/g, '\\$&')

export default controller<Scope>({
  scope: () => {
    const options = signal<Option[]>([])
    const containerEl = ref<HTMLDivElement>(null)
    return { options, containerEl }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const optionsSig = scope.options as unknown as Signal<Option[]>

    if (!host.hasAttribute('role')) host.setAttribute('role', 'radiogroup')

    // Light-DOM hidden input for form participation.
    const hiddenInput = document.createElement('input')
    hiddenInput.type = 'hidden'
    host.appendChild(hiddenInput)

    const readOptions = (): Option[] => {
      const els = Array.from(
        host.querySelectorAll(':scope > sui-segmented-option'),
      ) as OptionEl[]
      return els.map((el) => ({
        value: el.getAttribute('value') ?? '',
        label: el.textContent?.trim() ?? '',
        icon: el.getAttribute('icon') || null,
        disabled: el.hasAttribute('disabled'),
      }))
    }

    const select = (val: string) => {
      if (scope.disabled() as boolean) return
      const opts = optionsSig()
      const opt = opts.find((o) => o.value === val)
      if (!opt || opt.disabled) return
      if ((scope.value() as string) === val) return
      ;(scope.value as unknown as Signal<string>).set(val)
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: val },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onClick = (e: MouseEvent) => {
      const btn = (e.target as Element | null)?.closest<HTMLButtonElement>('.btn')
      if (!btn) return
      const val = btn.getAttribute('data-value')
      if (val !== null) select(val)
    }

    const onKeyDown = (e: KeyboardEvent) => {
      const opts = optionsSig()
      const enabled = opts.filter((o) => !o.disabled && !(scope.disabled() as boolean))
      if (!enabled.length) return
      const values = enabled.map((o) => o.value)
      const cur = scope.value() as string
      const i = Math.max(0, values.indexOf(cur))
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

    // Mirror light-DOM children into the options signal so the template
    // renders one button per option.
    optionsSig.set(readOptions())
    const observer = new MutationObserver(() => optionsSig.set(readOptions()))
    observer.observe(host, { childList: true, subtree: false })

    // Keep the form-participation input in sync.
    const stopHidden = effect(() => {
      hiddenInput.name = scope.name() as string
      hiddenInput.value = scope.value() as string
    })

    // Wire delegated click + keydown on the shadow container once it's
    // bound. The container is a bind:this ref in the template.
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
      observer.disconnect()
      stopHidden()
      stopContainer()
      hiddenInput.remove()
    }
  },
})
