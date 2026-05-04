import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-timestamp.sui'

const MINUTE = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

export function formatRelative(iso: string, now = Date.now()): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return iso
  const diff = now - t
  const abs = Math.abs(diff)
  const suffix = diff >= 0 ? 'ago' : 'from now'
  if (abs < MINUTE) return 'just now'
  if (abs < HOUR) return `${Math.floor(abs / MINUTE)}m ${suffix}`
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ${suffix}`
  const days = Math.floor(abs / DAY)
  if (days < 30) return `${days}d ${suffix}`
  if (days < 365) return `${Math.floor(days / 30)}mo ${suffix}`
  return `${Math.floor(days / 365)}y ${suffix}`
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const containerEl = scope.containerEl() as HTMLElement | null
    if (!containerEl) return

    let timeEl: HTMLTimeElement | null = null
    let absEl: HTMLSpanElement | null = null

    const render = () => {
      const v = scope.value()
      const fmt = scope.format()

      if (!v) {
        containerEl.innerHTML = ''
        timeEl = null
        absEl = null
        return
      }

      const parsed = Date.parse(v)
      const absStr = Number.isNaN(parsed) ? v : new Date(parsed).toISOString()
      const relStr = formatRelative(v)
      host.title = absStr

      if (!timeEl) {
        timeEl = document.createElement('time')
        timeEl.setAttribute('part', 'time')
        containerEl.innerHTML = ''
        containerEl.appendChild(timeEl)
        absEl = null
      }

      timeEl.setAttribute('datetime', v)

      if (fmt === 'absolute') {
        timeEl.textContent = absStr
        if (absEl) { absEl.remove(); absEl = null }
      } else if (fmt === 'both') {
        timeEl.textContent = relStr
        if (!absEl) {
          absEl = document.createElement('span')
          absEl.className = 'abs'
          absEl.setAttribute('part', 'abs')
          containerEl.appendChild(absEl)
        }
        absEl.textContent = `(${absStr})`
      } else {
        timeEl.textContent = relStr
        if (absEl) { absEl.remove(); absEl = null }
      }
    }

    const stopEffect = effect(() => {
      scope.value()
      scope.format()
      render()
    })

    const timer = setInterval(render, MINUTE)

    return () => {
      stopEffect()
      clearInterval(timer)
      timeEl = null
      absEl = null
    }
  },
})
