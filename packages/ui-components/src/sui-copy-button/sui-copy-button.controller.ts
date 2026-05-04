import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-copy-button.sui'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    let resetTimer: ReturnType<typeof setTimeout> | undefined

    const handleClick = async () => {
      try {
        await navigator.clipboard.writeText(scope.value())
      } catch {
        const ta = document.createElement('textarea')
        ta.value = scope.value()
        document.body.appendChild(ta)
        ta.select()
        try { document.execCommand('copy') } catch { /* ignore */ }
        ta.remove()
      }
      scope.copied.set(true)
      host.dispatchEvent(
        new CustomEvent('sui-copy', {
          detail: { value: scope.value() },
          bubbles: true,
          composed: true,
        }),
      )
      if (resetTimer) clearTimeout(resetTimer)
      resetTimer = setTimeout(() => { scope.copied.set(false) }, 1200)
    }

    const stopBtn = effect(() => {
      const btn = scope.buttonEl() as HTMLButtonElement | null
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
