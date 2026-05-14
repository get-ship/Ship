import { controller, signal, effect, ref } from '@ship/sui-runtime'
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
      if (state.disabled()) return
      const opts = state.options()
      const opt = opts.find((o) => o.value === val)
      if (!opt || opt.disabled) return
      if (state.value() === val) return
      state.value.set(val)
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
      const opts = state.options()
      const enabled = opts.filter((o) => !o.disabled && !state.disabled())
      if (!enabled.length) return
      const values = enabled.map((o) => o.value)
      const cur = state.value()
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

    state.options.set(readOptions())
    const observer = new MutationObserver(() => state.options.set(readOptions()))
    observer.observe(host, { childList: true, subtree: false })

    const stopHidden = effect(() => {
      hiddenInput.name = state.name()
      hiddenInput.value = state.value()
    })

    const stopContainer = effect(() => {
      const container = state.containerEl()
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
