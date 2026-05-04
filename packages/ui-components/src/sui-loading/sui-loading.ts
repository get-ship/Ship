import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-loading.sui'

const DEFAULT_MESSAGES = ['Waking daemon', 'Warming routes', 'Fetching state', 'Hydrating shell']

function parseMessages(raw: string): string[] {
  const trimmed = raw.trim()
  if (!trimmed) return DEFAULT_MESSAGES
  return trimmed.split('|').map((s) => s.trim()).filter(Boolean)
}

export default controller<Scope>({
  mount(state, _host) {
    const scope = state as unknown as Scope
    const msgEl = scope.msgEl() as HTMLElement | null
    if (!msgEl) return

    let index = 0

    const showMsg = () => {
      const msgs = parseMessages(scope.messages() as string)
      msgEl.textContent = msgs[index] ?? ''
      msgEl.style.opacity = '1'
    }
    showMsg()

    const timer = setInterval(() => {
      msgEl.style.opacity = '0'
      requestAnimationFrame(() => {
        const msgs = parseMessages(scope.messages() as string)
        index = (index + 1) % msgs.length
        msgEl.textContent = msgs[index] ?? ''
        msgEl.style.opacity = '1'
      })
    }, 1400)

    return () => clearInterval(timer)
  },
})
