import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-copy-button.sui'

export default controller<Scope>({
  mount(state, host) {
    let resetTimer: ReturnType<typeof setTimeout> | undefined

    const handleClick = async () => {
      try {
        await navigator.clipboard.writeText(state.$signals.value())
      } catch {
        const ta = document.createElement('textarea')
        ta.value = state.$signals.value()
        document.body.appendChild(ta)
        ta.select()
        try { document.execCommand('copy') } catch { /* ignore */ }
        ta.remove()
      }
      state.$signals.copied.set(true)
      host.dispatchEvent(
        new CustomEvent('sui-copy', {
          detail: { value: state.$signals.value() },
          bubbles: true,
          composed: true,
        }),
      )
      if (resetTimer) clearTimeout(resetTimer)
      resetTimer = setTimeout(() => { state.$signals.copied.set(false) }, 1200)
    }

    const stopBtn = effect(() => {
      const btn = state.$signals.buttonEl() as HTMLButtonElement | null
      if (!btn) return
      btn.addEventListener('click', handleClick)
      return () => btn.removeEventListener('click', handleClick)
    })

    return () => {
      stopBtn()
      if (resetTimer) clearTimeout(resetTimer)
    }
  },
})
